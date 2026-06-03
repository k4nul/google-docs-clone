import type { CollaborationSnapshot } from '@/features/editor/EditorShell';
import { StatusPill } from '@/shared/ui/DesignSystem';

export type EditorDocumentDetailStatus =
  | 'loading'
  | 'ready'
  | 'credential required'
  | 'unavailable';

interface EditorDocumentTimestamps {
  createdAt: string;
  updatedAt: string;
}

interface EditorDetailsSidebarProps {
  collaboration: CollaborationSnapshot;
  documentStatus: EditorDocumentDetailStatus;
  documentTimestamps: EditorDocumentTimestamps | null;
  isCollaborationReady: boolean;
}

export function EditorDetailsSidebar({
  collaboration,
  documentStatus,
  documentTimestamps,
  isCollaborationReady,
}: EditorDetailsSidebarProps) {
  return (
    <aside className="editor-side-stack" aria-label="Document details">
      <section className="editor-side-card">
        <div>
          <p className="section-kicker">Document</p>
          <h2>Details</h2>
        </div>
        <div className="info-list">
          <span>
            Document details: <code>{documentStatus}</code>
          </span>
          {documentTimestamps ? (
            <>
              <span>
                Created: <code>{documentTimestamps.createdAt}</code>
              </span>
              <span>
                Updated: <code>{documentTimestamps.updatedAt}</code>
              </span>
            </>
          ) : null}
          <span>
            Collaboration:{' '}
            <code>
              {isCollaborationReady
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
            <code>{collaboration.isCurrentUserTyping ? 'typing' : 'idle'}</code>
          </span>
          <span>
            Last sync event:{' '}
            <code>{collaboration.lastSyncedAt ?? 'not yet synced'}</code>
          </span>
          <span>
            Active users: <code>{collaboration.activeCollaborators.length}</code>
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
  );
}
