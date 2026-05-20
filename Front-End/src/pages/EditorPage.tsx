import { Link, Navigate, useParams } from 'react-router-dom';
import type { Editor } from '@tiptap/core';
import { useEffect, useState } from 'react';

import { EditorShell } from '@/features/editor/EditorShell';
import type { CollaborationSnapshot } from '@/features/editor/EditorShell';
import { FileManager } from '@/features/util/FileManager';
import { getBackendDocument } from '@/lib/api/documents';
import { ApiRequestError, buildApiUrl } from '@/lib/api/httpClient';
import { appEnv } from '@/shared/config/env';
import type { BackendDocument } from '@/shared/types/document';
import { PageLayout } from '@/shared/ui/PageLayout';

function formatMetadataError(error: unknown) {
  if (error instanceof ApiRequestError) {
    const owner = error.payload?.owner;

    if (owner?.node_id) {
      return `${error.message} Owner: ${owner.node_id}${owner.base_url ? ` (${owner.base_url})` : ''}`;
    }
  }

  return error instanceof Error
    ? error.message
    : '문서 메타데이터를 불러오지 못했습니다.';
}

function formatDateTime(value: string) {
  const date = new Date(value);

  if (Number.isNaN(date.getTime())) {
    return value;
  }

  return date.toLocaleString();
}

type DocumentMetadataState =
  | {
      docId: string | null;
      status: 'loading';
      document: null;
      error: null;
    }
  | {
      docId: string;
      status: 'loaded';
      document: BackendDocument;
      error: null;
    }
  | {
      docId: string;
      status: 'error';
      document: null;
      error: string;
    };

export function EditorPage() {
  const { docId } = useParams<{ docId: string }>();
  const decodedDocId = docId ? decodeURIComponent(docId) : '';
  const [editor, setEditor] = useState<Editor | null>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const [metadataState, setMetadataState] = useState<DocumentMetadataState>({
    docId: null,
    status: 'loading',
    document: null,
    error: null,
  });
  const [collaboration, setCollaboration] = useState<CollaborationSnapshot>({
    activeCollaborators: [],
    connectionStatus: appEnv.wsUrl ? 'connecting' : 'local-only',
    isCurrentUserTyping: false,
    lastSyncedAt: null,
  });

  useEffect(() => {
    if (!decodedDocId) {
      return undefined;
    }

    let isCurrent = true;

    async function loadDocument() {
      try {
        const backendDocument = await getBackendDocument(decodedDocId);

        if (!isCurrent) {
          return;
        }

        setMetadataState({
          docId: decodedDocId,
          status: 'loaded',
          document: backendDocument,
          error: null,
        });
      } catch (error) {
        if (!isCurrent) {
          return;
        }

        setMetadataState({
          docId: decodedDocId,
          status: 'error',
          document: null,
          error: formatMetadataError(error),
        });
      }
    }

    void loadDocument();

    return () => {
      isCurrent = false;
    };
  }, [decodedDocId]);

  const isMetadataCurrent = metadataState.docId === decodedDocId;
  const document =
    isMetadataCurrent && metadataState.status === 'loaded'
      ? metadataState.document
      : null;
  const documentError =
    isMetadataCurrent && metadataState.status === 'error'
      ? metadataState.error
      : null;
  const isDocumentLoading =
    !isMetadataCurrent || metadataState.status === 'loading';

  if (!docId) {
    return <Navigate replace to="/" />;
  }

  const realtimeServerUrl = document && !documentError ? appEnv.wsUrl : null;

  return (
    <PageLayout
      eyebrow="Realtime Document"
      title={document?.title ?? `Editor room: ${decodedDocId}`}
      description="Backend document metadata gates the realtime connection, then the editor joins the matching Yjs room with reconnect and presence status visible."
      actions={
        <div className="pill-row">
          <FileManager
            editor={editor}
            docId={decodedDocId}
            onNotice={setNotice}
          />
          <Link className="button-ghost" to="/">
            Back to document list
          </Link>
        </div>
      }
    >
      <div className="page-grid">
        <div className="aside-stack">
          {isDocumentLoading ? (
            <section className="card editor-loading">
              <p>Loading document metadata...</p>
            </section>
          ) : (
            <EditorShell
              docId={decodedDocId}
              realtimeServerUrl={realtimeServerUrl}
              onCollaborationChange={setCollaboration}
              onEditorReady={setEditor}
            />
          )}
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
                Document fetch:{' '}
                <code>{buildApiUrl(`/documents/${decodedDocId}`)}</code>
              </span>
              <span>
                Metadata:{' '}
                <code>
                  {isDocumentLoading
                    ? 'loading'
                    : document
                      ? 'loaded'
                      : 'unavailable'}
                </code>
              </span>
              {document ? (
                <>
                  <span>
                    Created: <code>{formatDateTime(document.createdAt)}</code>
                  </span>
                  <span>
                    Updated: <code>{formatDateTime(document.updatedAt)}</code>
                  </span>
                </>
              ) : null}
              <span>
                Revision sync:{' '}
                <code>
                  {realtimeServerUrl ?? 'disabled until metadata loads'}
                </code>
              </span>
              <span>
                Import ingest: <code>@/lib/import/docxImport.ts</code>
              </span>
            </div>
          </section>

          {documentError ? (
            <section className="card">
              <h3>Document metadata status</h3>
              <p className="muted">{documentError}</p>
            </section>
          ) : null}

          <section className="card">
            <h3>Realtime collaboration</h3>
            <div className="info-list">
              <span>
                Connection state: <code>{collaboration.connectionStatus}</code>
              </span>
              <span>
                My activity:{' '}
                <code>
                  {collaboration.isCurrentUserTyping ? 'typing' : 'idle'}
                </code>
              </span>
              <span>
                Last sync event:{' '}
                <code>{collaboration.lastSyncedAt ?? 'not yet synced'}</code>
              </span>
              {/* <span>
                Active users: <code>{collaboration.activeCollaborators.length}</code>
              </span> */}
            </div>
            <div className="pill-row" style={{ marginTop: '12px' }}>
              {collaboration.activeCollaborators.length > 0 ? (
                collaboration.activeCollaborators.map((collaborator) => (
                  <span
                    key={collaborator.id}
                    className={`pill ${collaborator.isTyping ? 'pill--accent' : ''}`}
                    title={collaborator.color}
                  >
                    {collaborator.name}
                    {collaborator.isCurrentUser ? ' (me)' : ''}
                    {collaborator.isTyping ? ' - typing' : ''}
                  </span>
                ))
              ) : (
                <span className="pill">No active collaborators yet</span>
              )}
            </div>
          </section>
        </aside>
      </div>
    </PageLayout>
  );
}
