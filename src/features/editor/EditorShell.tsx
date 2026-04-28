import type { Editor } from '@tiptap/core';
import { EditorContent, useEditor } from '@tiptap/react';
import { useEffect, useMemo, useState } from 'react';

import {
  connectCollaborationConnection,
  createCollaborationConnection,
  destroyCollaborationConnection,
} from '@/lib/collab/connection';
import { createPlaceholderCollaborationUser } from '@/lib/collab/user';
import { appEnv } from '@/shared/config/env';

import { createEditorExtensions, INITIAL_EDITOR_CONTENT } from './editorExtensions';
import { EditorToolbar } from './EditorToolbar';

interface EditorShellProps {
  docId: string;
  onEditorReady?: (editor: Editor | null) => void;
}

function getConnectionMode(hasProvider: boolean) {
  if (!hasProvider) {
    return 'Local-only Yjs mode';
  }

  return 'Realtime provider configured';
}

export function EditorShell({ docId, onEditorReady }: EditorShellProps) {
  const [user] = useState(() => createPlaceholderCollaborationUser());
  const connection = useMemo(
    () =>
      createCollaborationConnection({
        roomId: docId,
        serverUrl: appEnv.wsUrl,
      }),
    [docId],
  );

  useEffect(() => {
    connectCollaborationConnection(connection);

    return () => {
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
    if (!onEditorReady) {
      return;
    }

    onEditorReady(editor ?? null);

    return () => {
      onEditorReady(null);
    };
  }, [editor, onEditorReady]);

  return (
    <section className="card editor-shell">
      <div className="pill-row">
        <span className="pill pill--accent">{getConnectionMode(Boolean(connection.provider))}</span>
        <span className="pill">Room: {docId}</span>
        <span className="pill">User: {user.name}</span>
      </div>

      <div className="info-list">
        <span>Presence provider: <code>{appEnv.wsUrl ?? 'disabled'}</code></span>
        <span>Shared fragment: <code>content</code></span>
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
