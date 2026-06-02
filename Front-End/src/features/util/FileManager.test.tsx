import {
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor,
} from '@testing-library/react';
import type { Editor } from '@tiptap/core';
import { afterEach, describe, expect, it, vi } from 'vitest';

import { readEditorImportFile } from '@/lib/import/documentImport';

import { FileManager } from './FileManager';

vi.mock('@/lib/import/documentImport', () => ({
  readEditorImportFile: vi.fn(),
}));

const readEditorImportFileMock = vi.mocked(readEditorImportFile);

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
    readEditorImportFileMock.mockReset();
  });

  it('inserts resolved import content into the editor', async () => {
    const editor = createEditor();
    const onNotice = vi.fn();
    readEditorImportFileMock.mockResolvedValue({
      content: '<p>Imported draft</p>',
      kind: 'docx',
      notice: 'Imported DOCX file: draft.docx',
    });

    render(<FileManager editor={editor} docId="doc-1" onNotice={onNotice} />);

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
        '<p>Imported draft</p>',
      );
    });
    expect(readEditorImportFileMock).toHaveBeenCalledWith(expect.any(File));
    expect(onNotice).toHaveBeenCalledWith('Imported DOCX file: draft.docx');
  });

  it('reports unsupported imports without changing editor content', async () => {
    const editor = createEditor();
    const onNotice = vi.fn();
    readEditorImportFileMock.mockResolvedValue({
      kind: 'unsupported',
      notice: 'Unsupported file type. Choose a JSON or DOCX file.',
    });

    render(<FileManager editor={editor} docId="doc-1" onNotice={onNotice} />);

    fireEvent.change(screen.getByLabelText(/import json or docx file/i), {
      target: {
        files: [
          new File(['plain text'], 'notes.txt', {
            type: 'text/plain',
          }),
        ],
      },
    });

    await waitFor(() => {
      expect(onNotice).toHaveBeenCalledWith(
        'Unsupported file type. Choose a JSON or DOCX file.',
      );
    });
    expect(editor.commands.setContent).not.toHaveBeenCalled();
  });
});
