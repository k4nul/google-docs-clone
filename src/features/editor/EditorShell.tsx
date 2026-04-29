import type { Editor } from '@tiptap/core';
import { EditorContent, useEditor } from '@tiptap/react';
import { useEffect, useMemo, useRef, useState } from 'react';

import {
  connectCollaborationConnection,
  createCollaborationConnection,
  destroyCollaborationConnection,
  type ProviderConnectionStatus,
} from '@/lib/collab/connection';
import { createPlaceholderCollaborationUser } from '@/lib/collab/user';
import { appEnv } from '@/shared/config/env';

import { createEditorExtensions, INITIAL_EDITOR_CONTENT } from './editorExtensions';
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

export function EditorShell({
  docId,
  onEditorReady,
  onCollaborationChange,
}: EditorShellProps) {
  const [user] = useState(() => createPlaceholderCollaborationUser());
  const [connectionStatus, setConnectionStatus] = useState<ProviderConnectionStatus>(
    appEnv.wsUrl ? 'connecting' : 'local-only',
  );
  const [activeCollaborators, setActiveCollaborators] = useState<CollaborationSnapshot['activeCollaborators']>([]);
  const [isCurrentUserTyping, setIsCurrentUserTyping] = useState(false);
  const [lastSyncedAt, setLastSyncedAt] = useState<string | null>(null);
  const typingTimeoutRef = useRef<number | null>(null);
  const connection = useMemo(
    () =>
      createCollaborationConnection({
        roomId: docId,
        serverUrl: appEnv.wsUrl,
      }),
    [docId],
  );

  useEffect(() => {
    if (!connection.provider) {
      setConnectionStatus('local-only');
      setActiveCollaborators([]);
      return;
    }

    const unsubscribeStatus = connection.provider.onStatusChange(setConnectionStatus);

    const updateCollaborators = () => {
      const collaborators = Array.from(connection.provider?.awareness.getStates().entries() ?? []).map(
        ([clientId, state]) => {
          const userState = state.user as { name?: string; color?: string; id?: string } | undefined;
          return {
            id: clientId,
            name: userState?.name ?? 'Anonymous',
            ...(userState?.color ? { color: userState.color } : {}),
            isTyping: false,
            isCurrentUser: userState?.id === user.id,
          };
        },
      );

      setActiveCollaborators(collaborators);
    };

    connection.provider.awareness.setLocalState({
      user,
      client: {
        id: user.id,
        kind: 'editor',
      },
    });
    updateCollaborators();
    connection.provider.awareness.on('change', updateCollaborators);

    return () => {
      unsubscribeStatus();
      connection.provider?.awareness.off('change', updateCollaborators);
    };
  }, [connection.provider, user]);

  useEffect(() => {
    connectCollaborationConnection(connection);

    return () => {
      if (typingTimeoutRef.current !== null) {
        window.clearTimeout(typingTimeoutRef.current);
      }
      destroyCollaborationConnection(connection);
    };
  }, [connection]);

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
  }, [activeCollaborators, connectionStatus, isCurrentUserTyping, lastSyncedAt, onCollaborationChange]);

  return (
    <section className="card editor-shell">
      <div className="pill-row">
        <span className="pill pill--accent">{getConnectionMode(connectionStatus)}</span>
        <span className="pill">Room: {docId}</span>
        <span className="pill">User: {user.name}</span>
        <span className="pill">{isCurrentUserTyping ? 'You are typing' : 'Idle'}</span>
      </div>

      <div className="info-list">
        <span>Presence provider: <code>{appEnv.wsUrl ?? 'disabled'}</code></span>
        <span>Shared fragment: <code>content</code></span>
        <span>Peers in room: <code>{activeCollaborators.length}</code></span>
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
