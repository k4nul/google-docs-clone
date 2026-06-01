import { beforeEach, describe, expect, it, vi } from 'vitest';
import mammoth from 'mammoth';

import type { ImportedDocumentContent } from '@/shared/types/import';

import {
  createEditorImportPayload,
  importDocxToHtml,
  sanitizeImportedHtml,
} from './docxImport';

vi.mock('mammoth', () => ({
  default: {
    convertToHtml: vi.fn(),
  },
}));

const convertToHtmlMock = vi.mocked(mammoth.convertToHtml);

describe('sanitizeImportedHtml', () => {
  beforeEach(() => {
    convertToHtmlMock.mockReset();
  });

  it('removes unsafe markup while preserving safe content', () => {
    const html = sanitizeImportedHtml(
      '<p>Hello</p><a href="javascript:alert(1)">bad link</a><img src="x" onerror="alert(1)"><script>alert("xss")</script>',
    );

    expect(html).toContain('<p>Hello</p>');
    expect(html).not.toContain('<script>');
    expect(html).not.toContain('javascript:');
    expect(html).not.toContain('onerror');
  });

  it('converts DOCX output into sanitized editor content', async () => {
    const arrayBuffer = new ArrayBuffer(8);
    convertToHtmlMock.mockResolvedValue({
      value: '<p>Launch notes<script>alert("xss")</script></p>',
      messages: [
        {
          message: 'Unrecognised paragraph style',
          type: 'warning',
        },
      ],
    });

    await expect(importDocxToHtml(arrayBuffer)).resolves.toEqual({
      html: '<p>Launch notes</p>',
      messages: [
        {
          message: 'Unrecognised paragraph style',
          type: 'warning',
        },
      ],
      plainText: 'Launch notes',
      source: 'docx',
    });
    expect(convertToHtmlMock).toHaveBeenCalledWith({ arrayBuffer });
  });

  it('trims converted plain text while preserving sanitized HTML', async () => {
    convertToHtmlMock.mockResolvedValue({
      value: '  <p>  Draft title  </p>  ',
      messages: [],
    });

    await expect(importDocxToHtml(new ArrayBuffer(0))).resolves.toMatchObject({
      html: '  <p>  Draft title  </p>  ',
      messages: [],
      plainText: 'Draft title',
      source: 'docx',
    });
  });

  it('normalizes editor import payload messages', () => {
    const messages = [
      {
        detail: 'ignored by editor payload',
        message: 'Unsupported image',
        type: 'warning' as const,
      },
      {
        code: 'relationship-missing',
        message: 'Broken relationship',
        type: 'error' as const,
      },
    ];
    const content: ImportedDocumentContent = {
      html: '<p>Imported draft</p>',
      messages,
      plainText: 'Imported draft',
      source: 'docx',
    };

    const payload = createEditorImportPayload(content);

    expect(payload).toEqual({
      html: '<p>Imported draft</p>',
      messages: [
        {
          message: 'Unsupported image',
          type: 'warning',
        },
        {
          message: 'Broken relationship',
          type: 'error',
        },
      ],
      plainText: 'Imported draft',
      source: 'docx',
    });
    expect(payload.messages).not.toBe(content.messages);
  });
});
