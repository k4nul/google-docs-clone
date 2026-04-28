import { apiPost } from '@/lib/api/httpClient';
import { appEnv } from '@/shared/config/env';

interface BackendDocument {
  id: string;
  title: string;
  created_at: string;
  updated_at: string;
}

interface CreateDocumentResponse {
  document: BackendDocument;
  credentials: {
    access_token: string;
  };
}

function getAdminHeaders() {
  if (!appEnv.apiToken) {
    throw new Error('VITE_API_TOKEN is not configured.');
  }

  return {
    Authorization: `Bearer ${appEnv.apiToken}`,
  };
}

export async function createBackendDocument(title?: string) {
  const response = await apiPost<CreateDocumentResponse>(
    '/documents',
    { title },
    {
      headers: getAdminHeaders(),
    },
  );

  return {
    accessToken: response.credentials.access_token,
    document: {
      id: response.document.id,
      title: response.document.title,
      updatedAt: response.document.updated_at,
    },
  };
}
