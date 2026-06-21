import {
  act,
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor,
} from '@testing-library/react';
import type { Editor } from '@tiptap/core';
import type { MockInstance } from 'vitest';
import { afterEach, describe, expect, it, vi } from 'vitest';

import { createDocxExportBlob } from '@/lib/export/documentExport';
import { readEditorImportFile } from '@/lib/import/documentImport';

import { FileManager } from './FileManager';

vi.mock('@/lib/export/documentExport', () => ({
  createDocxExportBlob: vi.fn(),
}));

vi.mock('@/lib/import/documentImport', () => ({
  readEditorImportFile: vi.fn(),
}));

const createDocxExportBlobMock = vi.mocked(createDocxExportBlob);
const readEditorImportFileMock = vi.mocked(readEditorImportFile);
const restoreUrlMethods: Array<() => void> = [];

function createEditor(html = '<p>Current draft</p>') {
  return {
    commands: {
      setContent: vi.fn(),
    },
    getHTML: vi.fn(() => html),
  } as unknown as Editor;
}

function stubObjectUrls(objectUrl: string) {
  const createObjectUrlMock = vi.fn(() => objectUrl);
  const revokeObjectUrlMock = vi.fn();

  replaceUrlMethod('createObjectURL', createObjectUrlMock);
  replaceUrlMethod('revokeObjectURL', revokeObjectUrlMock);

  return { createObjectUrlMock, revokeObjectUrlMock };
}

function replaceUrlMethod(
  methodName: 'createObjectURL' | 'revokeObjectURL',
  replacement: typeof URL.createObjectURL | typeof URL.revokeObjectURL,
) {
  const descriptor = Object.getOwnPropertyDescriptor(URL, methodName);

  Object.defineProperty(URL, methodName, {
    configurable: true,
    value: replacement,
    writable: true,
  });

  restoreUrlMethods.push(() => {
    if (descriptor) {
      Object.defineProperty(URL, methodName, descriptor);
      return;
    }

    Reflect.deleteProperty(URL, methodName);
  });
}

function trackFileInputValue(input: HTMLInputElement, initialValue = '') {
  let inputValue = initialValue;

  Object.defineProperty(input, 'value', {
    configurable: true,
    get: () => inputValue,
    set: (value: string) => {
      inputValue = value;
    },
  });
}

function getAppendedAnchor(
  appendChildMock: MockInstance<typeof document.body.appendChild>,
): HTMLAnchorElement {
  const appendedNode = appendChildMock.mock.calls.find(
    ([node]) => node instanceof HTMLAnchorElement,
  )?.[0];

  if (!(appendedNode instanceof HTMLAnchorElement)) {
    throw new Error('Expected a DOCX download link to be appended.');
  }

  return appendedNode;
}

describe('FileManager', () => {
  afterEach(() => {
    cleanup();
    createDocxExportBlobMock.mockReset();
    readEditorImportFileMock.mockReset();
    restoreUrlMethods.splice(0).reverse().forEach((restore) => restore());
    vi.restoreAllMocks();
    vi.useRealTimers();
  });

  it('inserts resolved import content into the editor', async () => {
    const editor = createEditor();
    const onNotice = vi.fn();
    const selectedFile = new File(['docx bytes'], 'draft.docx', {
      type: 'application/vnd.openxmlformats-officedocument.wordprocessingml.document',
    });
    readEditorImportFileMock.mockResolvedValue({
      content: '<p>Imported draft</p>',
      kind: 'docx',
      notice: 'Imported DOCX file: draft.docx',
    });

    render(<FileManager editor={editor} docId="doc-1" onNotice={onNotice} />);

    const importInput = screen.getByLabelText(
      /import docx file/i,
    ) as HTMLInputElement;
    trackFileInputValue(importInput, 'C:\\fakepath\\draft.docx');

    fireEvent.change(importInput, {
      target: {
        files: [selectedFile],
      },
    });

    await waitFor(() => {
      expect(editor.commands.setContent).toHaveBeenCalledWith(
        '<p>Imported draft</p>',
      );
    });
    expect(readEditorImportFileMock).toHaveBeenCalledWith(selectedFile);
    expect(onNotice).toHaveBeenCalledWith('Imported DOCX file: draft.docx');
    expect(importInput.value).toBe('');
  });

  it('exports current editor HTML as a document-specific DOCX download', async () => {
    vi.useFakeTimers();
    const editor = createEditor('<h1>Exported draft</h1>');
    const onNotice = vi.fn();
    const exportedBlob = new Blob(['docx bytes'], {
      type: 'application/vnd.openxmlformats-officedocument.wordprocessingml.document',
    });
    let resolveExport!: (blob: Blob) => void;
    const exportPromise = new Promise<Blob>((resolve) => {
      resolveExport = resolve;
    });
    const { createObjectUrlMock, revokeObjectUrlMock } = stubObjectUrls(
      'blob:http://localhost/exported-docx',
    );
    const clickMock = vi
      .spyOn(HTMLAnchorElement.prototype, 'click')
      .mockImplementation(() => undefined);
    createDocxExportBlobMock.mockReturnValue(exportPromise);

    render(
      <FileManager editor={editor} docId="team-draft" onNotice={onNotice} />,
    );

    const appendChildMock = vi.spyOn(document.body, 'appendChild');

    fireEvent.click(screen.getByRole('button', { name: /export docx/i }));

    await act(async () => {
      resolveExport(exportedBlob);
      await exportPromise;
    });
    expect(createDocxExportBlobMock).toHaveBeenCalledWith(
      '<h1>Exported draft</h1>',
    );
    expect(onNotice).toHaveBeenCalledWith(
      'The current document was exported as DOCX.',
    );
    expect(createObjectUrlMock).toHaveBeenCalledWith(exportedBlob);
    expect(clickMock).toHaveBeenCalledTimes(1);
    const clickedLink = getAppendedAnchor(appendChildMock);

    expect(clickedLink.download).toBe('team-draft-export.docx');
    expect(clickedLink.isConnected).toBe(false);
    expect(revokeObjectUrlMock).not.toHaveBeenCalled();

    act(() => {
      vi.runOnlyPendingTimers();
    });
    expect(revokeObjectUrlMock).toHaveBeenCalledWith(
      'blob:http://localhost/exported-docx',
    );
  });

  it('preserves decoded document ids in exported DOCX filenames', async () => {
    vi.useFakeTimers();
    const editor = createEditor('<p>Filename draft</p>');
    const onNotice = vi.fn();
    const exportedBlob = new Blob(['docx bytes'], {
      type: 'application/vnd.openxmlformats-officedocument.wordprocessingml.document',
    });
    const { createObjectUrlMock, revokeObjectUrlMock } = stubObjectUrls(
      'blob:http://localhost/filename-docx',
    );
    vi.spyOn(HTMLAnchorElement.prototype, 'click').mockImplementation(
      () => undefined,
    );
    createDocxExportBlobMock.mockResolvedValue(exportedBlob);

    render(
      <FileManager
        editor={editor}
        docId="team draft 2026"
        onNotice={onNotice}
      />,
    );

    const appendChildMock = vi.spyOn(document.body, 'appendChild');

    fireEvent.click(screen.getByRole('button', { name: /export docx/i }));

    await act(async () => {
      await Promise.resolve();
    });
    expect(onNotice).toHaveBeenCalledWith(
      'The current document was exported as DOCX.',
    );

    const clickedLink = getAppendedAnchor(appendChildMock);

    expect(createDocxExportBlobMock).toHaveBeenCalledWith(
      '<p>Filename draft</p>',
    );
    expect(createObjectUrlMock).toHaveBeenCalledWith(exportedBlob);
    expect(clickedLink.download).toBe('team draft 2026-export.docx');

    act(() => {
      vi.runOnlyPendingTimers();
    });
    expect(revokeObjectUrlMock).toHaveBeenCalledWith(
      'blob:http://localhost/filename-docx',
    );
  });

  it('opens the hidden DOCX file input from the visible import button', () => {
    const editor = createEditor();
    const onNotice = vi.fn();

    render(<FileManager editor={editor} docId="doc-1" onNotice={onNotice} />);

    const importInput = screen.getByLabelText(/import docx file/i);
    const clickMock = vi
      .spyOn(importInput, 'click')
      .mockImplementation(() => undefined);

    fireEvent.click(screen.getByRole('button', { name: /import docx/i }));

    expect(clickMock).toHaveBeenCalledTimes(1);
  });

  it('reports unsupported imports without changing editor content', async () => {
    const editor = createEditor();
    const onNotice = vi.fn();
    readEditorImportFileMock.mockResolvedValue({
      kind: 'unsupported',
      notice: 'Unsupported file type. Choose a DOCX file.',
    });

    render(<FileManager editor={editor} docId="doc-1" onNotice={onNotice} />);

    const importInput = screen.getByLabelText(
      /import docx file/i,
    ) as HTMLInputElement;
    trackFileInputValue(importInput, 'C:\\fakepath\\notes.txt');

    fireEvent.change(importInput, {
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
        'Unsupported file type. Choose a DOCX file.',
      );
    });
    expect(editor.commands.setContent).not.toHaveBeenCalled();
    expect(importInput.value).toBe('');
  });

  it('reports direct file input changes when the editor is not ready', () => {
    const onNotice = vi.fn();

    render(<FileManager editor={null} docId="doc-1" onNotice={onNotice} />);

    fireEvent.change(screen.getByLabelText(/import docx file/i), {
      target: {
        files: [
          new File(['docx bytes'], 'draft.docx', {
            type: 'application/vnd.openxmlformats-officedocument.wordprocessingml.document',
          }),
        ],
      },
    });

    expect(
      screen.getByRole('button', { name: /import docx/i }),
    ).toBeDisabled();
    expect(
      screen.getByRole('button', { name: /export docx/i }),
    ).toBeDisabled();
    expect(onNotice).toHaveBeenCalledTimes(1);
    expect(onNotice).toHaveBeenCalledWith('The editor is not ready yet.');
    expect(readEditorImportFileMock).not.toHaveBeenCalled();
    expect(createDocxExportBlobMock).not.toHaveBeenCalled();
  });

  it('keeps disabled file actions inert until an editor is ready', () => {
    const onNotice = vi.fn();

    render(<FileManager editor={null} docId="doc-1" onNotice={onNotice} />);

    fireEvent.click(screen.getByRole('button', { name: /import docx/i }));
    fireEvent.click(screen.getByRole('button', { name: /export docx/i }));

    expect(onNotice).not.toHaveBeenCalled();
    expect(readEditorImportFileMock).not.toHaveBeenCalled();
    expect(createDocxExportBlobMock).not.toHaveBeenCalled();
  });

  it('reports empty file selections without reading import content', () => {
    const editor = createEditor();
    const onNotice = vi.fn();

    render(<FileManager editor={editor} docId="doc-1" onNotice={onNotice} />);

    const importInput = screen.getByLabelText(/import docx file/i);

    fireEvent.change(importInput, {
      target: {
        files: [],
      },
    });

    expect(onNotice).toHaveBeenCalledWith('No file was selected.');
    expect(readEditorImportFileMock).not.toHaveBeenCalled();
    expect(editor.commands.setContent).not.toHaveBeenCalled();
  });

  it('reports import failures without changing editor content', async () => {
    const editor = createEditor();
    const onNotice = vi.fn();
    readEditorImportFileMock.mockRejectedValue(new Error('read failed'));

    render(<FileManager editor={editor} docId="doc-1" onNotice={onNotice} />);

    const importInput = screen.getByLabelText(
      /import docx file/i,
    ) as HTMLInputElement;
    trackFileInputValue(importInput, 'C:\\fakepath\\draft.docx');

    fireEvent.change(importInput, {
      target: {
        files: [
          new File(['docx bytes'], 'draft.docx', {
            type: 'application/vnd.openxmlformats-officedocument.wordprocessingml.document',
          }),
        ],
      },
    });

    await waitFor(() => {
      expect(onNotice).toHaveBeenCalledWith(
        'Unable to import the selected file.',
      );
    });
    expect(editor.commands.setContent).not.toHaveBeenCalled();
    expect(importInput.value).toBe('');
  });

  it('reports export failures without starting a download', async () => {
    const editor = createEditor();
    const onNotice = vi.fn();
    const { createObjectUrlMock, revokeObjectUrlMock } = stubObjectUrls(
      'blob:should-not-download',
    );
    const clickMock = vi
      .spyOn(HTMLAnchorElement.prototype, 'click')
      .mockImplementation(() => undefined);
    createDocxExportBlobMock.mockRejectedValue(new Error('pack failed'));

    render(<FileManager editor={editor} docId="doc-1" onNotice={onNotice} />);

    fireEvent.click(screen.getByRole('button', { name: /export docx/i }));

    await waitFor(() => {
      expect(onNotice).toHaveBeenCalledWith(
        'Unable to export the document as DOCX.',
      );
    });
    expect(createObjectUrlMock).not.toHaveBeenCalled();
    expect(revokeObjectUrlMock).not.toHaveBeenCalled();
    expect(clickMock).not.toHaveBeenCalled();
  });
});
