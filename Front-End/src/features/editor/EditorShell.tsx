import type { Editor } from '@tiptap/core';
import { EditorContent, useEditor } from '@tiptap/react';
import { useEffect, useMemo, useRef, useState } from 'react';

import {
  connectCollaborationConnection,
  createCollaborationConnection,
  scheduleCollaborationConnectionDestroy,
  type CollaborationConnection,
  type ProviderConnectionStatus,
} from '@/lib/collab/connection';
import { createPlaceholderCollaborationUser } from '@/lib/collab/user';
import { appEnv } from '@/shared/config/env';

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
  docId: string;
  onEditorReady?: (editor: Editor | null) => void;
  onCollaborationChange?: (snapshot: CollaborationSnapshot) => void;
  realtimeServerUrl?: string | null;
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
    url: provider.url,
  };
}

export function EditorShell({
  docId,
  onEditorReady,
  onCollaborationChange,
  realtimeServerUrl = appEnv.wsUrl,
}: EditorShellProps) {
  const [user] = useState(() => createPlaceholderCollaborationUser());
  const [activeCollaborators, setActiveCollaborators] = useState<
    CollaborationSnapshot['activeCollaborators']
  >([]);
  const [isCurrentUserTyping, setIsCurrentUserTyping] = useState(false);
  const [lastSyncedAt, setLastSyncedAt] = useState<string | null>(null);
  const typingTimeoutRef = useRef<number | null>(null);
  const connection = useMemo(
    () =>
      createCollaborationConnection({
        roomId: docId,
        serverUrl: realtimeServerUrl,
      }),
    [docId, realtimeServerUrl],
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
    <section className="card editor-shell">
      <div className="pill-row">
        <span className="pill pill--accent">
          {getConnectionMode(connectionStatus)}
        </span>
        <span className="pill">Room: {docId}</span>
        <span className="pill">User: {user.name}</span>
        <span className="pill">
          {isCurrentUserTyping ? 'You are typing' : 'Idle'}
        </span>
      </div>

      <div className="info-list">
        <span>
          Presence provider:{' '}
          <code>{realtimeDebug.url ?? appEnv.wsUrl ?? 'disabled'}</code>
        </span>
        <span>
          Websocket: <code>{realtimeDebug.transport}</code> / synced:{' '}
          <code>{String(realtimeDebug.synced)}</code>
        </span>
        <span>
          Shared fragment: <code>content</code>
        </span>
        <span>
          Peers in room: <code>{activeCollaborators.length}</code>
        </span>
      </div>

      <EditorToolbar editor={editor} />

      <div className="editor-stage">
        {editor ? (
          <EditorContent editor={editor} />
        ) : (
          <div className="editor-loading">Preparing editor surface...</div>
        )}
      </div>
    </section>
  );
}
