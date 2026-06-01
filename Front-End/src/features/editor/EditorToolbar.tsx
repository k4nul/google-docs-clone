import type { Editor } from '@tiptap/react';

interface EditorToolbarProps {
  editor: Editor | null;
}

function toggleLink(editor: Editor) {
  const previousHref = editor.getAttributes('link').href as string | undefined;
  const nextHref = window.prompt(
    'Enter a link URL',
    previousHref ?? 'https://',
  );

  if (nextHref === null) {
    return;
  }

  if (nextHref.trim() === '') {
    editor.chain().focus().unsetLink().run();
    return;
  }

  editor
    .chain()
    .focus()
    .extendMarkRange('link')
    .setLink({ href: nextHref })
    .run();
}

export function EditorToolbar({ editor }: EditorToolbarProps) {
  return (
    <div aria-label="Editor toolbar" className="editor-toolbar" role="toolbar">
      <div aria-label="Text formatting" className="toolbar-group" role="group">
        <button
          aria-label="Toggle bold"
          aria-pressed={editor?.isActive('bold') ?? false}
          className="toolbar-button"
          disabled={!editor}
          title="Bold"
          onClick={() => editor?.chain().focus().toggleBold().run()}
          type="button"
        >
          B
        </button>
        <button
          aria-label="Toggle italic"
          aria-pressed={editor?.isActive('italic') ?? false}
          className="toolbar-button"
          disabled={!editor}
          title="Italic"
          onClick={() => editor?.chain().focus().toggleItalic().run()}
          type="button"
        >
          I
        </button>
      </div>

      <div aria-label="Insert and structure" className="toolbar-group" role="group">
        <button
          aria-label="Toggle bullet list"
          aria-pressed={editor?.isActive('bulletList') ?? false}
          className="toolbar-button"
          disabled={!editor}
          title="Bullet list"
          onClick={() => editor?.chain().focus().toggleBulletList().run()}
          type="button"
        >
          List
        </button>
        <button
          aria-label="Edit link"
          aria-pressed={editor?.isActive('link') ?? false}
          className="toolbar-button"
          disabled={!editor}
          title="Link"
          onClick={() => {
            if (editor) {
              toggleLink(editor);
            }
          }}
          type="button"
        >
          Link
        </button>
      </div>

      <div aria-label="History" className="toolbar-group" role="group">
        <button
          aria-label="Undo"
          className="toolbar-button"
          disabled={!editor?.can().undo()}
          title="Undo"
          onClick={() => editor?.chain().focus().undo().run()}
          type="button"
        >
          Undo
        </button>
        <button
          aria-label="Redo"
          className="toolbar-button"
          disabled={!editor?.can().redo()}
          title="Redo"
          onClick={() => editor?.chain().focus().redo().run()}
          type="button"
        >
          Redo
        </button>
      </div>
    </div>
  );
}
