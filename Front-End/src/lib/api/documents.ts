import { apiGet, apiPatch, apiPost } from '@/lib/api/httpClient';
import { appEnv } from '@/shared/config/env';
import type { BackendDocument, DocumentSummary } from '@/shared/types/document';

const DOCUMENT_CREDENTIALS_STORAGE_KEY = 'realtime-docs.document-credentials.v1';

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
  credentials?: DocumentCredentialsResponse | null;
}

interface DocumentCredentialsResponse {
  access_token?: unknown;
}

interface CreateDocumentResult {
  document: BackendDocument;
  credentials: {
    accessToken: string;
  };
}

export class MissingDocumentCredentialError extends Error {
  readonly documentId: string;

  constructor(documentId: string) {
    super('Document access token is required.');
    this.name = 'MissingDocumentCredentialError';
    this.documentId = documentId;
  }
}

function mapBackendDocument(
  document: BackendDocumentResponse,
): BackendDocument {
  return {
    id: document.id,
    title: document.title,
    createdAt: document.created_at,
    updatedAt: document.updated_at,
    hidePreview: Boolean(document.hide_preview ?? document.preview_hidden),
  };
}

function getAdminRequestInit(): RequestInit | undefined {
  if (!appEnv.apiToken) {
    return undefined;
  }

  return {
    headers: {
      Authorization: `Bearer ${appEnv.apiToken}`,
    },
  };
}

function getDocumentRequestInit(accessToken: string): RequestInit {
  return {
    headers: {
      Authorization: `Bearer ${accessToken}`,
    },
  };
}

function getBrowserStorage() {
  if (typeof window === 'undefined') {
    return null;
  }

  try {
    return window.localStorage;
  } catch {
    return null;
  }
}

function readCredentialMap(): Record<string, string> {
  const storage = getBrowserStorage();

  if (!storage) {
    return {};
  }

  let rawCredentials: string | null;

  try {
    rawCredentials = storage.getItem(DOCUMENT_CREDENTIALS_STORAGE_KEY);
  } catch {
    return {};
  }

  if (!rawCredentials) {
    return {};
  }

  try {
    const parsed = JSON.parse(rawCredentials) as unknown;

    if (!parsed || typeof parsed !== 'object' || Array.isArray(parsed)) {
      return {};
    }

    return Object.fromEntries(
      Object.entries(parsed).filter(
        (entry): entry is [string, string] =>
          typeof entry[0] === 'string' && typeof entry[1] === 'string',
      ),
    ) as Record<string, string>;
  } catch {
    return {};
  }
}

function writeCredentialMap(credentials: Record<string, string>) {
  const storage = getBrowserStorage();

  if (!storage) {
    return;
  }

  try {
    storage.setItem(
      DOCUMENT_CREDENTIALS_STORAGE_KEY,
      JSON.stringify(credentials),
    );
  } catch {
    return;
  }
}

export function getStoredDocumentAccessToken(documentId: string): string | null {
  return readCredentialMap()[documentId] ?? null;
}

export function storeDocumentAccessToken(
  documentId: string,
  accessToken: string,
) {
  const token = accessToken.trim();

  if (!token) {
    return;
  }

  writeCredentialMap({
    ...readCredentialMap(),
    [documentId]: token,
  });
}

function requireDocumentAccessToken(
  documentId: string,
  accessToken?: string | null,
) {
  const token = accessToken?.trim() || getStoredDocumentAccessToken(documentId);

  if (!token) {
    throw new MissingDocumentCredentialError(documentId);
  }

  return token;
}

function requireCreatedDocumentAccessToken(response: CreateDocumentResponse) {
  const token =
    typeof response.credentials?.access_token === 'string'
      ? response.credentials.access_token.trim()
      : '';

  if (!token) {
    throw new Error('Document creation response did not include an access token.');
  }

  return token;
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
  const response = await apiGet<DocumentsResponse>(
    '/documents',
    getAdminRequestInit(),
  );

  return response.documents
    .map((document) =>
      documentSummaryFromBackend(mapBackendDocument(document), document),
    )
    .sort(
      (left, right) =>
        getTimestamp(right.updatedAt) - getTimestamp(left.updatedAt),
    );
}

export async function getBackendDocument(
  documentId: string,
  accessToken?: string | null,
) {
  const token = requireDocumentAccessToken(documentId, accessToken);
  const response = await apiGet<DocumentResponse>(
    `/documents/${encodeURIComponent(documentId)}` as `/${string}`,
    getDocumentRequestInit(token),
  );

  return mapBackendDocument(response.document);
}

export async function createBackendDocument(
  title?: string,
): Promise<CreateDocumentResult> {
  const response = await apiPost<CreateDocumentResponse>(
    '/documents',
    { title },
    getAdminRequestInit(),
  );
  const document = mapBackendDocument(response.document);
  const accessToken = requireCreatedDocumentAccessToken(response);
  storeDocumentAccessToken(document.id, accessToken);

  return {
    document,
    credentials: {
      accessToken,
    },
  };
}

export async function updateBackendDocumentTitle(
  documentId: string,
  title: string,
  accessToken?: string | null,
) {
  const token = requireDocumentAccessToken(documentId, accessToken);
  const response = await apiPatch<DocumentResponse>(
    `/documents/${encodeURIComponent(documentId)}` as `/${string}`,
    { title },
    getDocumentRequestInit(token),
  );

  return mapBackendDocument(response.document);
}

export async function updateBackendDocumentSecurity(
  documentId: string,
  settings: { hidePreview: boolean },
  accessToken?: string | null,
) {
  const token = requireDocumentAccessToken(documentId, accessToken);
  const response = await apiPatch<DocumentResponse>(
    `/documents/${encodeURIComponent(documentId)}` as `/${string}`,
    { hide_preview: settings.hidePreview },
    getDocumentRequestInit(token),
  );

  return mapBackendDocument(response.document);
}
