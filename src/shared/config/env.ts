function normalizeEnvValue(value: string | undefined) {
  const trimmed = value?.trim();

  if (!trimmed) {
    return null;
  }

  return trimmed.replace(/\/+$/, '');
}

export const appEnv = {
  apiBaseUrl: normalizeEnvValue(import.meta.env.VITE_API_BASE_URL),
  wsUrl: normalizeEnvValue(import.meta.env.VITE_WS_URL),
} as const;
