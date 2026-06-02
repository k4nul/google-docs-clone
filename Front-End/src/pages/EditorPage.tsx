import { Navigate, useParams } from 'react-router-dom';
import type { Editor } from '@tiptap/core';
import { useEffect, useState } from 'react';

import { EditorShell } from '@/features/editor/EditorShell';
import type { CollaborationSnapshot } from '@/features/editor/EditorShell';
import { FileManager } from '@/features/util/FileManager';
import { getBackendDocument } from '@/lib/api/documents';
import { ApiRequestError } from '@/lib/api/httpClient';
import { appEnv } from '@/shared/config/env';
import type { BackendDocument } from '@/shared/types/document';
import {
  ErrorState,
  LinkButton,
  LoadingState,
  Panel,
  StatusPill,
} from '@/shared/ui/DesignSystem';
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
    : 'Unable to load document metadata.';
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
  const pageTitle = document?.title ?? 'Untitled document';
  const pageDescription = document
    ? 'Continue editing this document with realtime presence.'
    : documentError
      ? 'The editor is available in a protected local mode while document metadata is unavailable.'
      : 'Document metadata is loading before the editor joins the realtime workspace.';

  return (
    <PageLayout
      eyebrow="Realtime Document"
      title={pageTitle}
      description={pageDescription}
      actions={
        <>
          <FileManager
            editor={editor}
            docId={decodedDocId}
            onNotice={setNotice}
          />
          <LinkButton variant="secondary" to="/">
            Back to document list
          </LinkButton>
        </>
      }
    >
      <div className="editor-layout">
        <div className="editor-main-stack">
          {isDocumentLoading ? (
            <Panel>
              <LoadingState rows={2} title="Loading document metadata" />
            </Panel>
          ) : (
            <EditorShell
              docId={decodedDocId}
              documentTitle={pageTitle}
              lastEditedAt={
                document?.updatedAt ? formatDateTime(document.updatedAt) : null
              }
              realtimeServerUrl={realtimeServerUrl}
              onCollaborationChange={setCollaboration}
              onEditorReady={setEditor}
            />
          )}
          {notice ? (
            <Panel>
              <h2>File status</h2>
              <p className="muted">{notice}</p>
            </Panel>
          ) : null}
        </div>

        <aside className="editor-side-stack" aria-label="Document details">
          <section className="editor-side-card">
            <div>
              <p className="section-kicker">Document</p>
              <h2>Details</h2>
            </div>
            <div className="info-list">
              <span>
                Document details:{' '}
                <code>
                  {isDocumentLoading
                    ? 'loading'
                    : document
                      ? 'ready'
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
                Collaboration:{' '}
                <code>
                  {realtimeServerUrl
                    ? 'ready'
                    : 'available after document details load'}
                </code>
              </span>
            </div>
          </section>

          {documentError ? (
            <ErrorState
              description={documentError}
              title="Document metadata unavailable"
            />
          ) : null}

          <section className="editor-side-card">
            <div>
              <p className="section-kicker">Collaboration</p>
              <h2>Realtime presence</h2>
            </div>
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
              <span>
                Active users:{' '}
                <code>{collaboration.activeCollaborators.length}</code>
              </span>
            </div>
            <div className="presence-list">
              {collaboration.activeCollaborators.length > 0 ? (
                collaboration.activeCollaborators.map((collaborator) => (
                  <StatusPill
                    key={collaborator.id}
                    tone={collaborator.isTyping ? 'success' : 'neutral'}
                    title={collaborator.color}
                  >
                    {collaborator.name}
                    {collaborator.isCurrentUser ? ' (me)' : ''}
                    {collaborator.isTyping ? ' - typing' : ''}
                  </StatusPill>
                ))
              ) : (
                <StatusPill>No active collaborators yet</StatusPill>
              )}
            </div>
          </section>
        </aside>
      </div>
    </PageLayout>
  );
}
