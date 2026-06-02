import type { Editor } from '@tiptap/core';
import { EditorContent, useEditor } from '@tiptap/react';
import { useEffect, useMemo, useRef, useState, type FormEvent } from 'react';

import {
  connectCollaborationConnection,
  createCollaborationConnection,
  redactAccessToken,
  scheduleCollaborationConnectionDestroy,
  type CollaborationConnection,
  type ProviderConnectionStatus,
} from '@/lib/collab/connection';
import { createPlaceholderCollaborationUser } from '@/lib/collab/user';
import { appEnv } from '@/shared/config/env';
import { StatusPill } from '@/shared/ui/DesignSystem';

import {
  createEditorExtensions,
  INITIAL_EDITOR_CONTENT,
} from './editorExtensions';
import { EditorToolbar } from './EditorToolbar';

export interface CollaborationSnapshot {
  activeCollaborators: Array<{
    id: number;
    name: string;
    color?: string;
    isTyping: boolean;
    isCurrentUser: boolean;
  }>;
  connectionStatus: ProviderConnectionStatus;
  isCurrentUserTyping: boolean;
  lastSyncedAt: string | null;
}

interface EditorShellProps {
  documentAccessToken?: string | null;
  docId: string;
  documentTitle?: string;
  lastEditedAt?: string | null;
  onEditorReady?: (editor: Editor | null) => void;
  onCollaborationChange?: (snapshot: CollaborationSnapshot) => void;
  onTitleSubmit?: (title: string) => Promise<void> | void;
  realtimeServerUrl?: string | null;
  titleError?: string | null;
  titleStatus?: 'idle' | 'saving';
}

type WebsocketTransportStatus =
  | 'connected'
  | 'connecting'
  | 'disconnected'
  | 'disabled';

interface RealtimeDebugState {
  synced: boolean;
  transport: WebsocketTransportStatus;
  url: string | null;
}

function getConnectionMode(status: ProviderConnectionStatus) {
  if (status === 'local-only') {
    return 'Local-only mode';
  }

  if (status === 'connected') {
    return 'Realtime sync active';
  }

  if (status === 'connecting') {
    return 'Connecting to realtime server';
  }

  if (status === 'reconnecting') {
    return 'Reconnecting to realtime server';
  }

  return 'Realtime server disconnected';
}

function getRealtimeDebugState(
  connection: CollaborationConnection,
): RealtimeDebugState {
  const { provider } = connection;

  if (!provider) {
    return {
      synced: false,
      transport: 'disabled',
      url: null,
    };
  }

  return {
    synced: provider.synced,
    transport: provider.wsconnected
      ? 'connected'
      : provider.wsconnecting
        ? 'connecting'
        : 'disconnected',
    url: redactAccessToken(provider.url),
  };
}

export function EditorShell({
  documentAccessToken = null,
  docId,
  documentTitle = 'Untitled document',
  lastEditedAt = null,
  onEditorReady,
  onCollaborationChange,
  onTitleSubmit,
  realtimeServerUrl = appEnv.wsUrl,
  titleError = null,
  titleStatus = 'idle',
}: EditorShellProps) {
  const [user] = useState(() => createPlaceholderCollaborationUser());
  const [activeCollaborators, setActiveCollaborators] = useState<
    CollaborationSnapshot['activeCollaborators']
  >([]);
  const [isCurrentUserTyping, setIsCurrentUserTyping] = useState(false);
  const [lastSyncedAt, setLastSyncedAt] = useState<string | null>(null);
  const [draftTitleState, setDraftTitleState] = useState({
    sourceTitle: documentTitle,
    value: documentTitle,
  });
  const draftTitle =
    draftTitleState.sourceTitle === documentTitle
      ? draftTitleState.value
      : documentTitle;
  const typingTimeoutRef = useRef<number | null>(null);
  const connection = useMemo(
    () =>
      createCollaborationConnection({
        accessToken: documentAccessToken,
        roomId: docId,
        serverUrl: realtimeServerUrl,
      }),
    [docId, documentAccessToken, realtimeServerUrl],
  );
  const [connectionStatus, setConnectionStatus] =
    useState<ProviderConnectionStatus>(
      connection.provider ? 'connecting' : 'local-only',
    );
  const [realtimeDebug, setRealtimeDebug] = useState(() =>
    getRealtimeDebugState(connection),
  );

  const editor = useEditor(
    {
      content: INITIAL_EDITOR_CONTENT,
      editorProps: {
        attributes: {
          class: 'editor-surface',
        },
      },
      extensions: createEditorExtensions(connection, user),
    },
    [connection.roomId, connection.provider, user.id],
  );

  async function handleTitleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();

    if (!onTitleSubmit || titleStatus === 'saving') {
      return;
    }

    await onTitleSubmit(draftTitle);
  }

  useEffect(() => {
    const { provider } = connection;

    if (!provider) {
      return undefined;
    }

    const refreshRealtimeDebug = () => {
      setRealtimeDebug(getRealtimeDebugState(connection));
    };

    const unsubscribeStatus = provider.onStatusChange((status) => {
      setConnectionStatus(status);
      refreshRealtimeDebug();
    });

    const updateCollaborators = () => {
      const collaborators = Array.from(
        provider.awareness.getStates().entries(),
      ).map(([clientId, state]) => {
        const userState = state.user as
          | { name?: string; color?: string; id?: string }
          | undefined;
        return {
          id: clientId,
          name: userState?.name ?? 'Anonymous',
          ...(userState?.color ? { color: userState.color } : {}),
          isTyping: false,
          isCurrentUser: userState?.id === user.id,
        };
      });

      setActiveCollaborators(collaborators);
      refreshRealtimeDebug();
    };

    provider.awareness.setLocalState({
      user,
      client: {
        id: user.id,
        kind: 'editor',
      },
    });
    updateCollaborators();
    provider.awareness.on('change', updateCollaborators);

    return () => {
      unsubscribeStatus();
      provider.awareness.off('change', updateCollaborators);
    };
  }, [connection, user]);

  useEffect(() => {
    connectCollaborationConnection(connection);

    return () => {
      if (typingTimeoutRef.current !== null) {
        window.clearTimeout(typingTimeoutRef.current);
        typingTimeoutRef.current = null;
      }
      scheduleCollaborationConnectionDestroy(connection);
    };
  }, [connection]);

  useEffect(() => {
    if (!editor || !connection.provider) {
      return;
    }

    const markTypingStopped = () => {
      setIsCurrentUserTyping(false);
    };

    const handleUpdate = () => {
      setLastSyncedAt(new Date().toLocaleTimeString());
      setIsCurrentUserTyping(true);

      if (typingTimeoutRef.current !== null) {
        window.clearTimeout(typingTimeoutRef.current);
      }

      typingTimeoutRef.current = window.setTimeout(() => {
        typingTimeoutRef.current = null;
        markTypingStopped();
      }, 1200);
    };

    editor.on('update', handleUpdate);

    return () => {
      editor.off('update', handleUpdate);
      markTypingStopped();
      if (typingTimeoutRef.current !== null) {
        window.clearTimeout(typingTimeoutRef.current);
        typingTimeoutRef.current = null;
      }
    };
  }, [connection.provider, editor]);

  useEffect(() => {
    if (!onEditorReady) {
      return;
    }

    onEditorReady(editor ?? null);

    return () => {
      onEditorReady(null);
    };
  }, [editor, onEditorReady]);

  useEffect(() => {
    if (!onCollaborationChange) {
      return;
    }

    onCollaborationChange({
      activeCollaborators,
      connectionStatus,
      isCurrentUserTyping,
      lastSyncedAt,
    });
  }, [
    activeCollaborators,
    connectionStatus,
    isCurrentUserTyping,
    lastSyncedAt,
    onCollaborationChange,
  ]);

  return (
    <section className="editor-shell-card" aria-label="Document editor">
      <header className="editor-topbar">
        <div className="editor-title-group">
          <p className="section-kicker">Document editor</p>
          {onTitleSubmit ? (
            <form className="editor-title-form" onSubmit={handleTitleSubmit}>
              <label className="sr-only" htmlFor="document-title-input">
                Document title
              </label>
              <input
                id="document-title-input"
                value={draftTitle}
                onChange={(event) =>
                  setDraftTitleState({
                    sourceTitle: documentTitle,
                    value: event.target.value,
                  })
                }
              />
              <button
                className="ui-button ui-button--secondary ui-button--sm"
                disabled={
                  titleStatus === 'saving' ||
                  draftTitle.trim() === '' ||
                  draftTitle.trim() === documentTitle
                }
                type="submit"
              >
                {titleStatus === 'saving' ? 'Saving...' : 'Save title'}
              </button>
            </form>
          ) : (
            <h2>{documentTitle}</h2>
          )}
          <p>
            {lastEditedAt
              ? `Last edited ${lastEditedAt}`
              : 'Local editing surface is ready for collaboration.'}
          </p>
          {titleError ? <p className="form-error">{titleError}</p> : null}
        </div>
        <div className="editor-status-group">
          <StatusPill
            tone={connectionStatus === 'connected' ? 'success' : 'warning'}
          >
            {getConnectionMode(connectionStatus)}
          </StatusPill>
          <StatusPill>{isCurrentUserTyping ? 'Typing' : 'Idle'}</StatusPill>
        </div>
      </header>

      <div className="editor-status-strip" aria-label="Realtime status">
        <span className="editor-status-item">
          Room
          <strong>{docId}</strong>
        </span>
        <span className="editor-status-item">
          User
          <strong>{user.name}</strong>
        </span>
        <span className="editor-status-item">
          Transport
          <strong>{realtimeDebug.transport}</strong>
        </span>
        <span className="editor-status-item">
          Peers
          <strong>{activeCollaborators.length}</strong>
        </span>
      </div>

      <div className="sr-only" aria-live="polite">
        <span>{getConnectionMode(connectionStatus)}</span>
      </div>

      <EditorToolbar editor={editor} />

      <div className="editor-workspace">
        <div className="document-canvas">
          {editor ? (
            <EditorContent editor={editor} />
          ) : (
            <div className="editor-loading">Preparing editor surface...</div>
          )}
        </div>
      </div>
    </section>
  );
}
