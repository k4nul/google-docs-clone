import type { Editor } from '@tiptap/react';

import { Tooltip } from '@/shared/ui';

interface EditorToolbarProps {
  editor: Editor | null;
}

interface ToolbarButtonProps {
  children: string;
  disabled: boolean;
  label: string;
  pressed?: boolean;
  tooltip: string;
  onClick: () => void;
}

function ToolbarButton({
  children,
  disabled,
  label,
  pressed,
  tooltip,
  onClick,
}: ToolbarButtonProps) {
  return (
    <Tooltip content={tooltip}>
      <button
        aria-label={label}
        aria-pressed={pressed}
        className="toolbar-button"
        disabled={disabled}
        title={tooltip}
        onClick={onClick}
        type="button"
      >
        {children}
      </button>
    </Tooltip>
  );
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
        <ToolbarButton
          disabled={!editor}
          label="Toggle bold"
          pressed={editor?.isActive('bold') ?? false}
          tooltip="Bold"
          onClick={() => editor?.chain().focus().toggleBold().run()}
        >
          B
        </ToolbarButton>
        <ToolbarButton
          disabled={!editor}
          label="Toggle italic"
          pressed={editor?.isActive('italic') ?? false}
          tooltip="Italic"
          onClick={() => editor?.chain().focus().toggleItalic().run()}
        >
          I
        </ToolbarButton>
      </div>

      <div
        aria-label="Insert and structure"
        className="toolbar-group"
        role="group"
      >
        <ToolbarButton
          disabled={!editor}
          label="Toggle bullet list"
          pressed={editor?.isActive('bulletList') ?? false}
          tooltip="Bullet list"
          onClick={() => editor?.chain().focus().toggleBulletList().run()}
        >
          List
        </ToolbarButton>
        <ToolbarButton
          disabled={!editor}
          label="Edit link"
          pressed={editor?.isActive('link') ?? false}
          tooltip="Link"
          onClick={() => {
            if (editor) {
              toggleLink(editor);
            }
          }}
        >
          Link
        </ToolbarButton>
      </div>

      <div aria-label="History" className="toolbar-group" role="group">
        <ToolbarButton
          disabled={!editor?.can().undo()}
          label="Undo"
          tooltip="Undo"
          onClick={() => editor?.chain().focus().undo().run()}
        >
          Undo
        </ToolbarButton>
        <ToolbarButton
          disabled={!editor?.can().redo()}
          label="Redo"
          tooltip="Redo"
          onClick={() => editor?.chain().focus().redo().run()}
        >
          Redo
        </ToolbarButton>
      </div>
    </div>
  );
}
