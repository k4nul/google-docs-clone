import { describe, expect, it } from 'vitest';

import { sanitizeImportedHtml } from './docxImport';

describe('sanitizeImportedHtml', () => {
  it('removes unsafe markup while preserving safe content', () => {
    const html = sanitizeImportedHtml('<p>Hello</p><script>alert("xss")</script>');

    expect(html).toContain('<p>Hello</p>');
    expect(html).not.toContain('<script>');
  });
});
