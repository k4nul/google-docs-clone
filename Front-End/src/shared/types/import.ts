export interface ImportedDocumentMessage {
  type: 'error' | 'warning';
  message: string;
}

export interface ImportedDocumentContent {
  source: 'docx';
  html: string;
  plainText: string;
  messages: ImportedDocumentMessage[];
}

export interface EditorImportPayload {
  source: 'docx';
  html: string;
  plainText: string;
  messages: ImportedDocumentMessage[];
}
