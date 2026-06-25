const SAFE_EDITOR_LINK_PROTOCOLS = new Set(['http:', 'https:', 'mailto:']);

export function isSafeEditorLinkHref(value?: string | null) {
  const href = value?.trim();

  if (!href) {
    return false;
  }

  try {
    const url = new URL(href);

    if (!SAFE_EDITOR_LINK_PROTOCOLS.has(url.protocol)) {
      return false;
    }

    if (url.protocol === 'mailto:') {
      return url.pathname.length > 0;
    }

    return url.hostname.length > 0;
  } catch {
    return false;
  }
}

export function normalizeEditorLinkHref(value: string) {
  const href = value.trim();

  return isSafeEditorLinkHref(href) ? href : null;
}
