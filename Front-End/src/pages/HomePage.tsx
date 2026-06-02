import { useCallback, useEffect, useMemo, useState } from 'react';
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
  LoadingState,
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

function formatDocumentCount(count: number) {
  return `${count} document${count === 1 ? '' : 's'}`;
}

function formatCollaborators(count: number) {
  return `${count} collaborator${count === 1 ? '' : 's'}`;
}

type DocumentListStatus = 'loading' | 'ready' | 'fallback';

type ResolvedDocumentList = {
  documents: DocumentSummary[];
  error: string | null;
  status: Exclude<DocumentListStatus, 'loading'>;
};

async function resolveDocumentList(): Promise<ResolvedDocumentList> {
  try {
    return {
      documents: await listBackendDocuments(),
      error: null,
      status: 'ready',
    };
  } catch (error) {
    return {
      documents: mockDocuments,
      error: getErrorMessage(error, 'Unable to load documents.'),
      status: 'fallback',
    };
  }
}

export function HomePage() {
  const navigate = useNavigate();
  const [documents, setDocuments] = useState<DocumentSummary[]>([]);
  const [listStatus, setListStatus] = useState<DocumentListStatus>('loading');
  const [listError, setListError] = useState<string | null>(null);
  const [createError, setCreateError] = useState<string | null>(null);
  const [isCreating, setIsCreating] = useState(false);
  const [searchQuery, setSearchQuery] = useState('');
  const canCreateBackendDocument = useMemo(
    () => Boolean(appEnv.apiBaseUrl && appEnv.apiToken && appEnv.wsUrl),
    [],
  );
  const filteredDocuments = useMemo(() => {
    const query = searchQuery.trim().toLowerCase();

    if (!query) {
      return documents;
    }

    return documents.filter((document) =>
      [document.title, document.summary]
        .join(' ')
        .toLowerCase()
        .includes(query),
    );
  }, [documents, searchQuery]);
  const listStatusLabel =
    listStatus === 'loading'
      ? 'Loading documents'
      : formatDocumentCount(documents.length);

  const applyDocumentList = useCallback((nextList: ResolvedDocumentList) => {
    setDocuments(nextList.documents);
    setListError(nextList.error);
    setListStatus(nextList.status);
  }, []);

  const loadDocuments = useCallback(async () => {
    setListStatus('loading');
    applyDocumentList(await resolveDocumentList());
  }, [applyDocumentList]);

  useEffect(() => {
    let isCurrent = true;

    void resolveDocumentList().then((nextList) => {
      if (isCurrent) {
        applyDocumentList(nextList);
      }
    });

    return () => {
      isCurrent = false;
    };
  }, [applyDocumentList]);

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
      eyebrow="Documents"
      title="Collaborative document workspace"
      description="Create, reopen, and review shared documents from one focused workspace."
      actions={
        <>
          {canCreateBackendDocument ? (
            <Button
              disabled={isCreating}
              type="button"
              onClick={handleCreateBackendDocument}
            >
              {isCreating ? 'Creating document...' : 'New document'}
            </Button>
          ) : null}
        </>
      }
    >
      {createError ? (
        <ErrorState
          description={createError}
          title="Unable to create document"
        />
      ) : null}

      <Panel className="document-toolbar">
        <div>
          <p className="section-kicker">Documents</p>
          <h2>Recent documents</h2>
          <p className="muted">
            Search and reopen the documents you have been working on.
          </p>
        </div>
        <div className="document-toolbar__controls">
          <SearchInput
            aria-label="Search documents"
            label="Search"
            placeholder="Search by title or preview"
            value={searchQuery}
            onChange={(event) => setSearchQuery(event.target.value)}
          />
          <StatusPill tone={listStatus === 'fallback' ? 'warning' : 'neutral'}>
            {listStatusLabel}
          </StatusPill>
        </div>
      </Panel>

      {listError ? (
        <ErrorState
          action={
            <Button variant="secondary" onClick={() => void loadDocuments()}>
              Retry
            </Button>
          }
          description={`${listError} Try again to refresh your recent documents.`}
          title="Documents are temporarily unavailable"
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
              ? 'No documents match the current search. Try a different title or preview.'
              : 'Create a document to start writing with your team.'
          }
          title={searchQuery ? 'No matching documents' : 'No documents yet'}
        />
      ) : (
        <div className="document-list-scroll">
          <section className="document-grid" aria-label="Document list">
            {filteredDocuments.map((document) => (
              <Link
                key={document.id}
                className="document-card"
                to={`/docs/${document.id}`}
              >
                <span className="document-card__topline">
                  <span className="status-pill">
                    {formatCollaborators(document.collaborators)}
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
                </span>
                <span className="document-card__footer">
                  <span>Open document</span>
                  <span aria-hidden="true">→</span>
                </span>
              </Link>
            ))}
          </section>
        </div>
      )}
    </PageLayout>
  );
}
