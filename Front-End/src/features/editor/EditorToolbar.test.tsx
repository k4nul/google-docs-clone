import { cleanup, fireEvent, render, screen } from '@testing-library/react';
import type { Editor } from '@tiptap/react';
import { afterEach, describe, expect, it, vi } from 'vitest';

import { EditorToolbar } from './EditorToolbar';

function createCommandChain() {
  const chain = {
    extendMarkRange: vi.fn(),
    focus: vi.fn(),
    redo: vi.fn(),
    run: vi.fn(),
    setLink: vi.fn(),
    toggleBold: vi.fn(),
    toggleBulletList: vi.fn(),
    toggleItalic: vi.fn(),
    undo: vi.fn(),
    unsetLink: vi.fn(),
  };

  chain.extendMarkRange.mockReturnValue(chain);
  chain.focus.mockReturnValue(chain);
  chain.redo.mockReturnValue(chain);
  chain.run.mockReturnValue(true);
  chain.setLink.mockReturnValue(chain);
  chain.toggleBold.mockReturnValue(chain);
  chain.toggleBulletList.mockReturnValue(chain);
  chain.toggleItalic.mockReturnValue(chain);
  chain.undo.mockReturnValue(chain);
  chain.unsetLink.mockReturnValue(chain);

  return chain;
}

function createEditor({
  active = {},
  canRedo = true,
  canUndo = true,
  linkHref,
}: {
  active?: Record<string, boolean>;
  canRedo?: boolean;
  canUndo?: boolean;
  linkHref?: string;
} = {}) {
  const chain = createCommandChain();
  const can = {
    redo: vi.fn(() => canRedo),
    undo: vi.fn(() => canUndo),
  };
  const chainMock = vi.fn(() => chain);
  const getAttributesMock = vi.fn((name: string) =>
    name === 'link' ? { href: linkHref } : {},
  );
  const isActiveMock = vi.fn((name: string) => active[name] ?? false);

  return {
    can,
    chain,
    chainMock,
    editor: {
      can: vi.fn(() => can),
      chain: chainMock,
      getAttributes: getAttributesMock,
      isActive: isActiveMock,
    } as unknown as Editor,
    getAttributesMock,
    isActiveMock,
  };
}

function toolbarButton(name: string) {
  return screen.getByRole('button', { name });
}

describe('EditorToolbar', () => {
  afterEach(() => {
    cleanup();
    vi.restoreAllMocks();
  });

  it('disables editing commands when no editor is available', () => {
    render(<EditorToolbar editor={null} />);

    expect(
      screen.getByRole('toolbar', { name: /editor toolbar/i }),
    ).toBeInTheDocument();
    expect(toolbarButton('Toggle bold')).toBeDisabled();
    expect(toolbarButton('Toggle italic')).toBeDisabled();
    expect(toolbarButton('Toggle bullet list')).toBeDisabled();
    expect(toolbarButton('Edit link')).toBeDisabled();
    expect(toolbarButton('Undo')).toBeDisabled();
    expect(toolbarButton('Redo')).toBeDisabled();
  });

  it('reflects the active editor formatting state', () => {
    const { editor, isActiveMock } = createEditor({
      active: {
        bold: true,
        bulletList: true,
        italic: false,
        link: true,
      },
    });

    render(<EditorToolbar editor={editor} />);

    expect(toolbarButton('Toggle bold')).toHaveAttribute(
      'aria-pressed',
      'true',
    );
    expect(toolbarButton('Toggle italic')).toHaveAttribute(
      'aria-pressed',
      'false',
    );
    expect(toolbarButton('Toggle bullet list')).toHaveAttribute(
      'aria-pressed',
      'true',
    );
    expect(toolbarButton('Edit link')).toHaveAttribute('aria-pressed', 'true');
    expect(isActiveMock).toHaveBeenCalledWith('bold');
    expect(isActiveMock).toHaveBeenCalledWith('italic');
    expect(isActiveMock).toHaveBeenCalledWith('bulletList');
    expect(isActiveMock).toHaveBeenCalledWith('link');
  });

  it('runs formatting and history commands through focused editor chains', () => {
    const { chain, chainMock, editor } = createEditor();

    render(<EditorToolbar editor={editor} />);

    fireEvent.click(toolbarButton('Toggle bold'));
    fireEvent.click(toolbarButton('Toggle italic'));
    fireEvent.click(toolbarButton('Toggle bullet list'));
    fireEvent.click(toolbarButton('Undo'));
    fireEvent.click(toolbarButton('Redo'));

    expect(chainMock).toHaveBeenCalledTimes(5);
    expect(chain.focus).toHaveBeenCalledTimes(5);
    expect(chain.toggleBold).toHaveBeenCalledTimes(1);
    expect(chain.toggleItalic).toHaveBeenCalledTimes(1);
    expect(chain.toggleBulletList).toHaveBeenCalledTimes(1);
    expect(chain.undo).toHaveBeenCalledTimes(1);
    expect(chain.redo).toHaveBeenCalledTimes(1);
    expect(chain.run).toHaveBeenCalledTimes(5);
  });

  it('uses undo and redo capability checks for history button state', () => {
    const { can, editor } = createEditor({
      canRedo: true,
      canUndo: false,
    });

    render(<EditorToolbar editor={editor} />);

    expect(toolbarButton('Undo')).toBeDisabled();
    expect(toolbarButton('Redo')).not.toBeDisabled();
    expect(can.undo).toHaveBeenCalledTimes(1);
    expect(can.redo).toHaveBeenCalledTimes(1);
  });

  it('leaves the editor unchanged when the link prompt is canceled', () => {
    const promptSpy = vi.spyOn(window, 'prompt').mockReturnValue(null);
    const { chain, chainMock, editor, getAttributesMock } = createEditor({
      linkHref: 'https://existing.example',
    });

    render(<EditorToolbar editor={editor} />);

    fireEvent.click(toolbarButton('Edit link'));

    expect(promptSpy).toHaveBeenCalledWith(
      'Enter a link URL',
      'https://existing.example',
    );
    expect(getAttributesMock).toHaveBeenCalledWith('link');
    expect(chainMock).not.toHaveBeenCalled();
    expect(chain.unsetLink).not.toHaveBeenCalled();
    expect(chain.setLink).not.toHaveBeenCalled();
    expect(chain.run).not.toHaveBeenCalled();
  });

  it('removes a link when the prompt is submitted blank', () => {
    vi.spyOn(window, 'prompt').mockReturnValue('   ');
    const { chain, editor } = createEditor();

    render(<EditorToolbar editor={editor} />);

    fireEvent.click(toolbarButton('Edit link'));

    expect(chain.focus).toHaveBeenCalledTimes(1);
    expect(chain.unsetLink).toHaveBeenCalledTimes(1);
    expect(chain.extendMarkRange).not.toHaveBeenCalled();
    expect(chain.setLink).not.toHaveBeenCalled();
    expect(chain.run).toHaveBeenCalledTimes(1);
  });

  it('sets a provided link value through the active link mark range', () => {
    vi.spyOn(window, 'prompt').mockReturnValue('https://next.example');
    const { chain, editor } = createEditor();

    render(<EditorToolbar editor={editor} />);

    fireEvent.click(toolbarButton('Edit link'));

    expect(chain.focus).toHaveBeenCalledTimes(1);
    expect(chain.extendMarkRange).toHaveBeenCalledWith('link');
    expect(chain.setLink).toHaveBeenCalledWith({
      href: 'https://next.example',
    });
    expect(chain.unsetLink).not.toHaveBeenCalled();
    expect(chain.run).toHaveBeenCalledTimes(1);
  });
});
