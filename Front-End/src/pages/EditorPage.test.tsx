import { act, cleanup, render, screen, waitFor } from '@testing-library/react';
import { MemoryRouter, Route, Routes } from 'react-router-dom';
import { afterEach, describe, expect, it, vi } from 'vitest';

const editorShellMock = vi.hoisted(() => vi.fn());
const fileManagerMock = vi.hoisted(() => vi.fn());

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
  docId: string;
  documentTitle?: string;
  lastEditedAt?: string | null;
  onCollaborationChange?: (snapshot: MockCollaborationSnapshot) => void;
  onEditorReady?: (editor: null) => void;
  realtimeServerUrl?: string | null;
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
}));

import { getBackendDocument } from '@/lib/api/documents';
import { ApiRequestError } from '@/lib/api/httpClient';
import type { BackendDocument } from '@/shared/types/document';

import { EditorPage } from './EditorPage';

const getBackendDocumentMock = vi.mocked(getBackendDocument);

function createBackendDocument(
  overrides: Partial<BackendDocument> = {},
): BackendDocument {
  return {
    id: 'doc-1',
    title: 'Project brief',
    createdAt: 'created-on',
    updatedAt: 'updated-on',
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
  expect(screen.getByText(textContentEquals(expected))).toBeInTheDocument();
}

function latestEditorShellProps() {
  const calls = editorShellMock.mock.calls as Array<[MockEditorShellProps]>;
  const props = calls.at(-1)?.[0];

  if (!props) {
    throw new Error('EditorShell was not rendered.');
  }

  return props;
}

describe('EditorPage', () => {
  afterEach(() => {
    cleanup();
    vi.resetAllMocks();
  });

  it('holds the editor while document metadata is loading', () => {
    getBackendDocumentMock.mockReturnValue(
      new Promise<BackendDocument>(() => undefined),
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
    expectTextContent('Document details: loading');
    expectTextContent(
      'Collaboration: available after document details load',
    );
    expect(
      screen.queryByRole('region', { name: /mock editor shell/i }),
    ).not.toBeInTheDocument();
    expect(getBackendDocumentMock).toHaveBeenCalledWith('doc-1');
  });

  it('loads metadata and starts the editor with realtime document settings', async () => {
    getBackendDocumentMock.mockResolvedValue(
      createBackendDocument({
        id: 'team draft',
        title: 'Team draft',
      }),
    );

    renderEditorPage('/docs/team%20draft');

    expect(
      await screen.findByRole('heading', { name: 'Team draft' }),
    ).toBeInTheDocument();
    expect(getBackendDocumentMock).toHaveBeenCalledWith('team draft');
    expectTextContent('Document details: ready');
    expectTextContent('Created: created-on');
    expectTextContent('Updated: updated-on');
    expectTextContent('Collaboration: ready');

    await waitFor(() => expect(editorShellMock).toHaveBeenCalled());
    expect(latestEditorShellProps()).toMatchObject({
      docId: 'team draft',
      documentTitle: 'Team draft',
      lastEditedAt: 'updated-on',
      realtimeServerUrl: 'ws://localhost:3000/ws',
    });
    expect(typeof latestEditorShellProps().onEditorReady).toBe('function');
    expect(typeof latestEditorShellProps().onCollaborationChange).toBe(
      'function',
    );
  });

  it('falls back to a local-only editor when metadata loading fails', async () => {
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
        /protected local mode while document metadata is unavailable\./,
      ),
    ).toBeInTheDocument();
    expectTextContent('Document details: unavailable');
    expectTextContent(
      'Collaboration: available after document details load',
    );

    await waitFor(() => expect(editorShellMock).toHaveBeenCalled());
    expect(latestEditorShellProps()).toMatchObject({
      docId: 'missing-doc',
      documentTitle: 'Untitled document',
      realtimeServerUrl: null,
    });
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
    expectTextContent('Document details: unavailable');
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

    expectTextContent('Connection state: connected');
    expectTextContent('My activity: typing');
    expectTextContent('Last sync event: 10:45 AM');
    expectTextContent('Active users: 2');
    expectTextContent('Ada Lovelace (me) - typing');
    expect(screen.getByText('Grace Hopper')).toBeInTheDocument();
  });
});
