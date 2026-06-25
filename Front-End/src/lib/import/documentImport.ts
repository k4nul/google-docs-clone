import { importDocxToHtml } from './docxImport';

export const MAX_DOCX_IMPORT_BYTES = 10 * 1024 * 1024;

export type EditorImportResult =
  | {
      content: string;
      kind: 'docx';
      notice: string;
    }
  | {
      kind: 'unsupported';
      notice: string;
    };

export async function readEditorImportFile(
  file: File,
): Promise<EditorImportResult> {
  const fileName = file.name.toLowerCase();

  if (fileName.endsWith('.docx')) {
    if (file.size > MAX_DOCX_IMPORT_BYTES) {
      return {
        kind: 'unsupported',
        notice: 'DOCX file is too large. Choose a file under 10 MB.',
      };
    }

    const content = await importDocxToHtml(await file.arrayBuffer());

    return {
      content: content.html,
      kind: 'docx',
      notice: `Imported DOCX file: ${file.name}`,
    };
  }

  return {
    kind: 'unsupported',
    notice: 'Unsupported file type. Choose a DOCX file.',
  };
}
