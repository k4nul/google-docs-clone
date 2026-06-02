import { Document, HeadingLevel, Packer, Paragraph, TextRun } from 'docx';
import type { IParagraphOptions, IRunOptions } from 'docx';

interface InlineRun {
  text: string;
  bold?: boolean;
  italics?: boolean;
  underline?: boolean;
  break?: number;
}

const headingLevelByTag: Partial<
  Record<string, (typeof HeadingLevel)[keyof typeof HeadingLevel]>
> = {
  H1: HeadingLevel.HEADING_1,
  H2: HeadingLevel.HEADING_2,
  H3: HeadingLevel.HEADING_3,
  H4: HeadingLevel.HEADING_4,
  H5: HeadingLevel.HEADING_5,
  H6: HeadingLevel.HEADING_6,
};

function toTextRun(run: InlineRun) {
  const options: IRunOptions = {
    text: run.text,
    ...(run.bold ? { bold: true } : {}),
    ...(run.italics ? { italics: true } : {}),
    ...(run.underline ? { underline: {} } : {}),
    ...(typeof run.break === 'number' ? { break: run.break } : {}),
  };

  return new TextRun(options);
}

function getInlineRuns(node: Node): InlineRun[] {
  const runs: InlineRun[] = [];

  if (node.nodeType === Node.TEXT_NODE) {
    const text = node.textContent ?? '';
    if (text.trim().length > 0) {
      runs.push({ text });
    }
    return runs;
  }

  if (!(node instanceof HTMLElement)) {
    return runs;
  }

  const isBold = node.tagName === 'STRONG' || node.tagName === 'B';
  const isItalic = node.tagName === 'EM' || node.tagName === 'I';
  const isUnderline = node.tagName === 'U';

  if (node.tagName === 'BR') {
    runs.push({ text: '', break: 1 });
    return runs;
  }

  for (const child of Array.from(node.childNodes)) {
    const childRuns = getInlineRuns(child);
    for (const run of childRuns) {
      runs.push({
        text: run.text,
        ...(isBold || run.bold ? { bold: true } : {}),
        ...(isItalic || run.italics ? { italics: true } : {}),
        ...(isUnderline || run.underline ? { underline: true } : {}),
        ...(typeof run.break === 'number' ? { break: run.break } : {}),
      });
    }
  }

  return runs;
}

function createParagraph(
  textRuns: InlineRun[],
  options?: Partial<IParagraphOptions>,
) {
  if (textRuns.length === 0) {
    return new Paragraph({ text: '', ...(options ?? {}) });
  }

  return new Paragraph({
    children: textRuns.map(toTextRun),
    ...(options ?? {}),
  });
}

export function htmlToDocxParagraphs(html: string): Paragraph[] {
  const parser = new DOMParser();
  const doc = parser.parseFromString(html, 'text/html');
  const blocks: Paragraph[] = [];

  for (const node of Array.from(doc.body.childNodes)) {
    if (node.nodeType === Node.TEXT_NODE) {
      const text = node.textContent?.trim();
      if (text) {
        blocks.push(createParagraph([{ text }]));
      }
      continue;
    }

    if (!(node instanceof HTMLElement)) {
      continue;
    }

    const heading = headingLevelByTag[node.tagName];
    if (heading) {
      blocks.push(createParagraph(getInlineRuns(node), { heading }));
      continue;
    }

    if (node.tagName === 'P') {
      blocks.push(createParagraph(getInlineRuns(node)));
      continue;
    }

    if (node.tagName === 'BLOCKQUOTE') {
      blocks.push(
        createParagraph(getInlineRuns(node), { indent: { left: 720 } }),
      );
      continue;
    }

    if (node.tagName === 'UL' || node.tagName === 'OL') {
      for (const li of Array.from(node.children)) {
        if (li.tagName !== 'LI') {
          continue;
        }

        blocks.push(
          createParagraph(getInlineRuns(li), {
            ...(node.tagName === 'UL' ? { bullet: { level: 0 } } : {}),
            ...(node.tagName === 'OL'
              ? { numbering: { reference: 'numbered-list', level: 0 } }
              : {}),
          }),
        );
      }
      continue;
    }

    const text = node.textContent?.trim();
    if (text) {
      blocks.push(createParagraph([{ text }]));
    }
  }

  return blocks.length > 0 ? blocks : [new Paragraph('')];
}

export function createJsonExportBlob(content: unknown) {
  return new Blob([JSON.stringify(content, null, 2)], {
    type: 'application/json;charset=utf-8',
  });
}

export async function createDocxExportBlob(html: string) {
  const doc = new Document({
    sections: [{ children: htmlToDocxParagraphs(html) }],
  });

  return Packer.toBlob(doc);
}
