import { WebsocketProvider } from 'y-websocket';
import * as Y from 'yjs';

export interface CollaborationConnection {
  roomId: string;
  doc: Y.Doc;
  provider: WebsocketProvider | null;
}

interface CreateCollaborationConnectionParams {
  roomId: string;
  serverUrl: string | null;
}

export function createCollaborationConnection({
  roomId,
  serverUrl,
}: CreateCollaborationConnectionParams): CollaborationConnection {
  const normalizedRoomId = roomId.trim() || 'default-room';
  const doc = new Y.Doc();

  if (!serverUrl) {
    return {
      roomId: normalizedRoomId,
      doc,
      provider: null,
    };
  }

  const provider = new WebsocketProvider(serverUrl, normalizedRoomId, doc, {
    connect: false,
  });

  return {
    roomId: normalizedRoomId,
    doc,
    provider,
  };
}

export function connectCollaborationConnection(connection: CollaborationConnection) {
  connection.provider?.connect();
}

export function destroyCollaborationConnection(connection: CollaborationConnection) {
  connection.provider?.disconnect();
  connection.doc.destroy();
}
