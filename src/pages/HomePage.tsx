import { Link, useNavigate } from 'react-router-dom';
import { useMemo, useState } from 'react';

import { createBackendDocument } from '@/lib/api/documents';
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
  const navigate = useNavigate();
  const [createError, setCreateError] = useState<string | null>(null);
  const [isCreating, setIsCreating] = useState(false);
  const sampleDocument = mockDocuments[0];
  const canCreateBackendDocument = useMemo(
    () => Boolean(appEnv.apiBaseUrl && appEnv.apiToken && appEnv.wsUrl),
    [],
  );

  async function handleCreateBackendDocument() {
    setCreateError(null);
    setIsCreating(true);

    try {
      const { accessToken, document } = await createBackendDocument('Realtime collaboration draft');
      navigate(`/docs/${document.id}?accessToken=${encodeURIComponent(accessToken)}`);
    } catch (error) {
      setCreateError(error instanceof Error ? error.message : '백엔드 문서 생성에 실패했습니다.');
    } finally {
      setIsCreating(false);
    }
  }

  return (
    <PageLayout
      eyebrow="Collaborative Editor"
      title="Collaborative document workspace"
      description="React + Vite + Tiptap + Yjs 기반의 최소 협업 에디터 프론트엔드입니다. 현재는 문서 목록과 협업 에디터 셸을 연결해 둔 상태이며, 백엔드가 준비되면 바로 실시간 문서 흐름을 붙일 수 있습니다."
      actions={
        <div className="pill-row">
          {canCreateBackendDocument ? (
            <button className="button-link" disabled={isCreating} type="button" onClick={handleCreateBackendDocument}>
              {isCreating ? 'Creating backend doc...' : 'Create backend editor'}
            </button>
          ) : null}
          {sampleDocument ? (
            <Link className="button-ghost" to={`/docs/${sampleDocument.id}`}>
              Open mock editor
            </Link>
          ) : null}
        </div>
      }
    >
      {createError ? (
        <section className="card">
          <h3>Backend create error</h3>
          <p className="muted">{createError}</p>
        </section>
      ) : null}

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
            <span>API token: <code>{appEnv.apiToken ?? '(not configured)'}</code></span>
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
