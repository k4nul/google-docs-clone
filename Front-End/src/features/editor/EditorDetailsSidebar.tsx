import type { CollaborationSnapshot } from '@/features/editor/EditorShell';
import { Badge } from '@/shared/ui';

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
            Open status: <strong>{documentStatus}</strong>
          </span>
          {documentTimestamps ? (
            <>
              <span>
                Created {documentTimestamps.createdAt}
              </span>
              <span>
                Updated {documentTimestamps.updatedAt}
              </span>
            </>
          ) : null}
          <span>
            Collaboration{' '}
            {isCollaborationReady ? 'is ready' : 'starts after access opens'}
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
            Connection {collaboration.connectionStatus}
          </span>
          <span>
            You are {collaboration.isCurrentUserTyping ? 'typing' : 'idle'}
          </span>
          <span>
            Last saved {collaboration.lastSyncedAt ?? 'not yet synced'}
          </span>
          <span>
            Active users {collaboration.activeCollaborators.length}
          </span>
        </div>
        <div className="presence-list">
          {collaboration.activeCollaborators.length > 0 ? (
            collaboration.activeCollaborators.map((collaborator) => (
              <Badge
                key={collaborator.id}
                tone={collaborator.isTyping ? 'success' : 'neutral'}
                title={collaborator.color}
              >
                {collaborator.name}
                {collaborator.isCurrentUser ? ' (me)' : ''}
                {collaborator.isTyping ? ' - typing' : ''}
              </Badge>
            ))
          ) : (
            <Badge>No active collaborators yet</Badge>
          )}
        </div>
      </section>
    </aside>
  );
}
