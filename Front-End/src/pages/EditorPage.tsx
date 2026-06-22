import { Navigate, useParams } from 'react-router-dom';
import type { Editor } from '@tiptap/core';
import { useEffect, useState } from 'react';
import type { FormEvent } from 'react';

import { EditorAccessForm } from '@/features/editor/EditorAccessForm';
import { EditorDetailsSidebar } from '@/features/editor/EditorDetailsSidebar';
import type { EditorDocumentDetailStatus } from '@/features/editor/EditorDetailsSidebar';
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
  Card,
  ErrorState,
  LinkButton,
  LoadingState,
} from '@/shared/ui';
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
  const documentStatus: EditorDocumentDetailStatus = isDocumentLoading
    ? 'loading'
    : document
      ? 'ready'
      : isCredentialRequired
        ? 'credential required'
        : 'unavailable';
  const documentTimestamps = document
    ? {
        createdAt: formatDateTime(document.createdAt),
        updatedAt: formatDateTime(document.updatedAt),
      }
    : null;
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

  function handleCredentialInputChange(value: string) {
    setCredentialInputState({
      docId: decodedDocId,
      value,
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
            <Card>
              <LoadingState rows={2} title="Loading document metadata" />
            </Card>
          ) : (
            <>
              {isCredentialRequired ? (
                <Card>
                  <EditorAccessForm
                    description="Paste the access token created with this document to reopen it."
                    error={credentialError}
                    heading="Credential required"
                    kicker="Document access"
                    submitLabel="Unlock document"
                    value={credentialInput}
                    onSubmit={handleCredentialSubmit}
                    onValueChange={handleCredentialInputChange}
                  />
                </Card>
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
                <Card>
                  <ErrorState
                    description={
                      documentError ?? 'Document metadata is unavailable.'
                    }
                    title="Document metadata unavailable"
                  />
                  <EditorAccessForm
                    className="credential-form--retry"
                    error={credentialError}
                    submitLabel="Try credential"
                    value={credentialInput}
                    onSubmit={handleCredentialSubmit}
                    onValueChange={handleCredentialInputChange}
                  />
                </Card>
              )}
            </>
          )}
          {notice ? (
            <Card>
              <h2>File status</h2>
              <p className="muted">{notice}</p>
            </Card>
          ) : null}
        </div>

        <EditorDetailsSidebar
          collaboration={collaboration}
          documentStatus={documentStatus}
          documentTimestamps={documentTimestamps}
          isCollaborationReady={Boolean(realtimeServerUrl)}
        />
      </div>
    </PageLayout>
  );
}
