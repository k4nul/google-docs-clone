import {
  act,
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor,
} from '@testing-library/react';
import type { Editor } from '@tiptap/core';
import { afterEach, describe, expect, it, vi } from 'vitest';

type MockConnectionStatus =
  | 'local-only'
  | 'connecting'
  | 'connected'
  | 'reconnecting'
  | 'disconnected';

type AwarenessState = {
  user?: {
    color?: string;
    id?: string;
    name?: string;
  };
};

type MockProvider = {
  awareness: {
    getStates: ReturnType<typeof vi.fn>;
    off: ReturnType<typeof vi.fn>;
    on: ReturnType<typeof vi.fn>;
    setLocalState: ReturnType<typeof vi.fn>;
  };
  emitStatus: (status: MockConnectionStatus) => void;
  onStatusChange: ReturnType<typeof vi.fn>;
  setAwarenessStates: (states: Map<number, AwarenessState>) => void;
  synced: boolean;
  unsubscribeStatus: ReturnType<typeof vi.fn>;
  url: string;
  wsconnected: boolean;
  wsconnecting: boolean;
};

type MockConnection = {
  destroyed: boolean;
  destroyTimeout: ReturnType<typeof setTimeout> | null;
  doc: Record<string, never>;
  provider: MockProvider | null;
  roomId: string;
};

type MockEditor = Editor & {
  emitUpdate: () => void;
  off: ReturnType<typeof vi.fn>;
  on: ReturnType<typeof vi.fn>;
};

const collaborationMock = vi.hoisted(() => {
  const connections: MockConnection[] = [];
  const connectCollaborationConnection = vi.fn();
  const scheduleCollaborationConnectionDestroy = vi.fn();

  const createCollaborationConnection = vi.fn(
    ({
      accessToken,
      roomId,
      serverUrl,
    }: {
      accessToken?: string | null;
      roomId: string;
      serverUrl: string | null;
    }) => {
      const normalizedRoomId = roomId.trim() || 'default-room';
      let provider: MockProvider | null = null;

      if (serverUrl) {
        const statusListeners = new Set<
          (status: MockConnectionStatus) => void
        >();
        const awarenessListeners = new Set<() => void>();
        let awarenessStates = new Map<number, AwarenessState>();
        const unsubscribeStatus = vi.fn(() => undefined);
        const normalizedServerUrl = serverUrl.replace(/\/+$/, '');
        const tokenQuery = accessToken
          ? `?access_token=${encodeURIComponent(accessToken)}`
          : '';

        provider = {
          awareness: {
            getStates: vi.fn(() => awarenessStates),
            off: vi.fn((event: string, listener: () => void) => {
              if (event === 'change') {
                awarenessListeners.delete(listener);
              }
            }),
            on: vi.fn((event: string, listener: () => void) => {
              if (event === 'change') {
                awarenessListeners.add(listener);
              }
            }),
            setLocalState: vi.fn((state: AwarenessState) => {
              awarenessStates.set(101, state);
            }),
          },
          emitStatus(status: MockConnectionStatus) {
            if (provider) {
              provider.wsconnected = status === 'connected';
              provider.wsconnecting =
                status === 'connecting' || status === 'reconnecting';
              provider.synced = status === 'connected';
            }

            for (const listener of statusListeners) {
              listener(status);
            }
          },
          onStatusChange: vi.fn(
            (listener: (status: MockConnectionStatus) => void) => {
              statusListeners.add(listener);
              listener('connecting');

              return unsubscribeStatus;
            },
          ),
          setAwarenessStates(states: Map<number, AwarenessState>) {
            awarenessStates = states;

            for (const listener of awarenessListeners) {
              listener();
            }
          },
          synced: false,
          unsubscribeStatus,
          url: `${normalizedServerUrl}/ws/${encodeURIComponent(
            normalizedRoomId,
          )}${tokenQuery}`,
          wsconnected: false,
          wsconnecting: true,
        };
      }

      const connection: MockConnection = {
        destroyed: false,
        destroyTimeout: null,
        doc: {},
        provider,
        roomId: normalizedRoomId,
      };

      connections.push(connection);

      return connection;
    },
  );

  const redactAccessToken = vi.fn((url: string) =>
    url.replace(/([?&]access_token=)[^&]+/i, '$1%5Bredacted%5D'),
  );

  return {
    connectCollaborationConnection,
    connections,
    createCollaborationConnection,
    redactAccessToken,
    scheduleCollaborationConnectionDestroy,
  };
});

const editorExtensionMock = vi.hoisted(() => ({
  createEditorExtensions: vi.fn(() => []),
}));

const placeholderUserMock = vi.hoisted(() => ({
  createPlaceholderCollaborationUser: vi.fn(() => ({
    color: '#0f8b8d',
    id: 'current-user',
    name: 'Atlas',
  })),
}));

const tiptapMock = vi.hoisted(() => ({
  useEditor: vi.fn(),
}));

vi.mock('@tiptap/react', () => ({
  EditorContent: ({ editor }: { editor: Editor | null }) => (
    <div data-testid="editor-content">
      {editor ? 'Editor content ready' : 'Editor unavailable'}
    </div>
  ),
  useEditor: tiptapMock.useEditor,
}));

vi.mock('@/lib/collab/connection', () => ({
  connectCollaborationConnection:
    collaborationMock.connectCollaborationConnection,
  createCollaborationConnection:
    collaborationMock.createCollaborationConnection,
  redactAccessToken: collaborationMock.redactAccessToken,
  scheduleCollaborationConnectionDestroy:
    collaborationMock.scheduleCollaborationConnectionDestroy,
}));

vi.mock('@/lib/collab/user', () => ({
  createPlaceholderCollaborationUser:
    placeholderUserMock.createPlaceholderCollaborationUser,
}));

vi.mock('./editorExtensions', () => ({
  INITIAL_EDITOR_CONTENT: '<p>Initial editor content</p>',
  createEditorExtensions: editorExtensionMock.createEditorExtensions,
}));

vi.mock('./EditorToolbar', () => ({
  EditorToolbar: ({ editor }: { editor: Editor | null }) => (
    <div role="toolbar" aria-label="Editor toolbar">
      {editor ? 'Toolbar ready' : 'Toolbar unavailable'}
    </div>
  ),
}));

import { EditorShell } from './EditorShell';

function createMockEditor(): MockEditor {
  const updateHandlers = new Set<() => void>();
  const editor = {
    emitUpdate() {
      for (const handler of updateHandlers) {
        handler();
      }
    },
    off: vi.fn((event: string, handler: () => void) => {
      if (event === 'update') {
        updateHandlers.delete(handler);
      }
    }),
    on: vi.fn((event: string, handler: () => void) => {
      if (event === 'update') {
        updateHandlers.add(handler);
      }
    }),
  } as unknown as MockEditor;

  return editor;
}

function latestConnection() {
  const connection = collaborationMock.connections.at(-1);

  if (!connection) {
    throw new Error('Expected a collaboration connection to be created.');
  }

  return connection;
}

function renderEditorShell(
  props: Partial<React.ComponentProps<typeof EditorShell>> = {},
) {
  const editor = createMockEditor();
  tiptapMock.useEditor.mockReturnValue(editor);

  const view = render(
    <EditorShell
      documentAccessToken="doc-token"
      docId="team-room"
      realtimeServerUrl="ws://localhost:4000"
      {...props}
    />,
  );

  return {
    editor,
    ...view,
  };
}

describe('EditorShell', () => {
  afterEach(() => {
    cleanup();
    vi.useRealTimers();
    collaborationMock.connections.length = 0;
    collaborationMock.connectCollaborationConnection.mockClear();
    collaborationMock.createCollaborationConnection.mockClear();
    collaborationMock.redactAccessToken.mockClear();
    collaborationMock.scheduleCollaborationConnectionDestroy.mockClear();
    editorExtensionMock.createEditorExtensions.mockClear();
    placeholderUserMock.createPlaceholderCollaborationUser.mockClear();
    tiptapMock.useEditor.mockReset();
  });

  it('starts realtime collaboration and reports provider state changes', async () => {
    const onCollaborationChange = vi.fn();

    renderEditorShell({ onCollaborationChange });

    const connection = latestConnection();
    const provider = connection.provider;

    expect(provider).not.toBeNull();
    expect(collaborationMock.createCollaborationConnection).toHaveBeenCalledWith(
      {
        accessToken: 'doc-token',
        roomId: 'team-room',
        serverUrl: 'ws://localhost:4000',
      },
    );
    expect(collaborationMock.connectCollaborationConnection).toHaveBeenCalledWith(
      connection,
    );
    expect(provider?.awareness.setLocalState).toHaveBeenCalledWith({
      client: {
        id: 'current-user',
        kind: 'editor',
      },
      user: {
        color: '#0f8b8d',
        id: 'current-user',
        name: 'Atlas',
      },
    });

    expect(screen.getByText('Connecting autosave')).toBeInTheDocument();
    expect(
      screen.getAllByText('Connecting to realtime server'),
    ).not.toHaveLength(0);
    expect(screen.queryByLabelText('Realtime status')).not.toBeInTheDocument();
    expect(screen.queryByText('team-room')).not.toBeInTheDocument();
    expect(screen.queryByText('Atlas')).not.toBeInTheDocument();

    await waitFor(() =>
      expect(onCollaborationChange).toHaveBeenLastCalledWith({
        activeCollaborators: [
          {
            color: '#0f8b8d',
            id: 101,
            isCurrentUser: true,
            isTyping: false,
            name: 'Atlas',
          },
        ],
        connectionStatus: 'connecting',
        isCurrentUserTyping: false,
        lastSyncedAt: null,
      }),
    );

    act(() => {
      provider?.emitStatus('connected');
      provider?.setAwarenessStates(
        new Map([
          [
            101,
            {
              user: {
                color: '#0f8b8d',
                id: 'current-user',
                name: 'Atlas',
              },
            },
          ],
          [
            202,
            {
              user: {
                color: '#0ea5e9',
                id: 'remote-user',
                name: 'Grace Hopper',
              },
            },
          ],
          [303, {}],
        ]),
      );
    });

    await waitFor(() =>
      expect(screen.getAllByText('Realtime sync active')).not.toHaveLength(0),
    );
    expect(screen.getByText('Autosave ready')).toBeInTheDocument();
    expect(screen.queryByText('connected')).not.toBeInTheDocument();
    expect(screen.queryByText('3')).not.toBeInTheDocument();
    await waitFor(() =>
      expect(onCollaborationChange).toHaveBeenLastCalledWith({
        activeCollaborators: [
          {
            color: '#0f8b8d',
            id: 101,
            isCurrentUser: true,
            isTyping: false,
            name: 'Atlas',
          },
          {
            color: '#0ea5e9',
            id: 202,
            isCurrentUser: false,
            isTyping: false,
            name: 'Grace Hopper',
          },
          {
            id: 303,
            isCurrentUser: false,
            isTyping: false,
            name: 'Anonymous',
          },
        ],
        connectionStatus: 'connected',
        isCurrentUserTyping: false,
        lastSyncedAt: null,
      }),
    );
  });

  it('reports local-only collaboration when no realtime server is configured', async () => {
    const onCollaborationChange = vi.fn();

    renderEditorShell({
      onCollaborationChange,
      realtimeServerUrl: null,
    });

    const connection = latestConnection();

    expect(connection.provider).toBeNull();
    expect(collaborationMock.createCollaborationConnection).toHaveBeenCalledWith(
      {
        accessToken: 'doc-token',
        roomId: 'team-room',
        serverUrl: null,
      },
    );
    expect(collaborationMock.connectCollaborationConnection).toHaveBeenCalledWith(
      connection,
    );
    expect(screen.getAllByText('Local-only mode')).not.toHaveLength(0);
    expect(screen.getByText('Local draft only')).toBeInTheDocument();
    expect(screen.queryByText('disabled')).not.toBeInTheDocument();
    expect(screen.queryByText('0')).not.toBeInTheDocument();

    await waitFor(() =>
      expect(onCollaborationChange).toHaveBeenLastCalledWith({
        activeCollaborators: [],
        connectionStatus: 'local-only',
        isCurrentUserTyping: false,
        lastSyncedAt: null,
      }),
    );
  });

  it('submits edited document titles and disables unchanged submissions', () => {
    const onTitleSubmit = vi.fn();

    renderEditorShell({
      documentTitle: 'Project brief',
      onTitleSubmit,
    });

    const titleInput = screen.getByLabelText('Document title');
    const saveButton = screen.getByRole('button', { name: 'Save title' });

    expect(titleInput).toHaveValue('Project brief');
    expect(saveButton).toBeDisabled();

    fireEvent.change(titleInput, {
      target: {
        value: 'Project plan',
      },
    });

    expect(saveButton).not.toBeDisabled();

    fireEvent.click(saveButton);

    expect(onTitleSubmit).toHaveBeenCalledWith('Project plan');
  });

  it('keeps blank and saving title states from submitting through the disabled action', () => {
    const onTitleSubmit = vi.fn();
    const { rerender } = renderEditorShell({
      documentTitle: 'Project brief',
      onTitleSubmit,
    });

    const titleInput = screen.getByLabelText('Document title');
    const saveButton = screen.getByRole('button', { name: 'Save title' });

    fireEvent.change(titleInput, {
      target: {
        value: '   ',
      },
    });
    fireEvent.click(saveButton);

    expect(saveButton).toBeDisabled();
    expect(onTitleSubmit).not.toHaveBeenCalled();

    fireEvent.change(titleInput, {
      target: {
        value: 'Project plan',
      },
    });
    rerender(
      <EditorShell
        documentAccessToken="doc-token"
        docId="team-room"
        documentTitle="Project brief"
        onTitleSubmit={onTitleSubmit}
        realtimeServerUrl="ws://localhost:4000"
        titleStatus="saving"
      />,
    );

    const savingButton = screen.getByRole('button', { name: 'Saving...' });

    fireEvent.click(savingButton);

    expect(savingButton).toBeDisabled();
    expect(onTitleSubmit).not.toHaveBeenCalled();
  });

  it('shows the upstream title when document metadata changes during a local edit', () => {
    const onTitleSubmit = vi.fn();
    const { rerender } = renderEditorShell({
      documentTitle: 'Project brief',
      onTitleSubmit,
    });

    const titleInput = screen.getByLabelText('Document title');

    fireEvent.change(titleInput, {
      target: {
        value: 'Unsaved local title',
      },
    });

    expect(titleInput).toHaveValue('Unsaved local title');

    rerender(
      <EditorShell
        documentAccessToken="doc-token"
        docId="team-room"
        documentTitle="Published title"
        onTitleSubmit={onTitleSubmit}
        realtimeServerUrl="ws://localhost:4000"
      />,
    );

    expect(screen.getByLabelText('Document title')).toHaveValue(
      'Published title',
    );
    expect(screen.getByRole('button', { name: 'Save title' })).toBeDisabled();
    expect(onTitleSubmit).not.toHaveBeenCalled();
  });

  it('marks the current user as typing after editor updates and returns to idle', async () => {
    vi.useFakeTimers();
    const onCollaborationChange = vi.fn();
    const { editor } = renderEditorShell({ onCollaborationChange });
    const provider = latestConnection().provider;

    act(() => {
      provider?.emitStatus('connected');
    });

    expect(editor.on).toHaveBeenCalledWith('update', expect.any(Function));
    expect(screen.getByText('Autosave ready')).toBeInTheDocument();

    act(() => {
      editor.emitUpdate();
    });

    expect(screen.getByText('Typing')).toBeInTheDocument();
    expect(screen.getByText('Saving changes')).toBeInTheDocument();
    expect(onCollaborationChange).toHaveBeenLastCalledWith(
      expect.objectContaining({
        isCurrentUserTyping: true,
        lastSyncedAt: expect.any(String),
      }),
    );

    act(() => {
      vi.advanceTimersByTime(500);
    });

    expect(screen.getByText('Changes saved')).toBeInTheDocument();

    act(() => {
      vi.advanceTimersByTime(700);
    });

    expect(screen.getByText('Idle')).toBeInTheDocument();
    expect(onCollaborationChange).toHaveBeenLastCalledWith(
      expect.objectContaining({
        isCurrentUserTyping: false,
      }),
    );
  });

  it('surfaces autosave pause when realtime disconnects', () => {
    renderEditorShell();
    const provider = latestConnection().provider;

    act(() => {
      provider?.emitStatus('reconnecting');
    });

    expect(screen.getByText('Autosave paused')).toBeInTheDocument();
  });

  it('unsubscribes collaboration and editor listeners on unmount', async () => {
    const onEditorReady = vi.fn();
    const { editor, unmount } = renderEditorShell({ onEditorReady });
    const connection = latestConnection();
    const provider = connection.provider;

    await waitFor(() => expect(onEditorReady).toHaveBeenCalledWith(editor));
    await waitFor(() =>
      expect(editor.on).toHaveBeenCalledWith('update', expect.any(Function)),
    );

    unmount();

    expect(onEditorReady).toHaveBeenLastCalledWith(null);
    expect(provider?.unsubscribeStatus).toHaveBeenCalledTimes(1);
    expect(provider?.awareness.off).toHaveBeenCalledWith(
      'change',
      expect.any(Function),
    );
    expect(editor.off).toHaveBeenCalledWith('update', expect.any(Function));
    expect(
      collaborationMock.scheduleCollaborationConnectionDestroy,
    ).toHaveBeenCalledWith(connection);
  });
});
