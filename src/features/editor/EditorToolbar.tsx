import type { Editor } from '@tiptap/react';

interface EditorToolbarProps {
  editor: Editor | null;
}

function toggleLink(editor: Editor) {
  const previousHref = editor.getAttributes('link').href as string | undefined;
  const nextHref = window.prompt('Enter a link URL', previousHref ?? 'https://');

  if (nextHref === null) {
    return;
  }

  if (nextHref.trim() === '') {
    editor.chain().focus().unsetLink().run();
    return;
  }

  editor.chain().focus().extendMarkRange('link').setLink({ href: nextHref }).run();
}

export function EditorToolbar({ editor }: EditorToolbarProps) {
  return (
    <div aria-label="Editor toolbar" className="editor-toolbar">
      <button
        aria-pressed={editor?.isActive('bold') ?? false}
        className="toolbar-button"
        disabled={!editor}
        onClick={() => editor?.chain().focus().toggleBold().run()}
        type="button"
      >
        Bold
      </button>
      <button
        aria-pressed={editor?.isActive('italic') ?? false}
        className="toolbar-button"
        disabled={!editor}
        onClick={() => editor?.chain().focus().toggleItalic().run()}
        type="button"
      >
        Italic
      </button>
      <button
        aria-pressed={editor?.isActive('bulletList') ?? false}
        className="toolbar-button"
        disabled={!editor}
        onClick={() => editor?.chain().focus().toggleBulletList().run()}
        type="button"
      >
        Bullet list
      </button>
      <button
        aria-pressed={editor?.isActive('link') ?? false}
        className="toolbar-button"
        disabled={!editor}
        onClick={() => {
          if (editor) {
            toggleLink(editor);
          }
        }}
        type="button"
      >
        Link
      </button>
      <button
        className="toolbar-button"
        disabled={!editor?.can().undo()}
        onClick={() => editor?.chain().focus().undo().run()}
        type="button"
      >
        Undo
      </button>
      <button
        className="toolbar-button"
        disabled={!editor?.can().redo()}
        onClick={() => editor?.chain().focus().redo().run()}
        type="button"
      >
        Redo
      </button>
    </div>
  );
}
