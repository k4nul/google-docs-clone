import { describe, expect, it } from 'vitest';

import { isSafeEditorLinkHref, normalizeEditorLinkHref } from './linkSafety';

describe('editor link safety', () => {
  it('accepts explicit http, https, and mailto links', () => {
    expect(isSafeEditorLinkHref('https://example.test/docs')).toBe(true);
    expect(isSafeEditorLinkHref('http://localhost:4000/docs')).toBe(true);
    expect(isSafeEditorLinkHref('mailto:owner@example.test')).toBe(true);
  });

  it('normalizes surrounding whitespace from accepted links', () => {
    expect(normalizeEditorLinkHref('  https://example.test/docs  ')).toBe(
      'https://example.test/docs',
    );
  });

  it('rejects scriptable, file, relative, and malformed links', () => {
    expect(isSafeEditorLinkHref('javascript:alert(1)')).toBe(false);
    expect(isSafeEditorLinkHref('data:text/html,<p>test</p>')).toBe(false);
    expect(isSafeEditorLinkHref('file:///etc/passwd')).toBe(false);
    expect(isSafeEditorLinkHref('/docs/local')).toBe(false);
    expect(isSafeEditorLinkHref('https://')).toBe(false);
  });
});
