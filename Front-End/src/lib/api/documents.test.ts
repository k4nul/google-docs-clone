import { afterEach, describe, expect, it, vi } from 'vitest';

import {
  createBackendDocument,
  getBackendDocument,
  getStoredDocumentAccessToken,
  listBackendDocuments,
  storeDocumentAccessToken,
  updateBackendDocumentSecurity,
  updateBackendDocumentTitle,
} from './documents';

const originalLocalStorageDescriptor = Object.getOwnPropertyDescriptor(
  window,
  'localStorage',
);
const documentCredentialsStorageKey =
  'realtime-docs.document-credentials.v1';

function jsonResponse(body: unknown, init?: ResponseInit) {
  return new Response(JSON.stringify(body), {
    status: 200,
    ...init,
    headers: {
      'Content-Type': 'application/json',
      ...(init?.headers ?? {}),
    },
  });
}

function restoreLocalStorage() {
  if (originalLocalStorageDescriptor) {
    Object.defineProperty(window, 'localStorage', originalLocalStorageDescriptor);
  }
}

describe('document API client', () => {
  afterEach(() => {
    restoreLocalStorage();
    window.localStorage.clear();
    vi.unstubAllGlobals();
  });

  it('lists backend documents as newest-first document summaries', async () => {
    const fetchMock = vi.fn().mockResolvedValue(
      jsonResponse({
        documents: [
          {
            id: '11111111-1111-4111-8111-111111111111',
            title: 'Older document',
            created_at: '2026-04-01T10:00:00.000Z',
            updated_at: '2026-04-01T10:00:00.000Z',
            preview: 'First line of older content.',
          },
          {
            id: '22222222-2222-4222-8222-222222222222',
            title: 'Newer document',
            created_at: '2026-04-02T10:00:00.000Z',
            updated_at: '2026-04-04T10:00:00.000Z',
            collaborator_count: 3,
            preview: 'First line of newer content.',
          },
        ],
      }),
    );
    vi.stubGlobal('fetch', fetchMock);

    await expect(listBackendDocuments()).resolves.toEqual([
      {
        id: '22222222-2222-4222-8222-222222222222',
        title: 'Newer document',
        summary: 'First line of newer content.',
        createdAt: '2026-04-02T10:00:00.000Z',
        updatedAt: '2026-04-04T10:00:00.000Z',
        collaborators: 3,
      },
      {
        id: '11111111-1111-4111-8111-111111111111',
        title: 'Older document',
        summary: 'First line of older content.',
        createdAt: '2026-04-01T10:00:00.000Z',
        updatedAt: '2026-04-01T10:00:00.000Z',
        collaborators: 0,
      },
    ]);
    expect(fetchMock).toHaveBeenCalledWith(
      expect.stringContaining('/api/documents'),
      expect.any(Object),
    );
  });

  it('uses neutral placeholders when list previews are unavailable or hidden', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn().mockResolvedValue(
        jsonResponse({
          documents: [
            {
              id: '55555555-5555-4555-8555-555555555555',
              title: 'Hidden preview',
              created_at: '2026-04-01T10:00:00.000Z',
              updated_at: '2026-04-01T10:00:00.000Z',
              hide_preview: true,
              preview: 'Sensitive content that should not be shown.',
            },
            {
              id: '66666666-6666-4666-8666-666666666666',
              title: 'Missing preview',
              created_at: '2026-04-01T10:00:00.000Z',
              updated_at: '2026-04-01T10:00:00.000Z',
            },
          ],
        }),
      ),
    );

    await expect(listBackendDocuments()).resolves.toEqual(
      expect.arrayContaining([
        expect.objectContaining({
          id: '55555555-5555-4555-8555-555555555555',
          summary: 'Preview hidden',
        }),
        expect.objectContaining({
          id: '66666666-6666-4666-8666-666666666666',
          summary: 'No preview available',
        }),
      ]),
    );
  });

  it('normalizes list summaries from preview text before falling back to summary text', async () => {
    const longPreview = `${'Long preview text '.repeat(12)}tail`;
    vi.stubGlobal(
      'fetch',
      vi.fn().mockResolvedValue(
        jsonResponse({
          documents: [
            {
              id: '10101010-1010-4010-8010-101010101010',
              title: 'Whitespace preview',
              created_at: '2026-04-01T10:00:00.000Z',
              updated_at: '2026-04-01T10:00:00.000Z',
              preview: '  First line\n\nsecond\tline  ',
              summary: 'Summary should not win.',
            },
            {
              id: '20202020-2020-4020-8020-202020202020',
              title: 'Summary fallback',
              created_at: '2026-04-01T10:00:00.000Z',
              updated_at: '2026-04-01T10:00:00.000Z',
              preview: '   ',
              summary: '  Summary\nfallback  ',
            },
            {
              id: '30303030-3030-4030-8030-303030303030',
              title: 'Long preview',
              created_at: '2026-04-01T10:00:00.000Z',
              updated_at: '2026-04-01T10:00:00.000Z',
              preview: longPreview,
            },
          ],
        }),
      ),
    );

    await expect(listBackendDocuments()).resolves.toEqual(
      expect.arrayContaining([
        expect.objectContaining({
          id: '10101010-1010-4010-8010-101010101010',
          summary: 'First line second line',
        }),
        expect.objectContaining({
          id: '20202020-2020-4020-8020-202020202020',
          summary: 'Summary fallback',
        }),
        expect.objectContaining({
          id: '30303030-3030-4030-8030-303030303030',
          summary: `${longPreview.slice(0, 177)}...`,
        }),
      ]),
    );
  });

  it('honors both hidden-preview flags and collaborator count aliases in list results', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn().mockResolvedValue(
        jsonResponse({
          documents: [
            {
              id: '40404040-4040-4040-8040-404040404040',
              title: 'Legacy hidden flag',
              created_at: '2026-04-01T10:00:00.000Z',
              updated_at: '2026-04-01T10:00:00.000Z',
              preview_hidden: true,
              preview: 'Sensitive legacy preview.',
              collaborators: 5,
              collaborator_count: 2,
            },
            {
              id: '50505050-5050-4050-8050-505050505050',
              title: 'Count fallback',
              created_at: '2026-04-01T10:00:00.000Z',
              updated_at: '2026-04-01T10:00:00.000Z',
              preview: 'Visible preview.',
              collaborator_count: 4,
            },
          ],
        }),
      ),
    );

    await expect(listBackendDocuments()).resolves.toEqual(
      expect.arrayContaining([
        expect.objectContaining({
          id: '40404040-4040-4040-8040-404040404040',
          collaborators: 5,
          summary: 'Preview hidden',
        }),
        expect.objectContaining({
          id: '50505050-5050-4050-8050-505050505050',
          collaborators: 4,
          summary: 'Visible preview.',
        }),
      ]),
    );
  });

  it('keeps documents with invalid update timestamps behind dated documents', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn().mockResolvedValue(
        jsonResponse({
          documents: [
            {
              id: '60606060-6060-4060-8060-606060606060',
              title: 'Undated draft',
              created_at: 'not-a-date',
              updated_at: 'not-a-date',
            },
            {
              id: '70707070-7070-4070-8070-707070707070',
              title: 'Recent draft',
              created_at: '2026-04-01T10:00:00.000Z',
              updated_at: '2026-04-06T10:00:00.000Z',
            },
          ],
        }),
      ),
    );

    await expect(listBackendDocuments()).resolves.toMatchObject([
      {
        id: '70707070-7070-4070-8070-707070707070',
        title: 'Recent draft',
      },
      {
        id: '60606060-6060-4060-8060-606060606060',
        title: 'Undated draft',
      },
    ]);
  });

  it('fetches one backend document by encoded id', async () => {
    const fetchMock = vi.fn().mockResolvedValue(
      jsonResponse({
        document: {
          id: '33333333-3333-4333-8333-333333333333',
          title: 'Loaded document',
          created_at: '2026-04-03T10:00:00.000Z',
          updated_at: '2026-04-03T10:05:00.000Z',
          hide_preview: true,
        },
      }),
    );
    vi.stubGlobal('fetch', fetchMock);

    await expect(
      getBackendDocument(
        '33333333-3333-4333-8333-333333333333',
        'doc-token',
      ),
    ).resolves.toEqual({
      id: '33333333-3333-4333-8333-333333333333',
      title: 'Loaded document',
      createdAt: '2026-04-03T10:00:00.000Z',
      updatedAt: '2026-04-03T10:05:00.000Z',
      hidePreview: true,
    });
    expect(fetchMock).toHaveBeenCalledWith(
      expect.stringContaining(
        '/api/documents/33333333-3333-4333-8333-333333333333',
      ),
      expect.objectContaining({
        headers: expect.objectContaining({
          Authorization: 'Bearer doc-token',
        }),
      }),
    );
  });

  it('creates backend documents and stores the returned document credential', async () => {
    const fetchMock = vi.fn().mockResolvedValue(
      jsonResponse({
        document: {
          id: '44444444-4444-4444-8444-444444444444',
          title: 'Realtime draft',
          created_at: '2026-04-04T10:00:00.000Z',
          updated_at: '2026-04-04T10:00:00.000Z',
        },
        credentials: {
          access_token: 'created-doc-token',
        },
      }),
    );
    vi.stubGlobal('fetch', fetchMock);

    await expect(createBackendDocument('Realtime draft')).resolves.toEqual({
      document: {
        id: '44444444-4444-4444-8444-444444444444',
        title: 'Realtime draft',
        createdAt: '2026-04-04T10:00:00.000Z',
        updatedAt: '2026-04-04T10:00:00.000Z',
        hidePreview: false,
      },
      credentials: {
        accessToken: 'created-doc-token',
      },
    });
    expect(
      getStoredDocumentAccessToken('44444444-4444-4444-8444-444444444444'),
    ).toBe('created-doc-token');
    expect(fetchMock).toHaveBeenCalledWith(
      expect.stringContaining('/api/documents'),
      expect.objectContaining({
        method: 'POST',
        body: JSON.stringify({ title: 'Realtime draft' }),
      }),
    );
  });

  it('trims returned document credentials before exposing and storing them', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn().mockResolvedValue(
        jsonResponse({
          document: {
            id: '99999999-9999-4999-8999-999999999999',
            title: 'Whitespace credential draft',
            created_at: '2026-04-04T10:00:00.000Z',
            updated_at: '2026-04-04T10:00:00.000Z',
          },
          credentials: {
            access_token: '  created-doc-token  ',
          },
        }),
      ),
    );

    await expect(
      createBackendDocument('Whitespace credential draft'),
    ).resolves.toMatchObject({
      credentials: {
        accessToken: 'created-doc-token',
      },
    });
    expect(
      getStoredDocumentAccessToken('99999999-9999-4999-8999-999999999999'),
    ).toBe('created-doc-token');
  });

  it('rejects create responses without a usable document credential', async () => {
    const fetchMock = vi.fn().mockResolvedValue(
      jsonResponse({
        document: {
          id: '77777777-7777-4777-8777-777777777777',
          title: 'Missing credential draft',
          created_at: '2026-04-04T10:00:00.000Z',
          updated_at: '2026-04-04T10:00:00.000Z',
        },
        credentials: {
          access_token: '   ',
        },
      }),
    );
    vi.stubGlobal('fetch', fetchMock);

    await expect(createBackendDocument('Missing credential draft')).rejects.toThrow(
      'Document creation response did not include an access token.',
    );
    expect(
      getStoredDocumentAccessToken('77777777-7777-4777-8777-777777777777'),
    ).toBeNull();
  });

  it('fails closed when browser credential storage is unavailable', () => {
    Object.defineProperty(window, 'localStorage', {
      configurable: true,
      get() {
        throw new DOMException('Storage is blocked.', 'SecurityError');
      },
    });

    expect(() =>
      storeDocumentAccessToken(
        '88888888-8888-4888-8888-888888888888',
        'doc-token',
      ),
    ).not.toThrow();
    expect(
      getStoredDocumentAccessToken('88888888-8888-4888-8888-888888888888'),
    ).toBeNull();
  });

  it('stores trimmed credentials without overwriting existing entries with blanks', () => {
    storeDocumentAccessToken(
      'aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa',
      '  first-doc-token  ',
    );
    storeDocumentAccessToken(
      'bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb',
      'second-doc-token',
    );
    storeDocumentAccessToken('aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa', '   ');

    expect(
      getStoredDocumentAccessToken('aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa'),
    ).toBe('first-doc-token');
    expect(
      getStoredDocumentAccessToken('bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb'),
    ).toBe('second-doc-token');
  });

  it('does not call the backend when stored credentials are malformed', async () => {
    window.localStorage.setItem(documentCredentialsStorageKey, 'not-json');
    const fetchMock = vi.fn();
    vi.stubGlobal('fetch', fetchMock);

    await expect(
      updateBackendDocumentTitle(
        'cccccccc-cccc-4ccc-8ccc-cccccccccccc',
        'Renamed draft',
      ),
    ).rejects.toMatchObject({
      name: 'MissingDocumentCredentialError',
      documentId: 'cccccccc-cccc-4ccc-8ccc-cccccccccccc',
    });
    expect(fetchMock).not.toHaveBeenCalled();
  });

  it('renames backend documents with the stored document credential', async () => {
    window.localStorage.setItem(
      documentCredentialsStorageKey,
      JSON.stringify({
        '55555555-5555-4555-8555-555555555555': 'stored-doc-token',
      }),
    );
    const fetchMock = vi.fn().mockResolvedValue(
      jsonResponse({
        document: {
          id: '55555555-5555-4555-8555-555555555555',
          title: 'Renamed draft',
          created_at: '2026-04-04T10:00:00.000Z',
          updated_at: '2026-04-04T10:05:00.000Z',
        },
      }),
    );
    vi.stubGlobal('fetch', fetchMock);

    await expect(
      updateBackendDocumentTitle(
        '55555555-5555-4555-8555-555555555555',
        'Renamed draft',
      ),
    ).resolves.toEqual({
      id: '55555555-5555-4555-8555-555555555555',
      title: 'Renamed draft',
      createdAt: '2026-04-04T10:00:00.000Z',
      updatedAt: '2026-04-04T10:05:00.000Z',
      hidePreview: false,
    });
    expect(fetchMock).toHaveBeenCalledWith(
      expect.stringContaining(
        '/api/documents/55555555-5555-4555-8555-555555555555',
      ),
      expect.objectContaining({
        method: 'PATCH',
        body: JSON.stringify({ title: 'Renamed draft' }),
        headers: expect.objectContaining({
          Authorization: 'Bearer stored-doc-token',
        }),
      }),
    );
  });

  it('prefers an explicit document credential over a stored title credential', async () => {
    window.localStorage.setItem(
      documentCredentialsStorageKey,
      JSON.stringify({
        '55555555-5555-4555-8555-555555555555': 'stale-doc-token',
      }),
    );
    const fetchMock = vi.fn().mockResolvedValue(
      jsonResponse({
        document: {
          id: '55555555-5555-4555-8555-555555555555',
          title: 'Explicit token draft',
          created_at: '2026-04-04T10:00:00.000Z',
          updated_at: '2026-04-04T10:05:00.000Z',
        },
      }),
    );
    vi.stubGlobal('fetch', fetchMock);

    await updateBackendDocumentTitle(
      '55555555-5555-4555-8555-555555555555',
      'Explicit token draft',
      '  fresh-doc-token  ',
    );

    expect(fetchMock).toHaveBeenCalledWith(
      expect.stringContaining(
        '/api/documents/55555555-5555-4555-8555-555555555555',
      ),
      expect.objectContaining({
        headers: expect.objectContaining({
          Authorization: 'Bearer fresh-doc-token',
        }),
      }),
    );
  });

  it('updates backend document security settings with the stored credential', async () => {
    window.localStorage.setItem(
      documentCredentialsStorageKey,
      JSON.stringify({
        '55555555-5555-4555-8555-555555555555': 'stored-doc-token',
      }),
    );
    const fetchMock = vi.fn().mockResolvedValue(
      jsonResponse({
        document: {
          id: '55555555-5555-4555-8555-555555555555',
          title: 'Secured draft',
          created_at: '2026-04-04T10:00:00.000Z',
          updated_at: '2026-04-04T10:05:00.000Z',
          hide_preview: true,
        },
      }),
    );
    vi.stubGlobal('fetch', fetchMock);

    await expect(
      updateBackendDocumentSecurity(
        '55555555-5555-4555-8555-555555555555',
        { hidePreview: true },
      ),
    ).resolves.toEqual({
      id: '55555555-5555-4555-8555-555555555555',
      title: 'Secured draft',
      createdAt: '2026-04-04T10:00:00.000Z',
      updatedAt: '2026-04-04T10:05:00.000Z',
      hidePreview: true,
    });
    expect(fetchMock).toHaveBeenCalledWith(
      expect.stringContaining(
        '/api/documents/55555555-5555-4555-8555-555555555555',
      ),
      expect.objectContaining({
        method: 'PATCH',
        body: JSON.stringify({ hide_preview: true }),
        headers: expect.objectContaining({
          Authorization: 'Bearer stored-doc-token',
        }),
      }),
    );
  });

  it('encodes document ids before updating backend document security', async () => {
    const documentId = 'draft folder/one?rev=2';
    const fetchMock = vi.fn().mockResolvedValue(
      jsonResponse({
        document: {
          id: documentId,
          title: 'Encoded id draft',
          created_at: '2026-04-04T10:00:00.000Z',
          updated_at: '2026-04-04T10:05:00.000Z',
          hide_preview: false,
        },
      }),
    );
    vi.stubGlobal('fetch', fetchMock);

    await expect(
      updateBackendDocumentSecurity(
        documentId,
        { hidePreview: false },
        'encoded-doc-token',
      ),
    ).resolves.toMatchObject({
      id: documentId,
      hidePreview: false,
    });
    expect(fetchMock).toHaveBeenCalledWith(
      expect.stringContaining(
        `/api/documents/${encodeURIComponent(documentId)}`,
      ),
      expect.objectContaining({
        method: 'PATCH',
        body: JSON.stringify({ hide_preview: false }),
        headers: expect.objectContaining({
          Authorization: 'Bearer encoded-doc-token',
        }),
      }),
    );
  });

  it('rejects document detail requests when no document credential is stored', async () => {
    await expect(
      getBackendDocument('66666666-6666-4666-8666-666666666666'),
    ).rejects.toMatchObject({
      name: 'MissingDocumentCredentialError',
      documentId: '66666666-6666-4666-8666-666666666666',
    });
  });
});
