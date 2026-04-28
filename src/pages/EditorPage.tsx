import { Link, Navigate, useParams } from 'react-router-dom';
import type { Editor } from '@tiptap/core';
import { useState } from 'react';

import { EditorShell } from '@/features/editor/EditorShell';
import { FileManager } from '@/features/util/FileManager';
import { buildApiUrl } from '@/lib/api/httpClient';
import { appEnv } from '@/shared/config/env';
import { PageLayout } from '@/shared/ui/PageLayout';

export function EditorPage() {
  const { docId } = useParams<{ docId: string }>();
  const [editor, setEditor] = useState<Editor | null>(null);
  const [notice, setNotice] = useState<string | null>(null);

  if (!docId) {
    return <Navigate replace to="/" />;
  }

  const decodedDocId = decodeURIComponent(docId);

  return (
    <PageLayout
      eyebrow="Realtime Document"
      title={`Editor room: ${decodedDocId}`}
      description="Yjs document와 websocket provider를 분리된 초기화 로직으로 연결한 최소 협업 에디터 페이지입니다. 백엔드가 없어도 로컬 모드로 안전하게 렌더링됩니다."
      actions={
        <div className="pill-row">
          <FileManager editor={editor} docId={decodedDocId} onNotice={setNotice} />
          <Link className="button-ghost" to="/">
            Back to document list
          </Link>
        </div>
      }
    >
      <div className="page-grid">
        <div className="aside-stack">
          <EditorShell docId={decodedDocId} onEditorReady={setEditor} />
          {notice ? (
            <section className="card">
              <h3>Import / Export status</h3>
              <p className="muted">{notice}</p>
            </section>
          ) : null}
        </div>

        <aside className="aside-stack">
          <section className="card">
            <h3>Backend integration points</h3>
            <div className="info-list">
              <span>
                Document fetch: <code>{buildApiUrl(`/documents/${decodedDocId}`)}</code>
              </span>
              <span>
                Revision sync: <code>{appEnv.wsUrl ?? 'VITE_WS_URL not set'}</code>
              </span>
              <span>
                Import ingest: <code>@/lib/import/docxImport.ts</code>
              </span>
            </div>
          </section>
        </aside>
      </div>
    </PageLayout>
  );
}