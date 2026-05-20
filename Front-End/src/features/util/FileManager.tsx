import { useCallback, useRef } from 'react';
import type { ChangeEvent } from 'react';
import type { Editor, JSONContent } from '@tiptap/core';
import mammoth from 'mammoth/mammoth.browser';
import { Document, Packer, Paragraph, TextRun, HeadingLevel } from 'docx';
import type { IParagraphOptions, IRunOptions } from 'docx';

interface FileManagerProps {
  editor: Editor | null;
  docId: string;
  onNotice: (message: string) => void;
}

interface InlineRun {
  text: string;
  bold?: boolean;
  italics?: boolean;
  underline?: boolean;
  break?: number;
}

function downloadBlob(blob: Blob, fileName: string) {
  const downloadUrl = URL.createObjectURL(blob);
  const link = document.createElement('a');

  link.href = downloadUrl;
  link.download = fileName;
  document.body.appendChild(link);
  link.click();
  link.remove();

  window.setTimeout(() => URL.revokeObjectURL(downloadUrl), 0);
}

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

function htmlToDocxParagraphs(html: string): Paragraph[] {
  const parser = new DOMParser();
  const doc = parser.parseFromString(html, 'text/html');
  const blocks: Paragraph[] = [];

  const pushParagraph = (
    textRuns: InlineRun[],
    options?: Partial<IParagraphOptions>,
  ) => {
    if (textRuns.length === 0) {
      blocks.push(new Paragraph({ text: '', ...(options ?? {}) }));
      return;
    }
    blocks.push(
      new Paragraph({
        children: textRuns.map(toTextRun),
        ...(options ?? {}),
      }),
    );
  };

  for (const node of Array.from(doc.body.childNodes)) {
    if (node.nodeType === Node.TEXT_NODE) {
      const text = node.textContent?.trim();
      if (text) {
        pushParagraph([{ text }]);
      }
      continue;
    }

    if (!(node instanceof HTMLElement)) {
      continue;
    }

    const tag = node.tagName;

    if (tag === 'H1') {
      pushParagraph(getInlineRuns(node), { heading: HeadingLevel.HEADING_1 });
      continue;
    }
    if (tag === 'H2') {
      pushParagraph(getInlineRuns(node), { heading: HeadingLevel.HEADING_2 });
      continue;
    }
    if (tag === 'H3') {
      pushParagraph(getInlineRuns(node), { heading: HeadingLevel.HEADING_3 });
      continue;
    }
    if (tag === 'H4') {
      pushParagraph(getInlineRuns(node), { heading: HeadingLevel.HEADING_4 });
      continue;
    }
    if (tag === 'H5') {
      pushParagraph(getInlineRuns(node), { heading: HeadingLevel.HEADING_5 });
      continue;
    }
    if (tag === 'H6') {
      pushParagraph(getInlineRuns(node), { heading: HeadingLevel.HEADING_6 });
      continue;
    }

    if (tag === 'P') {
      pushParagraph(getInlineRuns(node));
      continue;
    }

    if (tag === 'BLOCKQUOTE') {
      pushParagraph(getInlineRuns(node), { indent: { left: 720 } });
      continue;
    }

    if (tag === 'UL' || tag === 'OL') {
      for (const li of Array.from(node.children)) {
        if (li.tagName !== 'LI') continue;
        pushParagraph(getInlineRuns(li), {
          ...(tag === 'UL' ? { bullet: { level: 0 } } : {}),
          ...(tag === 'OL'
            ? { numbering: { reference: 'numbered-list', level: 0 } }
            : {}),
        });
      }
      continue;
    }

    const text = node.textContent?.trim();
    if (text) {
      pushParagraph([{ text }]);
    }
  }

  return blocks.length > 0 ? blocks : [new Paragraph('')];
}

export function FileManager({ editor, docId, onNotice }: FileManagerProps) {
  const importInputRef = useRef<HTMLInputElement | null>(null);

  const openImportDialog = useCallback(() => {
    importInputRef.current?.click();
  }, []);

  const exportJson = useCallback(() => {
    if (!editor) {
      onNotice('에디터가 아직 준비되지 않았습니다.');
      return;
    }

    try {
      const currentContent = editor.getJSON();
      const fileBlob = new Blob([JSON.stringify(currentContent, null, 2)], {
        type: 'application/json;charset=utf-8',
      });

      downloadBlob(fileBlob, `${docId}-export.json`);
      onNotice('현재 문서를 JSON 파일로 내보냈습니다.');
    } catch {
      onNotice('JSON 파일 내보내기 중 오류가 발생했습니다.');
    }
  }, [docId, editor, onNotice]);

  const exportDocx = useCallback(async () => {
    if (!editor) {
      onNotice('에디터가 아직 준비되지 않았습니다.');
      return;
    }

    try {
      const html = editor.getHTML();
      const paragraphs = htmlToDocxParagraphs(html);

      const doc = new Document({
        sections: [{ children: paragraphs }],
      });

      const blob = await Packer.toBlob(doc);
      downloadBlob(blob, `${docId}-export.docx`);
      onNotice('현재 문서를 DOCX 파일로 내보냈습니다.');
    } catch {
      onNotice('DOCX 파일 내보내기 중 오류가 발생했습니다.');
    }
  }, [docId, editor, onNotice]);

  const handleImportFile = useCallback(
    async (event: ChangeEvent<HTMLInputElement>) => {
      if (!editor) {
        onNotice('에디터가 아직 준비되지 않았습니다.');
        return;
      }

      const selectedFile = event.target.files?.[0];
      if (!selectedFile) {
        onNotice('가져올 파일을 선택하지 않았습니다.');
        return;
      }

      const fileName = selectedFile.name.toLowerCase();

      try {
        if (fileName.endsWith('.json')) {
          const fileText = await selectedFile.text();
          const parsed = JSON.parse(fileText) as JSONContent;
          editor.commands.setContent(parsed);
          onNotice(`JSON 파일을 불러왔습니다: ${selectedFile.name}`);
          return;
        }

        if (fileName.endsWith('.docx')) {
          const arrayBuffer = await selectedFile.arrayBuffer();
          const result = await mammoth.convertToHtml({ arrayBuffer });
          editor.commands.setContent(result.value);
          onNotice(`DOCX 파일을 불러왔습니다: ${selectedFile.name}`);
          return;
        }

        onNotice(
          '지원하지 않는 파일 형식입니다. JSON 또는 DOCX 파일을 선택하세요.',
        );
      } catch {
        onNotice('파일 불러오기 중 오류가 발생했습니다.');
      } finally {
        event.target.value = '';
      }
    },
    [editor, onNotice],
  );

  return (
    <div className="pill-row">
      <button
        className="button-ghost"
        disabled={!editor}
        type="button"
        onClick={openImportDialog}
      >
        Import file
      </button>
      <button
        className="button-link"
        disabled={!editor}
        type="button"
        onClick={exportJson}
      >
        Export JSON
      </button>
      <button
        className="button-link"
        disabled={!editor}
        type="button"
        onClick={exportDocx}
      >
        Export DOCX
      </button>
      <input
        ref={importInputRef}
        accept="application/json,.json,application/vnd.openxmlformats-officedocument.wordprocessingml.document,.docx"
        style={{ display: 'none' }}
        type="file"
        onChange={handleImportFile}
      />
    </div>
  );
}
