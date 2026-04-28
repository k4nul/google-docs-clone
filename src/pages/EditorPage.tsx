import { Link, Navigate, useParams, useSearchParams } from 'react-router-dom';
import type { Editor } from '@tiptap/core';
import { useState } from 'react';

import { EditorShell } from '@/features/editor/EditorShell';
import type { CollaborationSnapshot } from '@/features/editor/EditorShell';
import { FileManager } from '@/features/util/FileManager';
import { buildApiUrl } from '@/lib/api/httpClient';
import { appEnv } from '@/shared/config/env';
import { PageLayout } from '@/shared/ui/PageLayout';

export function EditorPage() {
  const { docId } = useParams<{ docId: string }>();
  const [searchParams] = useSearchParams();
  const [editor, setEditor] = useState<Editor | null>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const [collaboration, setCollaboration] = useState<CollaborationSnapshot>({
    activeCollaborators: [],
    connectionStatus: appEnv.wsUrl ? 'connecting' : 'local-only',
    isCurrentUserTyping: false,
    lastSyncedAt: null,
  });

  if (!docId) {
    return <Navigate replace to="/" />;
  }

  const decodedDocId = decodeURIComponent(docId);
  const accessToken = searchParams.get('accessToken');

  return (
    <PageLayout
      eyebrow="Realtime Document"
      title={`Editor room: ${decodedDocId}`}
      description="Yjs document와 websocket provider를 분리된 초기화 로직으로 연결한 최소 협업 에디터 페이지입니다. 백엔드가 없어도 로컬 모드로 안전하게 렌더링됩니다."
      actions={
        <div className="pill-row">
          <FileManager editor={editor} docId={decodedDocId} onNotice={setNotice} />
          <Link className="button-ghost" to="/">
            Back to document list
          </Link>
        </div>
      }
    >
      <div className="page-grid">
        <div className="aside-stack">
          <EditorShell
            accessToken={accessToken}
            docId={decodedDocId}
            onCollaborationChange={setCollaboration}
            onEditorReady={setEditor}
          />
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
                Document fetch: <code>{buildApiUrl(`/documents/${decodedDocId}`)}</code>
              </span>
              <span>
                Revision sync: <code>{appEnv.wsUrl ?? 'VITE_WS_URL not set'}</code>
              </span>
              <span>
                Document token: <code>{accessToken ? 'provided' : 'missing'}</code>
              </span>
              <span>
                Import ingest: <code>@/lib/import/docxImport.ts</code>
              </span>
            </div>
          </section>

          <section className="card">
            <h3>Realtime collaboration</h3>
            <div className="info-list">
              <span>
                Connection state: <code>{collaboration.connectionStatus}</code>
              </span>
              <span>
                My activity: <code>{collaboration.isCurrentUserTyping ? 'typing' : 'idle'}</code>
              </span>
              <span>
                Last sync event: <code>{collaboration.lastSyncedAt ?? 'not yet synced'}</code>
              </span>
              <span>
                Active users: <code>{collaboration.activeCollaborators.length}</code>
              </span>
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