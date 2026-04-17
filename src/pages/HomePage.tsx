import { Link } from 'react-router-dom';

import { mockDocuments } from '@/features/documents/mockDocuments';
import { buildApiUrl } from '@/lib/api/httpClient';
import { appEnv } from '@/shared/config/env';
import { PageLayout } from '@/shared/ui/PageLayout';

function formatUpdatedAt(value: string) {
  return new Date(value).toLocaleDateString(undefined, {
    month: 'short',
    day: 'numeric',
    year: 'numeric',
  });
}

export function HomePage() {
  const sampleDocument = mockDocuments[0];

  return (
    <PageLayout
      eyebrow="Collaborative Editor"
      title="Collaborative document workspace"
      description="React + Vite + Tiptap + Yjs 기반의 최소 협업 에디터 프론트엔드입니다. 현재는 문서 목록과 협업 에디터 셸을 연결해 둔 상태이며, 백엔드가 준비되면 바로 실시간 문서 흐름을 붙일 수 있습니다."
      actions={
        sampleDocument ? (
          <Link className="button-link" to={`/docs/${sampleDocument.id}`}>
            Open sample editor
          </Link>
        ) : undefined
      }
    >
      <div className="card-grid">
        {mockDocuments.map((document) => (
          <article key={document.id} className="card document-card">
            <div className="pill-row">
              <span className="pill pill--accent">{document.status}</span>
              <span className="pill">{document.collaborators} collaborators</span>
            </div>
            <div>
              <h2>{document.title}</h2>
              <p className="muted">{document.summary}</p>
            </div>
            <div className="document-card__meta">
              <span>Last updated: {formatUpdatedAt(document.updatedAt)}</span>
              <span>
                Future API path: <code>{buildApiUrl(`/documents/${document.id}`)}</code>
              </span>
            </div>
            <div>
              <Link className="button-ghost" to={`/docs/${document.id}`}>
                Open editor
              </Link>
            </div>
          </article>
        ))}
      </div>

      <div className="card-grid">
        <section className="card">
          <h3>Runtime wiring</h3>
          <div className="info-list">
            <span>API base: <code>{appEnv.apiBaseUrl ?? '(not configured)'}</code></span>
            <span>WS provider: <code>{appEnv.wsUrl ?? '(local-only mode)'}</code></span>
            <span>Import path: <code>@/lib/import/docxImport.ts</code></span>
          </div>
        </section>

        <section className="card">
          <h3>Current scope</h3>
          <div className="info-list">
            <span>Document list placeholder route at <code>/</code></span>
            <span>Collaborative editor route at <code>/docs/:docId</code></span>
            <span>Compile-safe Yjs document/provider shell for backend hookup</span>
          </div>
        </section>
      </div>
    </PageLayout>
  );
}
