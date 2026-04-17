import { Link, Navigate, useParams } from 'react-router-dom';

import { EditorShell } from '@/features/editor/EditorShell';
import { buildApiUrl } from '@/lib/api/httpClient';
import { appEnv } from '@/shared/config/env';
import { PageLayout } from '@/shared/ui/PageLayout';

export function EditorPage() {
  const { docId } = useParams<{ docId: string }>();

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
        <Link className="button-ghost" to="/">
          Back to document list
        </Link>
      }
    >
      <div className="page-grid">
        <EditorShell docId={decodedDocId} />

        <aside className="aside-stack">
          <section className="card">
            <h3>Backend integration points</h3>
            <div className="info-list">
              <span>Document fetch: <code>{buildApiUrl(`/documents/${decodedDocId}`)}</code></span>
              <span>Revision sync: <code>{appEnv.wsUrl ?? 'VITE_WS_URL not set'}</code></span>
              <span>Import ingest: <code>@/lib/import/docxImport.ts</code></span>
            </div>
          </section>

          <section className="card">
            <h3>Current behavior</h3>
            <div className="info-list">
              <span>StarterKit history is disabled only when collaboration is active.</span>
              <span>Carets are enabled when websocket provider is configured.</span>
              <span>Editor content is intentionally local-first until backend APIs exist.</span>
            </div>
          </section>
        </aside>
      </div>
    </PageLayout>
  );
}
