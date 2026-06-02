import { Navigate, useParams } from 'react-router-dom';
import type { Editor } from '@tiptap/core';
import { type FormEvent, useEffect, useState } from 'react';

import { EditorShell } from '@/features/editor/EditorShell';
import type { CollaborationSnapshot } from '@/features/editor/EditorShell';
import { FileManager } from '@/features/util/FileManager';
import {
  getBackendDocument,
  getStoredDocumentAccessToken,
  storeDocumentAccessToken,
  updateBackendDocumentTitle,
} from '@/lib/api/documents';
import { ApiRequestError } from '@/lib/api/httpClient';
import { appEnv } from '@/shared/config/env';
import type { BackendDocument } from '@/shared/types/document';
import {
  Button,
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

function getErrorMessage(error: unknown, fallback: string) {
  return error instanceof Error ? error.message : fallback;
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
  const [documentCredential, setDocumentCredential] = useState<{
    docId: string;
    accessToken: string | null;
  }>(() => ({
    accessToken: decodedDocId
      ? getStoredDocumentAccessToken(decodedDocId)
      : null,
    docId: decodedDocId,
  }));
  const [credentialInputState, setCredentialInputState] = useState({
    docId: decodedDocId,
    value: '',
  });
  const [credentialErrorState, setCredentialErrorState] = useState<{
    docId: string;
    error: string | null;
  }>({
    docId: decodedDocId,
    error: null,
  });
  const [titleMutationState, setTitleMutationState] = useState<{
    docId: string;
    error: string | null;
    status: 'idle' | 'saving';
  }>({
    docId: decodedDocId,
    error: null,
    status: 'idle',
  });
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
  const documentAccessToken =
    documentCredential.docId === decodedDocId
      ? documentCredential.accessToken
      : decodedDocId
        ? getStoredDocumentAccessToken(decodedDocId)
        : null;
  const credentialInput =
    credentialInputState.docId === decodedDocId
      ? credentialInputState.value
      : '';
  const credentialError =
    credentialErrorState.docId === decodedDocId
      ? credentialErrorState.error
      : null;
  const titleMutation =
    titleMutationState.docId === decodedDocId
      ? titleMutationState
      : { error: null, status: 'idle' as const };

  useEffect(() => {
    if (!decodedDocId || !documentAccessToken) {
      return undefined;
    }

    let isCurrent = true;

    async function loadDocument() {
      setMetadataState({
        docId: decodedDocId,
        status: 'loading',
        document: null,
        error: null,
      });

      try {
        const backendDocument = await getBackendDocument(
          decodedDocId,
          documentAccessToken,
        );

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
  }, [decodedDocId, documentAccessToken]);

  const isMetadataCurrent = metadataState.docId === decodedDocId;
  const document =
    isMetadataCurrent && metadataState.status === 'loaded'
      ? metadataState.document
      : null;
  const documentError =
    isMetadataCurrent && metadataState.status === 'error'
      ? metadataState.error
      : null;
  const isCredentialRequired = Boolean(decodedDocId && !documentAccessToken);
  const isDocumentLoading =
    Boolean(documentAccessToken) &&
    (!isMetadataCurrent || metadataState.status === 'loading');

  if (!docId) {
    return <Navigate replace to="/" />;
  }

  const realtimeServerUrl =
    document && documentAccessToken && !documentError ? appEnv.wsUrl : null;
  const pageTitle = document?.title ?? 'Untitled document';
  const pageDescription = document
    ? 'Continue editing this document with realtime presence.'
    : isCredentialRequired
      ? 'Enter the document credential to open this workspace.'
    : documentError
      ? 'Document metadata could not be loaded with the current credential.'
      : 'Document metadata is loading before the editor joins the realtime workspace.';

  function handleCredentialSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const token = credentialInput.trim();

    if (!token) {
      setCredentialErrorState({
        docId: decodedDocId,
        error: 'Enter the document access token to continue.',
      });
      return;
    }

    storeDocumentAccessToken(decodedDocId, token);
    setCredentialErrorState({ docId: decodedDocId, error: null });
    setDocumentCredential({
      accessToken: token,
      docId: decodedDocId,
    });
  }

  async function handleTitleSubmit(title: string) {
    if (!document || !documentAccessToken) {
      return;
    }

    setTitleMutationState({
      docId: decodedDocId,
      error: null,
      status: 'saving',
    });

    try {
      const updatedDocument = await updateBackendDocumentTitle(
        decodedDocId,
        title,
        documentAccessToken,
      );
      setMetadataState({
        docId: decodedDocId,
        status: 'loaded',
        document: updatedDocument,
        error: null,
      });
      setTitleMutationState({
        docId: decodedDocId,
        error: null,
        status: 'idle',
      });
    } catch (error) {
      setTitleMutationState({
        docId: decodedDocId,
        error: getErrorMessage(error, 'Unable to rename this document.'),
        status: 'idle',
      });
    }
  }

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
            <>
              {isCredentialRequired ? (
                <Panel>
                  <form
                    className="credential-form"
                    onSubmit={handleCredentialSubmit}
                  >
                    <div>
                      <p className="section-kicker">Document access</p>
                      <h2>Credential required</h2>
                      <p className="muted">
                        Paste the access token created with this document to
                        reopen it.
                      </p>
                    </div>
                    <label className="credential-form__field">
                      <span>Access token</span>
                      <input
                        autoComplete="off"
                        value={credentialInput}
                        onChange={(event) =>
                          setCredentialInputState({
                            docId: decodedDocId,
                            value: event.target.value,
                          })
                        }
                      />
                    </label>
                    {credentialError ? (
                      <p className="form-error">{credentialError}</p>
                    ) : null}
                    <Button type="submit">Unlock document</Button>
                  </form>
                </Panel>
              ) : document ? (
                <EditorShell
                  docId={decodedDocId}
                  documentAccessToken={documentAccessToken}
                  documentTitle={pageTitle}
                  lastEditedAt={
                    document.updatedAt
                      ? formatDateTime(document.updatedAt)
                      : null
                  }
                  realtimeServerUrl={realtimeServerUrl}
                  titleError={titleMutation.error}
                  titleStatus={titleMutation.status}
                  onCollaborationChange={setCollaboration}
                  onEditorReady={setEditor}
                  onTitleSubmit={handleTitleSubmit}
                />
              ) : (
                <Panel>
                  <ErrorState
                    description={
                      documentError ?? 'Document metadata is unavailable.'
                    }
                    title="Document metadata unavailable"
                  />
                  <form
                    className="credential-form credential-form--retry"
                    onSubmit={handleCredentialSubmit}
                  >
                    <label className="credential-form__field">
                      <span>Access token</span>
                      <input
                        autoComplete="off"
                        value={credentialInput}
                        onChange={(event) =>
                          setCredentialInputState({
                            docId: decodedDocId,
                            value: event.target.value,
                          })
                        }
                      />
                    </label>
                    {credentialError ? (
                      <p className="form-error">{credentialError}</p>
                    ) : null}
                    <Button type="submit">Try credential</Button>
                  </form>
                </Panel>
              )}
            </>
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
                      : isCredentialRequired
                        ? 'credential required'
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
