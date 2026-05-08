import type { AnyExtension } from '@tiptap/core';
import Collaboration from '@tiptap/extension-collaboration';
import CollaborationCaret from '@tiptap/extension-collaboration-caret';
import Link from '@tiptap/extension-link';
import StarterKit from '@tiptap/starter-kit';

import type { CollaborationConnection } from '@/lib/collab/connection';
import type { CollaborationUser } from '@/shared/types/collaboration';

export const INITIAL_EDITOR_CONTENT = `
  <h2>Collaborative editor shell</h2>
  <p>This document is ready for Yjs-based collaboration wiring.</p>
  <ul>
    <li>Open the route from the document list.</li>
    <li>Realtime presence uses the current browser origin by default.</li>
    <li>Wire imported HTML through <code>@/lib/import/docxImport.ts</code>.</li>
  </ul>
`;

function renderCaret(user: Record<string, string>) {
  const cursor = document.createElement('span');
  cursor.className = 'collaboration-caret';
  cursor.style.setProperty('--caret-color', user.color ?? '#0f8b8d');

  const label = document.createElement('span');
  label.className = 'collaboration-caret__label';
  label.style.setProperty('--caret-color', user.color ?? '#0f8b8d');
  label.textContent = user.name ?? 'Anonymous';

  cursor.append(label);

  return cursor;
}

function renderSelection(user: Record<string, string>) {
  const color = user.color ?? '#0f8b8d';

  return {
    class: 'collaboration-selection',
    'data-user': user.name ?? 'Anonymous',
    nodeName: 'span',
    style: `background-color: ${color}22;`,
  };
}

export function createEditorExtensions(
  connection: CollaborationConnection | null,
  user: CollaborationUser,
): AnyExtension[] {
  const extensions: AnyExtension[] = [
    StarterKit.configure({
      link: false,
      ...(connection ? { undoRedo: false } : {}),
    }),
    Link.configure({
      autolink: true,
      linkOnPaste: true,
      openOnClick: false,
      HTMLAttributes: {
        rel: 'noopener noreferrer nofollow',
        target: '_blank',
      },
    }),
  ];

  if (!connection) {
    return extensions;
  }

  extensions.push(
    Collaboration.configure({
      document: connection.doc,
      field: 'content',
      provider: connection.provider,
    }),
  );

  if (connection.provider) {
    extensions.push(
      CollaborationCaret.configure({
        provider: connection.provider,
        render: renderCaret,
        selectionRender: renderSelection,
        user,
      }),
    );
  }

  return extensions;
}
