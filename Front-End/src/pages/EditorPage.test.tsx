import {
  act,
  cleanup,
  fireEvent,
  render,
  screen,
  within,
  waitFor,
} from '@testing-library/react';
import { MemoryRouter, Route, Routes } from 'react-router-dom';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

const editorShellMock = vi.hoisted(() => vi.fn());
const fileManagerMock = vi.hoisted(() => vi.fn());
const getStoredDocumentAccessTokenMock = vi.hoisted(() => vi.fn());
const storeDocumentAccessTokenMock = vi.hoisted(() => vi.fn());
const updateBackendDocumentTitleMock = vi.hoisted(() => vi.fn());

type MockCollaborationSnapshot = {
  activeCollaborators: Array<{
    id: number;
    name: string;
    color?: string;
    isTyping: boolean;
    isCurrentUser: boolean;
  }>;
  connectionStatus:
    | 'local-only'
    | 'connecting'
    | 'connected'
    | 'reconnecting'
    | 'disconnected';
  isCurrentUserTyping: boolean;
  lastSyncedAt: string | null;
};

type MockEditorShellProps = {
  documentAccessToken?: string | null;
  docId: string;
  documentTitle?: string;
  lastEditedAt?: string | null;
  onCollaborationChange?: (snapshot: MockCollaborationSnapshot) => void;
  onEditorReady?: (editor: unknown) => void;
  onTitleSubmit?: (title: string) => Promise<void> | void;
  realtimeServerUrl?: string | null;
  titleError?: string | null;
  titleStatus?: 'idle' | 'saving';
};

type MockFileManagerProps = {
  docId: string;
  editor: unknown;
  onNotice: (message: string) => void;
};

vi.mock('@/features/editor/EditorShell', () => ({
  EditorShell: (props: MockEditorShellProps) => {
    editorShellMock(props);

    return <section aria-label="Mock editor shell">Mock editor shell</section>;
  },
}));

vi.mock('@/features/util/FileManager', () => ({
  FileManager: (props: MockFileManagerProps) => {
    fileManagerMock(props);

    return <button type="button">Mock file manager</button>;
  },
}));

vi.mock('@/lib/api/documents', () => ({
  getBackendDocument: vi.fn(),
  getStoredDocumentAccessToken: getStoredDocumentAccessTokenMock,
  storeDocumentAccessToken: storeDocumentAccessTokenMock,
  updateBackendDocumentTitle: updateBackendDocumentTitleMock,
}));

vi.mock('@/shared/config/env', () => ({
  appEnv: {
    apiBaseUrl: 'http://localhost:4000/api',
    apiToken: 'dev-admin-token',
    wsUrl: 'ws://localhost:3000/ws',
  },
}));

import {
  getBackendDocument,
  updateBackendDocumentTitle as updateBackendDocumentTitleApi,
} from '@/lib/api/documents';
import { ApiRequestError } from '@/lib/api/httpClient';
import type { BackendDocument } from '@/shared/types/document';

import { EditorPage } from './EditorPage';

const getBackendDocumentMock = vi.mocked(getBackendDocument);
const updateBackendDocumentTitleMocked = vi.mocked(
  updateBackendDocumentTitleApi,
);

function createBackendDocument(
  overrides: Partial<BackendDocument> = {},
): BackendDocument {
  return {
    id: 'doc-1',
    title: 'Project brief',
    createdAt: 'created-on',
    updatedAt: 'updated-on',
    hidePreview: false,
    ...overrides,
  };
}

function renderEditorPage(path = '/docs/doc-1') {
  render(
    <MemoryRouter initialEntries={[path]}>
      <Routes>
        <Route element={<div>Home route</div>} path="/" />
        <Route element={<EditorPage />} path="/docs/:docId" />
      </Routes>
    </MemoryRouter>,
  );
}

function textContentEquals(expected: string) {
  return (_content: string, element: Element | null) =>
    element?.textContent?.replace(/\s+/g, ' ').trim() === expected;
}

function expectTextContent(expected: string) {
  expect(
    within(screen.getByRole('complementary', { name: /document details/i }))
      .getByText(textContentEquals(expected)),
  ).toBeInTheDocument();
}

function latestEditorShellProps() {
  const calls = editorShellMock.mock.calls as Array<[MockEditorShellProps]>;
  const props = calls.at(-1)?.[0];

  if (!props) {
    throw new Error('EditorShell was not rendered.');
  }

  return props;
}

function latestFileManagerProps() {
  const calls = fileManagerMock.mock.calls as Array<[MockFileManagerProps]>;
  const props = calls.at(-1)?.[0];

  if (!props) {
    throw new Error('FileManager was not rendered.');
  }

  return props;
}

describe('EditorPage', () => {
  beforeEach(() => {
    getStoredDocumentAccessTokenMock.mockReturnValue('stored-doc-token');
  });

  afterEach(() => {
    cleanup();
    vi.resetAllMocks();
  });

  it('holds the editor while document metadata is loading', () => {
    getBackendDocumentMock.mockReturnValue(
      new Promise<BackendDocument>(() => {
        // Keep metadata loading so the loading UI remains visible.
      }),
    );

    renderEditorPage('/docs/doc-1');

    expect(
      screen.getByRole('heading', { name: 'Untitled document' }),
    ).toBeInTheDocument();
    expect(screen.getByText('Loading document metadata')).toBeInTheDocument();
    expect(
      screen.getByText(
        /Document metadata is loading before the editor joins the realtime workspace\./,
      ),
    ).toBeInTheDocument();
    expectTextContent('Open status: loading');
    expectTextContent('Collaboration starts after access opens');
    expect(
      screen.queryByRole('region', { name: /mock editor shell/i }),
    ).not.toBeInTheDocument();
    expect(getBackendDocumentMock).toHaveBeenCalledWith(
      'doc-1',
      'stored-doc-token',
    );
  });

  it('prompts for a document credential before loading metadata', async () => {
    getStoredDocumentAccessTokenMock.mockReturnValue(null);

    renderEditorPage('/docs/protected-doc');

    expect(
      await screen.findByRole('heading', { name: /credential required/i }),
    ).toBeInTheDocument();
    expectTextContent('Open status: credential required');
    expect(getBackendDocumentMock).not.toHaveBeenCalled();
    expect(
      screen.queryByRole('region', { name: /mock editor shell/i }),
    ).not.toBeInTheDocument();
  });

  it('keeps credential-required documents closed until a non-empty token is submitted', async () => {
    getStoredDocumentAccessTokenMock.mockReturnValue(null);

    renderEditorPage('/docs/protected-doc');

    expect(
      await screen.findByRole('heading', { name: /credential required/i }),
    ).toBeInTheDocument();

    fireEvent.change(screen.getByLabelText(/access token/i), {
      target: { value: '   ' },
    });
    fireEvent.click(screen.getByRole('button', { name: /unlock document/i }));

    expect(
      screen.getByText('Enter the document access token to continue.'),
    ).toBeInTheDocument();
    expect(storeDocumentAccessTokenMock).not.toHaveBeenCalled();
    expect(getBackendDocumentMock).not.toHaveBeenCalled();
    expect(editorShellMock).not.toHaveBeenCalled();
  });

  it('retries metadata loading with a submitted credential after an access failure', async () => {
    getBackendDocumentMock
      .mockRejectedValueOnce(new Error('expired credential'))
      .mockResolvedValueOnce(
        createBackendDocument({
          id: 'retry-doc',
          title: 'Recovered plan',
        }),
      );

    renderEditorPage('/docs/retry-doc');

    expect(
      await screen.findByRole('heading', {
        name: 'Document metadata unavailable',
      }),
    ).toBeInTheDocument();
    expect(screen.getByText('expired credential')).toBeInTheDocument();

    fireEvent.change(screen.getByLabelText(/access token/i), {
      target: { value: ' refreshed-token ' },
    });
    fireEvent.click(screen.getByRole('button', { name: /try credential/i }));

    expect(storeDocumentAccessTokenMock).toHaveBeenCalledWith(
      'retry-doc',
      'refreshed-token',
    );
    expect(
      await screen.findByRole('heading', { name: 'Recovered plan' }),
    ).toBeInTheDocument();
    expect(getBackendDocumentMock).toHaveBeenNthCalledWith(
      1,
      'retry-doc',
      'stored-doc-token',
    );
    expect(getBackendDocumentMock).toHaveBeenNthCalledWith(
      2,
      'retry-doc',
      'refreshed-token',
    );
    expect(latestEditorShellProps()).toMatchObject({
      documentAccessToken: 'refreshed-token',
      docId: 'retry-doc',
      documentTitle: 'Recovered plan',
    });
  });

  it('loads metadata and starts the editor with realtime document settings', async () => {
    const updatedAt = '2026-06-20T10:45:00.000Z';

    getBackendDocumentMock.mockResolvedValue(
      createBackendDocument({
        createdAt: '2026-06-20T09:30:00.000Z',
        id: 'team draft',
        updatedAt,
        title: 'Team draft',
      }),
    );

    renderEditorPage('/docs/team%20draft');

    expect(
      await screen.findByRole('heading', { name: 'Team draft' }),
    ).toBeInTheDocument();
    expect(getBackendDocumentMock).toHaveBeenCalledWith(
      'team draft',
      'stored-doc-token',
    );
    expectTextContent('Open status: ready');
    expectTextContent('Collaboration is ready');

    await waitFor(() => expect(editorShellMock).toHaveBeenCalled());
    expect(latestEditorShellProps()).toMatchObject({
      documentAccessToken: 'stored-doc-token',
      docId: 'team draft',
      documentTitle: 'Team draft',
      lastEditedAt: new Date(updatedAt).toLocaleString(),
      realtimeServerUrl: 'ws://localhost:3000/ws',
    });
    expect(typeof latestEditorShellProps().onEditorReady).toBe('function');
    expect(typeof latestEditorShellProps().onCollaborationChange).toBe(
      'function',
    );
  });

  it('passes the ready editor to file actions and surfaces file notices', async () => {
    getBackendDocumentMock.mockResolvedValue(
      createBackendDocument({
        id: 'team draft',
        title: 'Team draft',
      }),
    );
    const readyEditor = {
      getHTML: vi.fn(() => '<p>Ready draft</p>'),
    };

    renderEditorPage('/docs/team%20draft');

    expect(
      await screen.findByRole('region', { name: /mock editor shell/i }),
    ).toBeInTheDocument();
    expect(latestFileManagerProps()).toMatchObject({
      docId: 'team draft',
      editor: null,
    });

    await act(async () => {
      latestEditorShellProps().onEditorReady?.(readyEditor);
    });

    await waitFor(() => {
      expect(latestFileManagerProps().editor).toBe(readyEditor);
    });
    expect(latestFileManagerProps().docId).toBe('team draft');

    await act(async () => {
      latestEditorShellProps().onEditorReady?.(null);
    });

    await waitFor(() => {
      expect(latestFileManagerProps()).toMatchObject({
        docId: 'team draft',
        editor: null,
      });
    });

    await act(async () => {
      latestFileManagerProps().onNotice(
        'The current document was exported as DOCX.',
      );
    });

    expect(
      screen.getByRole('heading', { name: 'File status' }),
    ).toBeInTheDocument();
    expect(
      screen.getByText('The current document was exported as DOCX.'),
    ).toBeInTheDocument();
  });

  it('blocks the editor when metadata loading fails', async () => {
    getBackendDocumentMock.mockRejectedValue(new Error('offline'));

    renderEditorPage('/docs/missing-doc');

    expect(
      await screen.findByRole('heading', {
        name: 'Document metadata unavailable',
      }),
    ).toBeInTheDocument();
    expect(screen.getByText('offline')).toBeInTheDocument();
    expect(
      screen.getByText(
        /Document metadata could not be loaded with the current credential\./,
      ),
    ).toBeInTheDocument();
    expectTextContent('Open status: unavailable');
    expectTextContent('Collaboration starts after access opens');

    expect(editorShellMock).not.toHaveBeenCalled();
  });

  it('surfaces owner handoff details from metadata API errors', async () => {
    getBackendDocumentMock.mockRejectedValue(
      new ApiRequestError(409, {
        message: 'Document owned elsewhere',
        owner: {
          base_url: 'http://node-b',
          node_id: 'node-b',
        },
      }),
    );

    renderEditorPage('/docs/owned-doc');

    expect(
      await screen.findByText(
        'Document owned elsewhere Owner: node-b (http://node-b)',
      ),
    ).toBeInTheDocument();
    expectTextContent('Open status: unavailable');
    expect(editorShellMock).not.toHaveBeenCalled();
  });

  it('reflects collaboration snapshots from the editor shell side panel', async () => {
    getBackendDocumentMock.mockResolvedValue(
      createBackendDocument({ title: 'Live plan' }),
    );

    renderEditorPage('/docs/live-plan');

    expect(
      await screen.findByRole('region', { name: /mock editor shell/i }),
    ).toBeInTheDocument();

    await act(async () => {
      latestEditorShellProps().onCollaborationChange?.({
        activeCollaborators: [
          {
            color: '#f97316',
            id: 1,
            isCurrentUser: true,
            isTyping: true,
            name: 'Ada Lovelace',
          },
          {
            color: '#0ea5e9',
            id: 2,
            isCurrentUser: false,
            isTyping: false,
            name: 'Grace Hopper',
          },
        ],
        connectionStatus: 'connected',
        isCurrentUserTyping: true,
        lastSyncedAt: '10:45 AM',
      });
    });

    expectTextContent('Connection connected');
    expectTextContent('You are typing');
    expectTextContent('Last saved 10:45 AM');
    expectTextContent('Active users 2');
    expectTextContent('Ada Lovelace (me) - typing');
    expect(screen.getByText('Grace Hopper')).toBeInTheDocument();
  });

  it('renames the loaded document through the protected document API', async () => {
    getBackendDocumentMock.mockResolvedValue(
      createBackendDocument({ title: 'Live plan' }),
    );
    updateBackendDocumentTitleMocked.mockResolvedValue(
      createBackendDocument({
        title: 'Renamed plan',
        updatedAt: 'updated-after-rename',
      }),
    );

    renderEditorPage('/docs/live-plan');

    expect(
      await screen.findByRole('region', { name: /mock editor shell/i }),
    ).toBeInTheDocument();

    await act(async () => {
      await latestEditorShellProps().onTitleSubmit?.('Renamed plan');
    });

    expect(updateBackendDocumentTitleMocked).toHaveBeenCalledWith(
      'live-plan',
      'Renamed plan',
      'stored-doc-token',
    );
    expect(
      await screen.findByRole('heading', { name: 'Renamed plan' }),
    ).toBeInTheDocument();
  });

  it('keeps the loaded document title and surfaces title rename failures', async () => {
    getBackendDocumentMock.mockResolvedValue(
      createBackendDocument({ title: 'Live plan' }),
    );
    updateBackendDocumentTitleMocked.mockRejectedValue(
      new Error('Rename denied'),
    );

    renderEditorPage('/docs/live-plan');

    expect(
      await screen.findByRole('region', { name: /mock editor shell/i }),
    ).toBeInTheDocument();

    await act(async () => {
      await latestEditorShellProps().onTitleSubmit?.('Restricted plan');
    });

    expect(updateBackendDocumentTitleMocked).toHaveBeenCalledWith(
      'live-plan',
      'Restricted plan',
      'stored-doc-token',
    );
    expect(
      screen.getByRole('heading', { name: 'Live plan' }),
    ).toBeInTheDocument();
    expect(latestEditorShellProps()).toMatchObject({
      documentTitle: 'Live plan',
      titleError: 'Rename denied',
      titleStatus: 'idle',
    });
  });
});
