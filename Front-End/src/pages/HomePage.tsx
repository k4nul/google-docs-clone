import { Link, useNavigate } from 'react-router-dom';
import { useEffect, useMemo, useState } from 'react';

import { createBackendDocument, listBackendDocuments } from '@/lib/api/documents';
import { mockDocuments } from '@/features/documents/mockDocuments';
import { appEnv } from '@/shared/config/env';
import type { DocumentSummary } from '@/shared/types/document';
import { PageLayout } from '@/shared/ui/PageLayout';

function formatUpdatedAt(value: string) {
  const date = new Date(value);

  if (Number.isNaN(date.getTime())) {
    return value;
  }

  return date.toLocaleDateString(undefined, {
    month: 'short',
    day: 'numeric',
    year: 'numeric',
  });
}

function getErrorMessage(error: unknown, fallback: string) {
  return error instanceof Error ? error.message : fallback;
}

type DocumentListStatus = 'loading' | 'backend' | 'fallback';

export function HomePage() {
  const navigate = useNavigate();
  const [documents, setDocuments] = useState<DocumentSummary[]>(mockDocuments);
  const [listStatus, setListStatus] = useState<DocumentListStatus>('loading');
  const [listError, setListError] = useState<string | null>(null);
  const [createError, setCreateError] = useState<string | null>(null);
  const [isCreating, setIsCreating] = useState(false);
  const sampleDocument = mockDocuments[0];
  const canCreateBackendDocument = useMemo(
    () => Boolean(appEnv.apiBaseUrl && appEnv.wsUrl),
    [],
  );
  const listStatusLabel = {
    loading: 'Loading backend documents',
    backend: `${documents.length} backend document${documents.length === 1 ? '' : 's'}`,
    fallback: 'Showing local samples',
  }[listStatus];

  useEffect(() => {
    let isCurrent = true;

    async function loadDocuments() {
      setListStatus('loading');

      try {
        const backendDocuments = await listBackendDocuments();

        if (!isCurrent) {
          return;
        }

        setDocuments(backendDocuments);
        setListError(null);
        setListStatus('backend');
      } catch (error) {
        if (!isCurrent) {
          return;
        }

        setDocuments(mockDocuments);
        setListError(getErrorMessage(error, '문서 목록을 불러오지 못했습니다.'));
        setListStatus('fallback');
      }
    }

    void loadDocuments();

    return () => {
      isCurrent = false;
    };
  }, []);

  async function handleCreateBackendDocument() {
    setCreateError(null);
    setIsCreating(true);

    try {
      const { document } = await createBackendDocument('Realtime collaboration draft');
      navigate(`/docs/${document.id}`);
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
              Open sample editor
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

      <section className="card document-list-header">
        <div>
          <h2>Documents</h2>
          {listError ? <p className="muted">Backend list unavailable: {listError}</p> : null}
        </div>
        <span className={`pill ${listStatus === 'backend' ? 'pill--accent' : ''}`}>
          {listStatusLabel}
        </span>
      </section>

      <div className="card-grid">
        {documents.length === 0 ? (
          <article className="card document-card">
            <div>
              <h2>No backend documents</h2>
              <p className="muted">Create a backend document to start a realtime room.</p>
            </div>
          </article>
        ) : null}

        {documents.map((document) => (
          <article key={document.id} className="card document-card">
            <div className="pill-row">
              <span className="pill pill--accent">
                {document.source === 'backend' ? 'backend' : document.status}
              </span>
              <span className="pill">{document.collaborators} collaborators</span>
            </div>
            <div>
              <h2>{document.title}</h2>
              <p className="muted">{document.summary}</p>
            </div>
            <div className="document-card__meta">
              <span>Last updated: {formatUpdatedAt(document.updatedAt)}</span>
              {document.createdAt ? <span>Created: {formatUpdatedAt(document.createdAt)}</span> : null}
              <span>Source: {document.source === 'backend' ? 'Backend API' : 'Local sample'}</span>
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
            <span>API auth: <code>{appEnv.apiToken ? 'legacy token configured' : 'not required'}</code></span>
            <span>WS provider: <code>{appEnv.wsUrl ?? '(local-only mode)'}</code></span>
            <span>Import path: <code>@/lib/import/docxImport.ts</code></span>
          </div>
        </section>

        <section className="card">
          <h3>Current scope</h3>
          <div className="info-list">
            <span>Backend document list route at <code>/</code></span>
            <span>Collaborative editor route at <code>/docs/:docId</code></span>
            <span>Compile-safe Yjs document/provider shell for backend hookup</span>
          </div>
        </section>
      </div>
    </PageLayout>
  );
}
