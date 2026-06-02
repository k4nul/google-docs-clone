import { afterEach, describe, expect, it, vi } from 'vitest';

import {
  createBackendDocument,
  getBackendDocument,
  listBackendDocuments,
} from './documents';

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

describe('document API client', () => {
  afterEach(() => {
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

  it('fetches one backend document by encoded id', async () => {
    const fetchMock = vi.fn().mockResolvedValue(
      jsonResponse({
        document: {
          id: '33333333-3333-4333-8333-333333333333',
          title: 'Loaded document',
          created_at: '2026-04-03T10:00:00.000Z',
          updated_at: '2026-04-03T10:05:00.000Z',
        },
      }),
    );
    vi.stubGlobal('fetch', fetchMock);

    await expect(
      getBackendDocument('33333333-3333-4333-8333-333333333333'),
    ).resolves.toEqual({
      id: '33333333-3333-4333-8333-333333333333',
      title: 'Loaded document',
      createdAt: '2026-04-03T10:00:00.000Z',
      updatedAt: '2026-04-03T10:05:00.000Z',
    });
    expect(fetchMock).toHaveBeenCalledWith(
      expect.stringContaining(
        '/api/documents/33333333-3333-4333-8333-333333333333',
      ),
      expect.any(Object),
    );
  });

  it('creates backend documents without requiring a frontend token', async () => {
    const fetchMock = vi.fn().mockResolvedValue(
      jsonResponse({
        document: {
          id: '44444444-4444-4444-8444-444444444444',
          title: 'Realtime draft',
          created_at: '2026-04-04T10:00:00.000Z',
          updated_at: '2026-04-04T10:00:00.000Z',
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
      },
    });
    expect(fetchMock).toHaveBeenCalledWith(
      expect.stringContaining('/api/documents'),
      expect.objectContaining({
        method: 'POST',
        body: JSON.stringify({ title: 'Realtime draft' }),
      }),
    );
  });
});
