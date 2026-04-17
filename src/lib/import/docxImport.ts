import DOMPurify from 'dompurify';
import mammoth from 'mammoth';

import type {
  EditorImportPayload,
  ImportedDocumentContent,
  ImportedDocumentMessage,
} from '@/shared/types/import';

function extractPlainText(html: string) {
  const parser = new DOMParser();
  const document = parser.parseFromString(html, 'text/html');

  return document.body.textContent?.trim() ?? '';
}

function normalizeMessages(messages: readonly ImportedDocumentMessage[]): ImportedDocumentMessage[] {
  return messages.map((message) => ({
    message: message.message,
    type: message.type,
  }));
}

export function sanitizeImportedHtml(html: string) {
  return DOMPurify.sanitize(html, {
    USE_PROFILES: {
      html: true,
    },
  });
}

export async function importDocxToHtml(arrayBuffer: ArrayBuffer): Promise<ImportedDocumentContent> {
  const result = await mammoth.convertToHtml({ arrayBuffer });
  const html = sanitizeImportedHtml(result.value);
  const messages: ImportedDocumentMessage[] = result.messages.map((message) => ({
    message: message.message,
    type: message.type,
  }));

  return {
    html,
    messages,
    plainText: extractPlainText(html),
    source: 'docx',
  };
}

export function createEditorImportPayload(
  content: ImportedDocumentContent,
): EditorImportPayload {
  return {
    html: content.html,
    messages: normalizeMessages(content.messages),
    plainText: content.plainText,
    source: content.source,
  };
}
