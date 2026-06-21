import { beforeEach, describe, expect, it, vi } from 'vitest';

const docxMock = vi.hoisted(() => {
  const Document = vi.fn(function Document(options: unknown) {
    return { kind: 'document', options };
  });
  const Paragraph = vi.fn(function Paragraph(options: unknown) {
    return { kind: 'paragraph', options };
  });
  const TextRun = vi.fn(function TextRun(options: unknown) {
    return { kind: 'text-run', options };
  });
  const Packer = {
    toBlob: vi.fn(async () => new Blob(['docx'], { type: 'application/docx' })),
  };

  return {
    Document,
    HeadingLevel: {
      HEADING_1: 'heading-1',
      HEADING_2: 'heading-2',
      HEADING_3: 'heading-3',
      HEADING_4: 'heading-4',
      HEADING_5: 'heading-5',
      HEADING_6: 'heading-6',
    },
    Packer,
    Paragraph,
    TextRun,
  };
});

vi.mock('docx', () => docxMock);

import { createDocxExportBlob } from './documentExport';

describe('document export helpers', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('maps common HTML blocks to DOCX paragraphs', async () => {
    const blob = await createDocxExportBlob(
      '<h1>Title</h1><p><strong>Bold</strong> text</p><blockquote>Quote</blockquote><ol><li>Step</li></ol><ul><li>Point</li></ul>',
    );

    expect(docxMock.Paragraph).toHaveBeenCalledWith(
      expect.objectContaining({ heading: 'heading-1' }),
    );
    expect(docxMock.TextRun).toHaveBeenCalledWith(
      expect.objectContaining({ bold: true, text: 'Bold' }),
    );
    expect(docxMock.Paragraph).toHaveBeenCalledWith(
      expect.objectContaining({ indent: { left: 720 } }),
    );
    expect(docxMock.Paragraph).toHaveBeenCalledWith(
      expect.objectContaining({
        numbering: { level: 0, reference: 'numbered-list' },
      }),
    );
    expect(docxMock.Paragraph).toHaveBeenCalledWith(
      expect.objectContaining({ bullet: { level: 0 } }),
    );
    expect(docxMock.Packer.toBlob).toHaveBeenCalledWith(
      expect.objectContaining({ kind: 'document' }),
    );
    expect(blob.type).toBe('application/docx');
  });

  it('preserves nested inline formatting and line breaks', async () => {
    await createDocxExportBlob(
      '<p><strong><em><u>Nested</u></em></strong><br>Next line</p>',
    );

    expect(docxMock.TextRun).toHaveBeenCalledWith(
      expect.objectContaining({
        bold: true,
        italics: true,
        text: 'Nested',
        underline: {},
      }),
    );
    expect(docxMock.TextRun).toHaveBeenCalledWith(
      expect.objectContaining({ break: 1, text: '' }),
    );
    expect(docxMock.TextRun).toHaveBeenCalledWith(
      expect.objectContaining({ text: 'Next line' }),
    );
  });

  it('creates a blank DOCX paragraph for empty HTML', async () => {
    await createDocxExportBlob('   ');

    expect(docxMock.Paragraph).toHaveBeenCalledWith('');
  });

  it('keeps top-level text nodes and unknown block text as paragraphs', async () => {
    await createDocxExportBlob(
      'Loose text<section><span>Fallback block</span></section>',
    );

    expect(docxMock.TextRun).toHaveBeenCalledWith(
      expect.objectContaining({ text: 'Loose text' }),
    );
    expect(docxMock.TextRun).toHaveBeenCalledWith(
      expect.objectContaining({ text: 'Fallback block' }),
    );
  });

  it('ignores non-list children inside list containers', async () => {
    await createDocxExportBlob(
      '<ul><li>First point</li><div>Ignored block</div><li>Second point</li></ul>',
    );

    expect(docxMock.TextRun).toHaveBeenCalledWith(
      expect.objectContaining({ text: 'First point' }),
    );
    expect(docxMock.TextRun).toHaveBeenCalledWith(
      expect.objectContaining({ text: 'Second point' }),
    );
    expect(docxMock.TextRun).not.toHaveBeenCalledWith(
      expect.objectContaining({ text: 'Ignored block' }),
    );
  });
});
