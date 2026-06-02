import type { JSONContent } from '@tiptap/core';

import { importDocxToHtml } from './docxImport';

export type EditorImportResult =
  | {
      content: JSONContent;
      kind: 'json';
      notice: string;
    }
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

  if (fileName.endsWith('.json')) {
    const fileText = await file.text();

    return {
      content: JSON.parse(fileText) as JSONContent,
      kind: 'json',
      notice: `Imported JSON file: ${file.name}`,
    };
  }

  if (fileName.endsWith('.docx')) {
    const content = await importDocxToHtml(await file.arrayBuffer());

    return {
      content: content.html,
      kind: 'docx',
      notice: `Imported DOCX file: ${file.name}`,
    };
  }

  return {
    kind: 'unsupported',
    notice: 'Unsupported file type. Choose a JSON or DOCX file.',
  };
}
