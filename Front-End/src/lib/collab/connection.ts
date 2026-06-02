import * as awarenessProtocol from 'y-protocols/awareness';
import * as syncProtocol from 'y-protocols/sync';
import * as decoding from 'lib0/decoding';
import * as encoding from 'lib0/encoding';
import * as Y from 'yjs';

const MSG_SYNC = 0;
const MSG_AWARENESS = 1;
const MSG_AWARENESS_QUERY = 3;
const PROVIDER_RECONNECT_DELAY_MS = 1_500;
const PROVIDER_RESYNC_INTERVAL_MS = 10_000;

export type ProviderConnectionStatus =
  | 'local-only'
  | 'connecting'
  | 'connected'
  | 'reconnecting'
  | 'disconnected';

function buildWsEndpoint(
  serverUrl: string,
  roomId: string,
  accessToken?: string | null,
) {
  const normalizedBaseUrl = serverUrl.replace(/\/+$/, '');
  const endpoint = new URL(
    `/ws/${encodeURIComponent(roomId)}`,
    `${normalizedBaseUrl}/`,
  );

  if (accessToken) {
    endpoint.searchParams.set('access_token', accessToken);
  }

  return endpoint.toString();
}

function logConnectionEvent(
  message: string,
  details?: Record<string, unknown>,
) {
  if (details) {
    console.info(`[collab] ${message}`, details);
    return;
  }

  console.info(`[collab] ${message}`);
}

export function redactAccessToken(value: string) {
  try {
    const url = new URL(value);

    if (url.searchParams.has('access_token')) {
      url.searchParams.set('access_token', '[redacted]');
    }

    return url.toString();
  } catch {
    return value.replace(/([?&]access_token=)[^&]+/i, '$1[redacted]');
  }
}

function sendAwarenessUpdate(
  provider: BinaryWebsocketProvider,
  clientIds: number[],
) {
  if (
    !provider.ws ||
    provider.ws.readyState !== WebSocket.OPEN ||
    clientIds.length === 0
  ) {
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
  readonly url: string;

  private connectionStatus: ProviderConnectionStatus = 'disconnected';
  private destroyed = false;
  private readonly statusListeners = new Set<
    (status: ProviderConnectionStatus) => void
  >();
  private reconnectTimer: number | null = null;
  private resyncTimer: number | null = null;
  private readonly awarenessUpdateHandler: (
    changes: { added: number[]; updated: number[]; removed: number[] },
    origin: unknown,
  ) => void;
  private readonly docUpdateHandler: (
    update: Uint8Array,
    origin: unknown,
  ) => void;

  ws: WebSocket | null = null;
  wsconnected = false;
  wsconnecting = false;
  shouldConnect = false;
  synced = false;

  constructor(
    serverUrl: string,
    roomId: string,
    doc: Y.Doc,
    accessToken?: string | null,
  ) {
    this.serverUrl = serverUrl;
    this.roomId = roomId;
    this.doc = doc;
    this.url = buildWsEndpoint(serverUrl, roomId, accessToken);
    this.awareness = new awarenessProtocol.Awareness(doc);

    this.docUpdateHandler = (update, origin) => {
      if (
        origin === this ||
        !this.ws ||
        this.ws.readyState !== WebSocket.OPEN
      ) {
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

  private sendSyncStep1(ws: WebSocket) {
    const encoder = encoding.createEncoder();
    encoding.writeVarUint(encoder, MSG_SYNC);
    syncProtocol.writeSyncStep1(encoder, this.doc);
    ws.send(encoding.toUint8Array(encoder));
  }

  private startResync() {
    this.stopResync();
    this.resyncTimer = window.setInterval(() => {
      if (this.ws?.readyState === WebSocket.OPEN) {
        this.sendSyncStep1(this.ws);
      }
    }, PROVIDER_RESYNC_INTERVAL_MS);
  }

  private stopResync() {
    if (this.resyncTimer === null) {
      return;
    }

    window.clearInterval(this.resyncTimer);
    this.resyncTimer = null;
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
    this.synced = false;
    this.setConnectionStatus(
      this.reconnectTimer === null ? 'connecting' : 'reconnecting',
    );
    logConnectionEvent('websocket connect requested', {
      endpoint: redactAccessToken(this.url),
      roomId: this.roomId,
      status: this.getStatus(),
    });

    const ws = new WebSocket(this.url);
    ws.binaryType = 'arraybuffer';
    this.ws = ws;

    ws.onopen = () => {
      if (this.destroyed || !this.shouldConnect) {
        logConnectionEvent(
          'websocket opened after cleanup, closing stale socket',
          {
            endpoint: redactAccessToken(this.url),
            roomId: this.roomId,
          },
        );
        ws.close();
        return;
      }

      this.wsconnecting = false;
      this.wsconnected = true;
      this.setConnectionStatus('connected');
      this.startResync();
      logConnectionEvent('websocket connected', {
        endpoint: redactAccessToken(this.url),
        roomId: this.roomId,
      });

      this.sendSyncStep1(ws);

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
        this.synced = true;

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
        sendAwarenessUpdate(
          this,
          Array.from(this.awareness.getStates().keys()),
        );
      }
    };

    ws.onclose = (event) => {
      this.stopResync();
      this.synced = false;
      this.wsconnected = false;
      this.wsconnecting = false;
      if (this.ws === ws) {
        this.ws = null;
      }
      logConnectionEvent('websocket closed', {
        code: event.code,
        endpoint: redactAccessToken(this.url),
        roomId: this.roomId,
        wasClean: event.wasClean,
        willReconnect: this.shouldConnect,
        ...(event.reason ? { reason: event.reason } : {}),
      });
      awarenessProtocol.removeAwarenessStates(
        this.awareness,
        Array.from(this.awareness.getStates().keys()),
        this,
      );

      if (this.shouldConnect && !this.destroyed) {
        this.setConnectionStatus('reconnecting');
        this.reconnectTimer = window.setTimeout(() => {
          this.reconnectTimer = null;
          this.connect();
        }, PROVIDER_RECONNECT_DELAY_MS);
        return;
      }

      this.setConnectionStatus('disconnected');
    };

    ws.onerror = () => {
      logConnectionEvent('websocket error', {
        endpoint: redactAccessToken(this.url),
        roomId: this.roomId,
        readyState: ws.readyState,
      });
      if (
        ws.readyState === WebSocket.OPEN ||
        ws.readyState === WebSocket.CONNECTING
      ) {
        ws.close();
      }
    };
  }

  disconnect() {
    this.shouldConnect = false;
    this.stopResync();
    this.synced = false;
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
    if (this.destroyed) {
      return;
    }

    this.destroyed = true;
    this.disconnect();
    this.doc.off('update', this.docUpdateHandler);
    this.awareness.off('update', this.awarenessUpdateHandler);
    awarenessProtocol.removeAwarenessStates(
      this.awareness,
      [this.doc.clientID],
      this,
    );
  }
}

export interface CollaborationConnection {
  roomId: string;
  doc: Y.Doc;
  provider: BinaryWebsocketProvider | null;
  destroyTimeout: ReturnType<typeof setTimeout> | null;
  destroyed: boolean;
}

interface CreateCollaborationConnectionParams {
  accessToken?: string | null;
  roomId: string;
  serverUrl: string | null;
}

export function createCollaborationConnection({
  accessToken = null,
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
      destroyTimeout: null,
      destroyed: false,
    };
  }

  const provider = new BinaryWebsocketProvider(
    serverUrl,
    normalizedRoomId,
    doc,
    accessToken,
  );

  return {
    roomId: normalizedRoomId,
    doc,
    provider,
    destroyTimeout: null,
    destroyed: false,
  };
}

export function connectCollaborationConnection(
  connection: CollaborationConnection,
) {
  cancelScheduledCollaborationConnectionDestroy(connection);
  connection.provider?.connect();
}

export function destroyCollaborationConnection(
  connection: CollaborationConnection,
) {
  if (connection.destroyed) {
    return;
  }

  cancelScheduledCollaborationConnectionDestroy(connection);
  connection.provider?.destroy();
  connection.doc.destroy();
  connection.destroyed = true;
}

export function scheduleCollaborationConnectionDestroy(
  connection: CollaborationConnection,
) {
  if (connection.destroyed || connection.destroyTimeout) {
    return;
  }

  connection.destroyTimeout = setTimeout(() => {
    connection.destroyTimeout = null;
    destroyCollaborationConnection(connection);
  }, 0);
}

export function cancelScheduledCollaborationConnectionDestroy(
  connection: CollaborationConnection,
) {
  if (!connection.destroyTimeout) {
    return;
  }

  clearTimeout(connection.destroyTimeout);
  connection.destroyTimeout = null;
}
