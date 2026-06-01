import { useEffect, useMemo, useState } from 'react';
import { Link, useNavigate } from 'react-router-dom';

import {
  createBackendDocument,
  listBackendDocuments,
} from '@/lib/api/documents';
import { mockDocuments } from '@/features/documents/mockDocuments';
import { appEnv } from '@/shared/config/env';
import type { DocumentSummary } from '@/shared/types/document';
import {
  Button,
  EmptyState,
  ErrorState,
  LinkButton,
  LoadingState,
  MetricTile,
  Panel,
  SearchInput,
  StatusPill,
} from '@/shared/ui/DesignSystem';
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
  const [documents, setDocuments] = useState<DocumentSummary[]>([]);
  const [listStatus, setListStatus] = useState<DocumentListStatus>('loading');
  const [listError, setListError] = useState<string | null>(null);
  const [createError, setCreateError] = useState<string | null>(null);
  const [isCreating, setIsCreating] = useState(false);
  const [searchQuery, setSearchQuery] = useState('');
  const sampleDocument = mockDocuments[0];
  const canCreateBackendDocument = useMemo(
    () => Boolean(appEnv.apiBaseUrl && appEnv.wsUrl),
    [],
  );
  const filteredDocuments = useMemo(() => {
    const query = searchQuery.trim().toLowerCase();

    if (!query) {
      return documents;
    }

    return documents.filter((document) =>
      [document.title, document.summary, document.status, document.source]
        .join(' ')
        .toLowerCase()
        .includes(query),
    );
  }, [documents, searchQuery]);
  const listStatusLabel = {
    loading: 'Loading documents',
    backend: `${documents.length} backend document${documents.length === 1 ? '' : 's'}`,
    fallback: 'Showing local sample documents',
  }[listStatus];
  const backendModeLabel = appEnv.apiBaseUrl
    ? 'Backend connected'
    : 'Local preview';
  const realtimeModeLabel = appEnv.wsUrl ? 'Realtime enabled' : 'Local editing';

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
        setListError(getErrorMessage(error, 'Unable to load documents.'));
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
      const { document } = await createBackendDocument('Untitled document');
      navigate(`/docs/${document.id}`);
    } catch (error) {
      setCreateError(
        error instanceof Error ? error.message : 'Unable to create a document.',
      );
    } finally {
      setIsCreating(false);
    }
  }

  return (
    <PageLayout
      eyebrow="Collaborative Editor"
      title="Collaborative document workspace"
      description="A production-oriented workspace for drafting, reviewing, and exporting shared documents with realtime presence and backend document metadata."
      actions={
        <>
          {canCreateBackendDocument ? (
            <Button
              disabled={isCreating}
              type="button"
              onClick={handleCreateBackendDocument}
            >
              {isCreating ? 'Creating document...' : 'Create document'}
            </Button>
          ) : null}
          {sampleDocument ? (
            <LinkButton variant="secondary" to={`/docs/${sampleDocument.id}`}>
              Open sample editor
            </LinkButton>
          ) : null}
        </>
      }
    >
      <section className="dashboard-metrics" aria-label="Workspace summary">
        <MetricTile label="Documents" value={documents.length} />
        <MetricTile label="Data source" value={backendModeLabel} />
        <MetricTile label="Collaboration" value={realtimeModeLabel} />
      </section>

      {createError ? (
        <ErrorState
          description={createError}
          title="Unable to create document"
        />
      ) : null}

      <Panel className="document-toolbar">
        <div>
          <p className="section-kicker">Documents</p>
          <h2>Recent workspace files</h2>
          <p className="muted">
            Search, open, and continue editing shared documents from one
            dashboard.
          </p>
        </div>
        <div className="document-toolbar__controls">
          <SearchInput
            aria-label="Search documents"
            label="Search"
            placeholder="Search by title, status, or source"
            value={searchQuery}
            onChange={(event) => setSearchQuery(event.target.value)}
          />
          <StatusPill tone={listStatus === 'backend' ? 'success' : 'warning'}>
            {listStatusLabel}
          </StatusPill>
        </div>
      </Panel>

      {listError ? (
        <ErrorState
          description={`Backend list unavailable: ${listError}`}
          title="Using local sample documents"
        />
      ) : null}

      {listStatus === 'loading' ? (
        <LoadingState rows={3} title="Loading documents" />
      ) : filteredDocuments.length === 0 ? (
        <EmptyState
          action={
            searchQuery ? (
              <Button variant="secondary" onClick={() => setSearchQuery('')}>
                Clear search
              </Button>
            ) : canCreateBackendDocument ? (
              <Button onClick={handleCreateBackendDocument}>
                Create document
              </Button>
            ) : null
          }
          description={
            searchQuery
              ? 'No documents match the current search. Try a different title or status.'
              : 'Create a document to start a shared workspace, or open the sample editor to review the editing flow.'
          }
          title={searchQuery ? 'No matching documents' : 'No documents yet'}
        />
      ) : (
        <section className="document-grid" aria-label="Document list">
          {filteredDocuments.map((document) => (
            <Link
              key={document.id}
              className="document-card"
              to={`/docs/${document.id}`}
            >
              <span className="document-card__topline">
                <StatusPill
                  tone={document.source === 'backend' ? 'success' : 'neutral'}
                >
                  {document.source === 'backend' ? 'Backend' : document.status}
                </StatusPill>
                <span className="status-pill">
                  {document.collaborators} collaborators
                </span>
              </span>
              <span className="document-card__body">
                <h2 className="document-card__title">{document.title}</h2>
                <span className="document-card__summary">
                  {document.summary}
                </span>
              </span>
              <span className="document-card__meta">
                <span>Updated {formatUpdatedAt(document.updatedAt)}</span>
                {document.createdAt ? (
                  <span>Created {formatUpdatedAt(document.createdAt)}</span>
                ) : null}
                <span>
                  Source:{' '}
                  {document.source === 'backend'
                    ? 'Backend API'
                    : 'Local sample'}
                </span>
              </span>
              <span className="document-card__footer">
                <span>Open editor</span>
                <span aria-hidden="true">→</span>
              </span>
            </Link>
          ))}
        </section>
      )}

      <div className="secondary-grid">
        <Panel>
          <h3>Workspace configuration</h3>
          <div className="info-list">
            <span>
              API base: <code>{appEnv.apiBaseUrl ?? '(not configured)'}</code>
            </span>
            <span>
              API auth:{' '}
              <code>
                {appEnv.apiToken ? 'legacy token configured' : 'not required'}
              </code>
            </span>
            <span>
              WS provider: <code>{appEnv.wsUrl ?? '(local-only mode)'}</code>
            </span>
            <span>
              Import formats: <code>JSON, DOCX</code>
            </span>
          </div>
        </Panel>

        <Panel>
          <h3>Editing flow</h3>
          <div className="info-list">
            <span>Open a document from the dashboard.</span>
            <span>Review metadata before realtime sync starts.</span>
            <span>
              Use the editor toolbar for formatting and export actions.
            </span>
          </div>
        </Panel>
      </div>
    </PageLayout>
  );
}
