import * as awarenessProtocol from 'y-protocols/awareness';
import * as syncProtocol from 'y-protocols/sync';
import * as decoding from 'lib0/decoding';
import * as encoding from 'lib0/encoding';
import * as Y from 'yjs';

const MSG_SYNC = 0;
const MSG_AWARENESS = 1;
const MSG_AWARENESS_QUERY = 3;

export type ProviderConnectionStatus =
  | 'local-only'
  | 'connecting'
  | 'connected'
  | 'reconnecting'
  | 'disconnected';

function buildWsEndpoint(serverUrl: string, roomId: string) {
  const normalizedBaseUrl = serverUrl.replace(/\/+$/, '');
  const endpoint = new URL(`/ws/${encodeURIComponent(roomId)}`, `${normalizedBaseUrl}/`);

  return endpoint.toString();
}

function logConnectionEvent(message: string, details?: Record<string, unknown>) {
  if (details) {
    console.info(`[collab] ${message}`, details);
    return;
  }

  console.info(`[collab] ${message}`);
}

function sendAwarenessUpdate(provider: BinaryWebsocketProvider, clientIds: number[]) {
  if (!provider.ws || provider.ws.readyState !== WebSocket.OPEN || clientIds.length === 0) {
    return;
  }

  const encoder = encoding.createEncoder();
  encoding.writeVarUint(encoder, MSG_AWARENESS);
  encoding.writeVarUint8Array(
    encoder,
    awarenessProtocol.encodeAwarenessUpdate(provider.awareness, clientIds),
  );
  provider.ws.send(encoding.toUint8Array(encoder));
}

export class BinaryWebsocketProvider {
  readonly awareness: awarenessProtocol.Awareness;
  readonly doc: Y.Doc;
  readonly roomId: string;
  readonly serverUrl: string;

  private connectionStatus: ProviderConnectionStatus = 'disconnected';
  private destroyed = false;
  private readonly statusListeners = new Set<(status: ProviderConnectionStatus) => void>();
  private reconnectTimer: number | null = null;
  private readonly awarenessUpdateHandler: (
    changes: { added: number[]; updated: number[]; removed: number[] },
    origin: unknown,
  ) => void;
  private readonly docUpdateHandler: (update: Uint8Array, origin: unknown) => void;

  ws: WebSocket | null = null;
  wsconnected = false;
  wsconnecting = false;
  shouldConnect = false;

  constructor(serverUrl: string, roomId: string, doc: Y.Doc) {
    this.serverUrl = serverUrl;
    this.roomId = roomId;
    this.doc = doc;
    this.awareness = new awarenessProtocol.Awareness(doc);

    this.docUpdateHandler = (update, origin) => {
      if (origin === this || !this.ws || this.ws.readyState !== WebSocket.OPEN) {
        return;
      }

      const encoder = encoding.createEncoder();
      encoding.writeVarUint(encoder, MSG_SYNC);
      syncProtocol.writeUpdate(encoder, update);
      this.ws.send(encoding.toUint8Array(encoder));
    };

    this.awarenessUpdateHandler = ({ added, updated, removed }, origin) => {
      if (origin === this) {
        return;
      }

      sendAwarenessUpdate(this, added.concat(updated).concat(removed));
    };

    this.doc.on('update', this.docUpdateHandler);
    this.awareness.on('update', this.awarenessUpdateHandler);
  }

  private setConnectionStatus(status: ProviderConnectionStatus) {
    this.connectionStatus = status;
    for (const listener of this.statusListeners) {
      listener(status);
    }
  }

  getStatus() {
    return this.connectionStatus;
  }

  onStatusChange(listener: (status: ProviderConnectionStatus) => void) {
    this.statusListeners.add(listener);
    listener(this.connectionStatus);

    return () => {
      this.statusListeners.delete(listener);
    };
  }

  connect() {
    if (this.destroyed || this.wsconnecting || this.wsconnected) {
      return;
    }

    this.shouldConnect = true;
    this.wsconnecting = true;
    this.setConnectionStatus(this.reconnectTimer === null ? 'connecting' : 'reconnecting');

    const endpoint = buildWsEndpoint(this.serverUrl, this.roomId);
    logConnectionEvent('websocket connect requested', {
      endpoint,
      roomId: this.roomId,
      status: this.getStatus(),
    });

    const ws = new WebSocket(endpoint);
    ws.binaryType = 'arraybuffer';
    this.ws = ws;

    ws.onopen = () => {
      if (this.destroyed || !this.shouldConnect) {
        logConnectionEvent('websocket opened after cleanup, closing stale socket', {
          endpoint,
          roomId: this.roomId,
        });
        ws.close();
        return;
      }

      this.wsconnecting = false;
      this.wsconnected = true;
      this.setConnectionStatus('connected');
      logConnectionEvent('websocket connected', {
        endpoint,
        roomId: this.roomId,
      });

      const encoder = encoding.createEncoder();
      encoding.writeVarUint(encoder, MSG_SYNC);
      syncProtocol.writeSyncStep1(encoder, this.doc);
      ws.send(encoding.toUint8Array(encoder));

      const localState = this.awareness.getLocalState();
      if (localState) {
        sendAwarenessUpdate(this, [this.doc.clientID]);
      }
    };

    ws.onmessage = (event) => {
      if (this.destroyed) {
        return;
      }

      if (!(event.data instanceof ArrayBuffer)) {
        return;
      }

      const data = new Uint8Array(event.data);
      const decoder = decoding.createDecoder(data);
      const messageType = decoding.readVarUint(decoder);

      if (messageType === MSG_SYNC) {
        const reply = encoding.createEncoder();
        encoding.writeVarUint(reply, MSG_SYNC);
        syncProtocol.readSyncMessage(decoder, reply, this.doc, this);

        const replyBytes = encoding.toUint8Array(reply);
        if (replyBytes.length > 1 && ws.readyState === WebSocket.OPEN) {
          ws.send(replyBytes);
        }
        return;
      }

      if (messageType === MSG_AWARENESS) {
        const update = decoding.readVarUint8Array(decoder);
        awarenessProtocol.applyAwarenessUpdate(this.awareness, update, this);
        return;
      }

      if (messageType === MSG_AWARENESS_QUERY) {
        sendAwarenessUpdate(this, Array.from(this.awareness.getStates().keys()));
      }
    };

    ws.onclose = (event) => {
      this.wsconnected = false;
      this.wsconnecting = false;
      if (this.ws === ws) {
        this.ws = null;
      }
      logConnectionEvent('websocket closed', {
        code: event.code,
        endpoint,
        roomId: this.roomId,
        wasClean: event.wasClean,
        willReconnect: this.shouldConnect,
        ...(event.reason ? { reason: event.reason } : {}),
      });
      awarenessProtocol.removeAwarenessStates(this.awareness, Array.from(this.awareness.getStates().keys()), this);

      if (this.shouldConnect && !this.destroyed) {
        this.setConnectionStatus('reconnecting');
        this.reconnectTimer = window.setTimeout(() => {
          this.reconnectTimer = null;
          this.connect();
        }, 1500);
        return;
      }

      this.setConnectionStatus('disconnected');
    };

    ws.onerror = () => {
      logConnectionEvent('websocket error', {
        endpoint,
        roomId: this.roomId,
        readyState: ws.readyState,
      });
      if (ws.readyState === WebSocket.OPEN || ws.readyState === WebSocket.CONNECTING) {
        ws.close();
      }
    };
  }

  disconnect() {
    this.shouldConnect = false;
    if (this.reconnectTimer !== null) {
      window.clearTimeout(this.reconnectTimer);
      this.reconnectTimer = null;
    }
    if (this.ws?.readyState === WebSocket.CONNECTING) {
      const pendingSocket = this.ws;
      pendingSocket.onopen = () => {
        pendingSocket.close();
      };
      pendingSocket.onmessage = null;
      pendingSocket.onerror = null;
      return;
    }

    this.ws?.close();
    if (!this.wsconnected && !this.wsconnecting) {
      this.setConnectionStatus('disconnected');
    }
  }

  destroy() {
    this.destroyed = true;
    this.disconnect();
    this.doc.off('update', this.docUpdateHandler);
    this.awareness.off('update', this.awarenessUpdateHandler);
    awarenessProtocol.removeAwarenessStates(this.awareness, [this.doc.clientID], this);
  }
}

export interface CollaborationConnection {
  roomId: string;
  doc: Y.Doc;
  provider: BinaryWebsocketProvider | null;
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

  const provider = new BinaryWebsocketProvider(serverUrl, normalizedRoomId, doc);

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
  connection.provider?.destroy();
  connection.doc.destroy();
}
