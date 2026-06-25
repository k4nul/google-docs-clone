import mammoth from 'mammoth';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import { MAX_DOCX_IMPORT_BYTES, readEditorImportFile } from './documentImport';

vi.mock('mammoth', () => ({
  default: {
    convertToHtml: vi.fn(),
  },
}));

const convertToHtmlMock = vi.mocked(mammoth.convertToHtml);

describe('readEditorImportFile', () => {
  beforeEach(() => {
    convertToHtmlMock.mockReset();
  });

  it('sanitizes DOCX imports before returning editor content', async () => {
    convertToHtmlMock.mockResolvedValue({
      messages: [],
      value:
        '<p>Imported draft</p><img src="x" onerror="alert(1)"><script>alert("xss")</script>',
    });
    const file = new File(['docx bytes'], 'draft.docx', {
      type: 'application/vnd.openxmlformats-officedocument.wordprocessingml.document',
    });

    await expect(readEditorImportFile(file)).resolves.toEqual({
      content: '<p>Imported draft</p><img src="x">',
      kind: 'docx',
      notice: 'Imported DOCX file: draft.docx',
    });
    expect(convertToHtmlMock).toHaveBeenCalledWith({
      arrayBuffer: expect.any(ArrayBuffer),
    });
  });

  it('returns an unsupported result for unknown file types', async () => {
    const file = new File(['plain text'], 'notes.txt', {
      type: 'text/plain',
    });

    await expect(readEditorImportFile(file)).resolves.toEqual({
      kind: 'unsupported',
      notice: 'Unsupported file type. Choose a DOCX file.',
    });
    expect(convertToHtmlMock).not.toHaveBeenCalled();
  });

  it('rejects oversized DOCX imports before reading file bytes', async () => {
    const arrayBuffer = vi.fn(async () => new ArrayBuffer(0));
    const file = {
      arrayBuffer,
      name: 'large.docx',
      size: MAX_DOCX_IMPORT_BYTES + 1,
    } as unknown as File;

    await expect(readEditorImportFile(file)).resolves.toEqual({
      kind: 'unsupported',
      notice: 'DOCX file is too large. Choose a file under 10 MB.',
    });
    expect(arrayBuffer).not.toHaveBeenCalled();
    expect(convertToHtmlMock).not.toHaveBeenCalled();
  });
});
