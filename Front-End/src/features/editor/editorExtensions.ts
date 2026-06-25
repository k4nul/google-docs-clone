import type { AnyExtension } from '@tiptap/core';
import Collaboration from '@tiptap/extension-collaboration';
import CollaborationCaret from '@tiptap/extension-collaboration-caret';
import Link from '@tiptap/extension-link';
import StarterKit from '@tiptap/starter-kit';

import type { CollaborationConnection } from '@/lib/collab/connection';
import type { CollaborationUser } from '@/shared/types/collaboration';

import { isSafeEditorLinkHref } from './linkSafety';

const DEFAULT_COLLABORATION_COLOR = '#0f8b8d';
const SAFE_COLLABORATION_COLOR_PATTERN = /^#[0-9a-f]{6}$/i;

export const INITIAL_EDITOR_CONTENT = `
  <h2>Project brief</h2>
  <p>Use this canvas to draft decisions, capture review notes, and align the team before handoff.</p>
  <ul>
    <li>Add the current objective and owner.</li>
    <li>Keep open questions visible for collaborators.</li>
    <li>Export the document when the review is ready to share.</li>
  </ul>
`;

export function normalizeCollaborationColor(value?: string | null) {
  const color = value?.trim();

  return color && SAFE_COLLABORATION_COLOR_PATTERN.test(color)
    ? color
    : DEFAULT_COLLABORATION_COLOR;
}

function renderCaret(user: Record<string, string>) {
  const color = normalizeCollaborationColor(user.color);
  const cursor = document.createElement('span');
  cursor.className = 'collaboration-caret';
  cursor.style.setProperty('--caret-color', color);

  const label = document.createElement('span');
  label.className = 'collaboration-caret__label';
  label.style.setProperty('--caret-color', color);
  label.textContent = user.name ?? 'Anonymous';

  cursor.append(label);

  return cursor;
}

function renderSelection(user: Record<string, string>) {
  const color = normalizeCollaborationColor(user.color);

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
      isAllowedUri: isSafeEditorLinkHref,
      linkOnPaste: true,
      openOnClick: false,
      protocols: ['http', 'https', 'mailto'],
      shouldAutoLink: isSafeEditorLinkHref,
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
