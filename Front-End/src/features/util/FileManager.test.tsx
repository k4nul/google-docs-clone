import { cleanup, fireEvent, render, screen, waitFor } from '@testing-library/react';
import type { Editor } from '@tiptap/core';
import mammoth from 'mammoth/mammoth.browser';
import { afterEach, describe, expect, it, vi } from 'vitest';

import { FileManager } from './FileManager';

vi.mock('mammoth/mammoth.browser', () => ({
  default: {
    convertToHtml: vi.fn(),
  },
}));

const convertToHtmlMock = vi.mocked(mammoth.convertToHtml);

function createEditor() {
  return {
    commands: {
      setContent: vi.fn(),
    },
    getHTML: vi.fn(),
    getJSON: vi.fn(),
  } as unknown as Editor;
}

describe('FileManager', () => {
  afterEach(() => {
    cleanup();
    convertToHtmlMock.mockReset();
  });

  it('sanitizes converted DOCX HTML before inserting it into the editor', async () => {
    const editor = createEditor();
    const onNotice = vi.fn();
    convertToHtmlMock.mockResolvedValue({
      value:
        '<p>Imported draft</p><img src="x" onerror="alert(1)"><script>alert("xss")</script>',
      messages: [],
    });

    render(
      <FileManager editor={editor} docId="doc-1" onNotice={onNotice} />,
    );

    fireEvent.change(screen.getByLabelText(/import json or docx file/i), {
      target: {
        files: [
          new File(['docx bytes'], 'draft.docx', {
            type: 'application/vnd.openxmlformats-officedocument.wordprocessingml.document',
          }),
        ],
      },
    });

    await waitFor(() => {
      expect(editor.commands.setContent).toHaveBeenCalledWith(
        expect.not.stringContaining('<script>'),
      );
    });
    expect(editor.commands.setContent).toHaveBeenCalledWith(
      expect.not.stringContaining('onerror'),
    );
    expect(editor.commands.setContent).toHaveBeenCalledWith(
      expect.stringContaining('<p>Imported draft</p>'),
    );
    expect(onNotice).toHaveBeenCalledWith('Imported DOCX file: draft.docx');
  });
});
