import { appEnv } from '@/shared/config/env';

export interface ApiErrorPayload {
  error?: string;
  message?: string;
  owner?: {
    node_id?: string;
    base_url?: string;
  };
}

export class ApiRequestError extends Error {
  readonly payload: ApiErrorPayload | null;
  readonly status: number;

  constructor(status: number, payload: ApiErrorPayload | null) {
    super(payload?.message ?? `API request failed: ${status}`);
    this.name = 'ApiRequestError';
    this.status = status;
    this.payload = payload;
  }
}

function ensureTrailingSlash(value: string) {
  return value.endsWith('/') ? value : `${value}/`;
}

export function buildApiUrl(path: `/${string}`) {
  if (!appEnv.apiBaseUrl) {
    return path;
  }

  return new URL(
    path.slice(1),
    ensureTrailingSlash(appEnv.apiBaseUrl),
  ).toString();
}

async function readErrorPayload(response: Response) {
  if (!response.headers.get('content-type')?.includes('application/json')) {
    return null;
  }

  try {
    return (await response.json()) as ApiErrorPayload;
  } catch {
    return null;
  }
}

async function readJsonResponse<T>(response: Response): Promise<T> {
  if (!response.ok) {
    throw new ApiRequestError(
      response.status,
      await readErrorPayload(response),
    );
  }

  return (await response.json()) as T;
}

export async function apiGet<T>(
  path: `/${string}`,
  init?: RequestInit,
): Promise<T> {
  const response = await fetch(buildApiUrl(path), {
    ...init,
    headers: {
      Accept: 'application/json',
      ...(init?.headers ?? {}),
    },
  });

  return readJsonResponse<T>(response);
}

export async function apiPost<T>(
  path: `/${string}`,
  body?: unknown,
  init?: RequestInit,
): Promise<T> {
  const requestInit: RequestInit = {
    method: 'POST',
    ...init,
    headers: {
      Accept: 'application/json',
      'Content-Type': 'application/json',
      ...(init?.headers ?? {}),
    },
    ...(body === undefined
      ? init?.body !== undefined
        ? { body: init.body }
        : {}
      : { body: JSON.stringify(body) }),
  };

  const response = await fetch(buildApiUrl(path), requestInit);

  return readJsonResponse<T>(response);
}
