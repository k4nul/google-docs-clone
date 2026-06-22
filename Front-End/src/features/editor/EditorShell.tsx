import type { Editor } from '@tiptap/core';
import { EditorContent, useEditor } from '@tiptap/react';
import {
  useCallback,
  useEffect,
  useMemo,
  useRef,
  useState,
} from 'react';
import type { FormEvent } from 'react';

import {
  connectCollaborationConnection,
  createCollaborationConnection,
  scheduleCollaborationConnectionDestroy,
} from '@/lib/collab/connection';
import type { ProviderConnectionStatus } from '@/lib/collab/connection';
import { createPlaceholderCollaborationUser } from '@/lib/collab/user';
import { appEnv } from '@/shared/config/env';
import { Badge, Button } from '@/shared/ui';

import {
  createEditorExtensions,
  INITIAL_EDITOR_CONTENT,
} from './editorExtensions';
import { EditorToolbar } from './EditorToolbar';

const AUTOSAVE_SETTLED_DELAY_MS = 500;

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

type AutosaveStatus =
  | 'connecting'
  | 'local-only'
  | 'paused'
  | 'ready'
  | 'saved'
  | 'saving';

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

function getAutosaveStatus(
  status: ProviderConnectionStatus,
  hasProvider: boolean,
): AutosaveStatus {
  if (!hasProvider) {
    return 'local-only';
  }

  if (status === 'connected') {
    return 'ready';
  }

  if (status === 'connecting') {
    return 'connecting';
  }

  return 'paused';
}

function getAutosaveLabel(status: AutosaveStatus) {
  if (status === 'saving') {
    return 'Saving changes';
  }

  if (status === 'saved') {
    return 'Changes saved';
  }

  if (status === 'paused') {
    return 'Autosave paused';
  }

  if (status === 'connecting') {
    return 'Connecting autosave';
  }

  if (status === 'local-only') {
    return 'Local draft only';
  }

  return 'Autosave ready';
}

function getAutosaveTone(status: AutosaveStatus) {
  if (status === 'saved' || status === 'ready') {
    return 'success';
  }

  if (status === 'paused' || status === 'saving' || status === 'connecting') {
    return 'warning';
  }

  return 'neutral';
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
  const initialConnectionStatus: ProviderConnectionStatus = connection.provider
    ? 'connecting'
    : 'local-only';
  const connectionStatusRef = useRef<ProviderConnectionStatus>(
    initialConnectionStatus,
  );
  const [connectionStatus, setConnectionStatus] =
    useState<ProviderConnectionStatus>(initialConnectionStatus);
  const [autosaveStatus, setAutosaveStatus] = useState<AutosaveStatus>(() =>
    getAutosaveStatus(
      connection.provider ? 'connecting' : 'local-only',
      Boolean(connection.provider),
    ),
  );
  const visibleConnectionStatus = connection.provider
    ? connectionStatus
    : 'local-only';
  const visibleAutosaveStatus = connection.provider
    ? autosaveStatus
    : 'local-only';
  const autosaveTimeoutRef = useRef<number | null>(null);
  const clearAutosaveTimeout = useCallback(() => {
    if (autosaveTimeoutRef.current !== null) {
      window.clearTimeout(autosaveTimeoutRef.current);
      autosaveTimeoutRef.current = null;
    }
  }, []);

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
      connectionStatusRef.current = 'local-only';
      return undefined;
    }

    const unsubscribeStatus = provider.onStatusChange((status) => {
      connectionStatusRef.current = status;
      setConnectionStatus(status);
      if (status !== 'connected') {
        clearAutosaveTimeout();
      }
      setAutosaveStatus((currentStatus) => {
        if (
          status === 'connected' &&
          (currentStatus === 'saving' || currentStatus === 'saved')
        ) {
          return currentStatus;
        }

        return getAutosaveStatus(status, true);
      });
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
  }, [clearAutosaveTimeout, connection, user]);

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
      const currentConnectionStatus = connectionStatusRef.current;

      setLastSyncedAt(new Date().toLocaleTimeString());
      setIsCurrentUserTyping(true);
      clearAutosaveTimeout();

      if (currentConnectionStatus === 'connected') {
        setAutosaveStatus('saving');
        autosaveTimeoutRef.current = window.setTimeout(() => {
          autosaveTimeoutRef.current = null;
          setAutosaveStatus('saved');
        }, AUTOSAVE_SETTLED_DELAY_MS);
      } else {
        setAutosaveStatus(getAutosaveStatus(currentConnectionStatus, true));
      }

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
      clearAutosaveTimeout();
      if (typingTimeoutRef.current !== null) {
        window.clearTimeout(typingTimeoutRef.current);
        typingTimeoutRef.current = null;
      }
    };
  }, [clearAutosaveTimeout, connection.provider, editor]);

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
      connectionStatus: visibleConnectionStatus,
      isCurrentUserTyping,
      lastSyncedAt,
    });
  }, [
    activeCollaborators,
    isCurrentUserTyping,
    lastSyncedAt,
    onCollaborationChange,
    visibleConnectionStatus,
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
              <Button
                variant="secondary"
                size="sm"
                disabled={
                  titleStatus === 'saving' ||
                  draftTitle.trim() === '' ||
                  draftTitle.trim() === documentTitle
                }
                type="submit"
              >
                {titleStatus === 'saving' ? 'Saving...' : 'Save title'}
              </Button>
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
          <Badge tone={getAutosaveTone(visibleAutosaveStatus)}>
            {getAutosaveLabel(visibleAutosaveStatus)}
          </Badge>
          <Badge
            tone={
              visibleConnectionStatus === 'connected' ? 'success' : 'warning'
            }
          >
            {getConnectionMode(visibleConnectionStatus)}
          </Badge>
          <Badge>{isCurrentUserTyping ? 'Typing' : 'Idle'}</Badge>
        </div>
      </header>

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
