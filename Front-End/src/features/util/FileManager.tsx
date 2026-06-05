import { useCallback, useRef } from 'react';
import type { ChangeEvent } from 'react';
import type { Editor } from '@tiptap/core';

import { createDocxExportBlob } from '@/lib/export/documentExport';
import { readEditorImportFile } from '@/lib/import/documentImport';
import { Button } from '@/shared/ui';

interface FileManagerProps {
  editor: Editor | null;
  docId: string;
  onNotice: (message: string) => void;
}

function downloadBlob(blob: Blob, fileName: string) {
  const downloadUrl = URL.createObjectURL(blob);
  const link = document.createElement('a');

  link.href = downloadUrl;
  link.download = fileName;
  document.body.appendChild(link);
  link.click();
  link.remove();

  window.setTimeout(() => URL.revokeObjectURL(downloadUrl), 0);
}

export function FileManager({ editor, docId, onNotice }: FileManagerProps) {
  const importInputRef = useRef<HTMLInputElement | null>(null);

  const openImportDialog = useCallback(() => {
    importInputRef.current?.click();
  }, []);

  const exportDocx = useCallback(async () => {
    if (!editor) {
      onNotice('The editor is not ready yet.');
      return;
    }

    try {
      downloadBlob(
        await createDocxExportBlob(editor.getHTML()),
        `${docId}-export.docx`,
      );
      onNotice('The current document was exported as DOCX.');
    } catch {
      onNotice('Unable to export the document as DOCX.');
    }
  }, [docId, editor, onNotice]);

  const handleImportFile = useCallback(
    async (event: ChangeEvent<HTMLInputElement>) => {
      if (!editor) {
        onNotice('The editor is not ready yet.');
        return;
      }

      const selectedFile = event.target.files?.[0];
      if (!selectedFile) {
        onNotice('No file was selected.');
        return;
      }

      try {
        const result = await readEditorImportFile(selectedFile);

        if (result.kind === 'unsupported') {
          onNotice(result.notice);
          return;
        }

        editor.commands.setContent(result.content);
        onNotice(result.notice);
      } catch {
        onNotice('Unable to import the selected file.');
      } finally {
        event.target.value = '';
      }
    },
    [editor, onNotice],
  );

  return (
    <div className="file-actions">
      <Button
        disabled={!editor}
        variant="secondary"
        type="button"
        onClick={openImportDialog}
      >
        Import DOCX
      </Button>
      <Button
        disabled={!editor}
        variant="primary"
        type="button"
        onClick={exportDocx}
      >
        Export DOCX
      </Button>
      <input
        ref={importInputRef}
        accept="application/vnd.openxmlformats-officedocument.wordprocessingml.document,.docx"
        aria-label="Import DOCX file"
        className="visually-hidden-file-input"
        type="file"
        onChange={handleImportFile}
      />
    </div>
  );
}
