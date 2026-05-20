import { apiGet, apiPost } from '@/lib/api/httpClient';
import { appEnv } from '@/shared/config/env';
import type { BackendDocument, DocumentSummary } from '@/shared/types/document';

interface BackendDocumentResponse {
  id: string;
  title: string;
  created_at: string;
  updated_at: string;
}

interface DocumentsResponse {
  documents: BackendDocumentResponse[];
}

interface DocumentResponse {
  document: BackendDocumentResponse;
}

interface CreateDocumentResponse {
  document: BackendDocumentResponse;
}

function mapBackendDocument(document: BackendDocumentResponse): BackendDocument {
  return {
    id: document.id,
    title: document.title,
    createdAt: document.created_at,
    updatedAt: document.updated_at,
  };
}

function getCreateDocumentInit() {
  if (!appEnv.apiToken) {
    return undefined;
  }

  return {
    headers: {
      Authorization: `Bearer ${appEnv.apiToken}`,
    },
  };
}

function getTimestamp(value: string) {
  const timestamp = Date.parse(value);

  return Number.isNaN(timestamp) ? 0 : timestamp;
}

export function documentSummaryFromBackend(document: BackendDocument): DocumentSummary {
  return {
    id: document.id,
    title: document.title,
    summary: `Backend document ${document.id}`,
    createdAt: document.createdAt,
    updatedAt: document.updatedAt,
    collaborators: 0,
    status: 'active',
    source: 'backend',
  };
}

export async function listBackendDocuments() {
  const response = await apiGet<DocumentsResponse>('/documents');

  return response.documents
    .map(mapBackendDocument)
    .map(documentSummaryFromBackend)
    .sort((left, right) => getTimestamp(right.updatedAt) - getTimestamp(left.updatedAt));
}

export async function getBackendDocument(documentId: string) {
  const response = await apiGet<DocumentResponse>(
    `/documents/${encodeURIComponent(documentId)}` as `/${string}`,
  );

  return mapBackendDocument(response.document);
}

export async function createBackendDocument(title?: string) {
  const response = await apiPost<CreateDocumentResponse>(
    '/documents',
    { title },
    getCreateDocumentInit(),
  );

  return {
    document: mapBackendDocument(response.document),
  };
}
