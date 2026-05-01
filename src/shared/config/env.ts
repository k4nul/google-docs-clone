function normalizeEnvValue(value: string | undefined) {
  const trimmed = value?.trim();

  if (!trimmed) {
    return null;
  }

  return trimmed.replace(/\/+$/, '');
}

function getBrowserLocation() {
  if (typeof window === 'undefined') {
    return null;
  }

  return window.location;
}

function getRuntimeApiBaseUrl() {
  const location = getBrowserLocation();

  if (!location) {
    return null;
  }

  return `${location.origin}/api`;
}

function getRuntimeWebsocketUrl() {
  const location = getBrowserLocation();

  if (!location) {
    return null;
  }

  const protocol = location.protocol === 'https:' ? 'wss:' : 'ws:';

  return `${protocol}//${location.host}/ws`;
}

export const appEnv = {
  apiBaseUrl: normalizeEnvValue(import.meta.env.VITE_API_BASE_URL) ?? getRuntimeApiBaseUrl(),
  apiToken: normalizeEnvValue(import.meta.env.VITE_API_TOKEN),
  wsUrl: normalizeEnvValue(import.meta.env.VITE_WS_URL) ?? getRuntimeWebsocketUrl(),
} as const;
