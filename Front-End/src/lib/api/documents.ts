import { apiGet, apiPost } from '@/lib/api/httpClient';
import { appEnv } from '@/shared/config/env';
import type { BackendDocument, DocumentSummary } from '@/shared/types/document';

interface BackendDocumentResponse {
  id: string;
  title: string;
  created_at: string;
  updated_at: string;
  collaborator_count?: number | null;
  collaborators?: number | null;
  hide_preview?: boolean | null;
  preview?: string | null;
  preview_hidden?: boolean | null;
  summary?: string | null;
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

function mapBackendDocument(
  document: BackendDocumentResponse,
): BackendDocument {
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

function normalizePreviewText(value: string | null | undefined) {
  const preview = value?.replace(/\s+/g, ' ').trim();

  if (!preview) {
    return null;
  }

  return preview.length > 180 ? `${preview.slice(0, 177)}...` : preview;
}

function getDocumentPreview(document: BackendDocumentResponse) {
  if (document.preview_hidden || document.hide_preview) {
    return 'Preview hidden';
  }

  return (
    normalizePreviewText(document.preview) ??
    normalizePreviewText(document.summary) ??
    'No preview available'
  );
}

function getCollaboratorCount(document: BackendDocumentResponse) {
  return document.collaborators ?? document.collaborator_count ?? 0;
}

export function documentSummaryFromBackend(
  document: BackendDocument,
  rawDocument?: BackendDocumentResponse,
): DocumentSummary {
  return {
    id: document.id,
    title: document.title,
    summary: rawDocument ? getDocumentPreview(rawDocument) : 'No preview available',
    createdAt: document.createdAt,
    updatedAt: document.updatedAt,
    collaborators: rawDocument ? getCollaboratorCount(rawDocument) : 0,
  };
}

export async function listBackendDocuments() {
  const response = await apiGet<DocumentsResponse>('/documents');

  return response.documents
    .map((document) =>
      documentSummaryFromBackend(mapBackendDocument(document), document),
    )
    .sort(
      (left, right) =>
        getTimestamp(right.updatedAt) - getTimestamp(left.updatedAt),
    );
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
