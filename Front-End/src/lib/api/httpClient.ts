import { appEnv } from '@/shared/config/env';

function ensureTrailingSlash(value: string) {
  return value.endsWith('/') ? value : `${value}/`;
}

export function buildApiUrl(path: `/${string}`) {
  if (!appEnv.apiBaseUrl) {
    return path;
  }

  return new URL(path.slice(1), ensureTrailingSlash(appEnv.apiBaseUrl)).toString();
}

export async function apiGet<T>(path: `/${string}`, init?: RequestInit): Promise<T> {
  const response = await fetch(buildApiUrl(path), {
    ...init,
    headers: {
      Accept: 'application/json',
      ...(init?.headers ?? {}),
    },
  });

  if (!response.ok) {
    throw new Error(`API request failed: ${response.status}`);
  }

  return (await response.json()) as T;
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
      ? (init?.body !== undefined ? { body: init.body } : {})
      : { body: JSON.stringify(body) }),
  };

  const response = await fetch(buildApiUrl(path), requestInit);

  if (!response.ok) {
    throw new Error(`API request failed: ${response.status}`);
  }

  return (await response.json()) as T;
}
