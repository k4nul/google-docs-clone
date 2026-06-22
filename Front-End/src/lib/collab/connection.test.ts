import * as Y from 'yjs';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import {
  BinaryWebsocketProvider,
  connectCollaborationConnection,
  createCollaborationConnection,
  redactAccessToken,
  scheduleCollaborationConnectionDestroy,
} from './connection';
import type { ProviderConnectionStatus } from './connection';

class FakeWebSocket {
  static readonly CONNECTING = 0;
  static readonly OPEN = 1;
  static readonly CLOSING = 2;
  static readonly CLOSED = 3;
  static instances: FakeWebSocket[] = [];

  binaryType: BinaryType = 'blob';
  onclose: ((this: WebSocket, ev: CloseEvent) => unknown) | null = null;
  onerror: ((this: WebSocket, ev: Event) => unknown) | null = null;
  onmessage:
    | ((this: WebSocket, ev: MessageEvent<ArrayBuffer>) => unknown)
    | null = null;
  onopen: ((this: WebSocket, ev: Event) => unknown) | null = null;
  readonly sentMessages: Array<Parameters<WebSocket['send']>[0]> = [];
  readyState = FakeWebSocket.CONNECTING;
  readonly url: string;

  constructor(url: string | URL) {
    this.url = String(url);
    FakeWebSocket.instances.push(this);
  }

  send(data: Parameters<WebSocket['send']>[0]) {
    this.sentMessages.push(data);
  }

  close() {
    this.readyState = FakeWebSocket.CLOSING;
  }

  open() {
    this.readyState = FakeWebSocket.OPEN;
    this.onopen?.call(this.asWebSocket(), new Event('open'));
  }

  closeFromServer({
    code = 1000,
    reason = '',
    wasClean = true,
  }: Partial<CloseEvent> = {}) {
    if (this.readyState === FakeWebSocket.CLOSED) {
      return;
    }

    this.readyState = FakeWebSocket.CLOSED;
    this.onclose?.call(this.asWebSocket(), {
      code,
      reason,
      wasClean,
    } as CloseEvent);
  }

  private asWebSocket() {
    return this as unknown as WebSocket;
  }
}

function latestFakeWebSocket() {
  const socket = FakeWebSocket.instances.at(-1);

  if (!socket) {
    throw new Error('Expected a websocket to be created.');
  }

  return socket;
}

describe('collaboration connection lifecycle', () => {
  const providers: BinaryWebsocketProvider[] = [];

  function createProvider(
    serverUrl: string,
    roomId: string,
    doc: Y.Doc,
    accessToken?: string | null,
  ) {
    const provider = new BinaryWebsocketProvider(
      serverUrl,
      roomId,
      doc,
      accessToken,
    );
    providers.push(provider);
    return provider;
  }

  beforeEach(() => {
    FakeWebSocket.instances = [];
    vi.stubGlobal('WebSocket', FakeWebSocket);
    vi.spyOn(console, 'info').mockImplementation(() => undefined);
  });

  afterEach(() => {
    for (const provider of providers.splice(0)) {
      provider.destroy();
    }
    vi.useRealTimers();
    vi.restoreAllMocks();
    vi.unstubAllGlobals();
    FakeWebSocket.instances = [];
  });

  it('keeps the Yjs document alive when a scheduled destroy is canceled by reconnect', () => {
    vi.useFakeTimers();
    const connection = createCollaborationConnection({
      roomId: 'strict-mode-room',
      serverUrl: null,
    });
    const updateHandler = vi.fn();

    connection.doc.on('update', updateHandler);
    scheduleCollaborationConnectionDestroy(connection);
    connectCollaborationConnection(connection);
    vi.runAllTimers();

    connection.doc.getText('content').insert(0, 'still-live');

    expect(connection.destroyed).toBe(false);
    expect(updateHandler).toHaveBeenCalledTimes(1);
  });

  it('destroys the Yjs document when the scheduled cleanup is not canceled', () => {
    vi.useFakeTimers();
    const connection = createCollaborationConnection({
      roomId: 'unmounted-room',
      serverUrl: null,
    });
    const destroyHandler = vi.fn();

    connection.doc.on('destroy', destroyHandler);
    scheduleCollaborationConnectionDestroy(connection);
    vi.runAllTimers();

    expect(connection.destroyed).toBe(true);
    expect(destroyHandler).toHaveBeenCalledTimes(1);
  });

  it('adds the document access token to websocket provider URLs', () => {
    const connection = createCollaborationConnection({
      accessToken: 'doc-token',
      roomId: 'credentialed-room',
      serverUrl: 'ws://localhost:4000',
    });

    expect(connection.provider?.url).toBe(
      'ws://localhost:4000/ws/credentialed-room?access_token=doc-token',
    );
    expect(redactAccessToken(connection.provider?.url ?? '')).toBe(
      'ws://localhost:4000/ws/credentialed-room?access_token=%5Bredacted%5D',
    );
  });

  it('opens websocket providers and periodically resyncs while connected', () => {
    vi.useFakeTimers();
    const provider = createProvider(
      'ws://localhost:4000',
      'team room',
      new Y.Doc(),
      'doc-token',
    );
    const statuses: ProviderConnectionStatus[] = [];

    provider.onStatusChange((status) => {
      statuses.push(status);
    });
    provider.connect();

    const socket = latestFakeWebSocket();

    expect(socket.url).toBe(
      'ws://localhost:4000/ws/team%20room?access_token=doc-token',
    );
    expect(socket.binaryType).toBe('arraybuffer');
    expect(statuses).toEqual(['disconnected', 'connecting']);

    socket.open();

    expect(provider.wsconnected).toBe(true);
    expect(provider.getStatus()).toBe('connected');
    expect(statuses).toEqual(['disconnected', 'connecting', 'connected']);
    expect(socket.sentMessages).toHaveLength(2);
    expect(socket.sentMessages[0]).toBeInstanceOf(Uint8Array);

    vi.advanceTimersByTime(10_000);

    expect(socket.sentMessages).toHaveLength(3);
  });

  it('schedules a reconnect when an active websocket closes unexpectedly', () => {
    vi.useFakeTimers();
    const provider = createProvider(
      'ws://localhost:4000/',
      'reconnect-room',
      new Y.Doc(),
    );
    const statuses: ProviderConnectionStatus[] = [];

    provider.onStatusChange((status) => {
      statuses.push(status);
    });
    provider.connect();
    latestFakeWebSocket().open();

    latestFakeWebSocket().closeFromServer({
      code: 1006,
      reason: 'network lost',
      wasClean: false,
    });

    expect(provider.ws).toBeNull();
    expect(provider.wsconnected).toBe(false);
    expect(provider.getStatus()).toBe('reconnecting');
    expect(FakeWebSocket.instances).toHaveLength(1);

    vi.advanceTimersByTime(1_499);

    expect(FakeWebSocket.instances).toHaveLength(1);

    vi.advanceTimersByTime(1);

    expect(FakeWebSocket.instances).toHaveLength(2);
    expect(latestFakeWebSocket().url).toBe(
      'ws://localhost:4000/ws/reconnect-room',
    );
    expect(statuses).toEqual([
      'disconnected',
      'connecting',
      'connected',
      'reconnecting',
      'connecting',
    ]);
  });

  it('stops sending local document updates after provider destroy', () => {
    const doc = new Y.Doc();
    const provider = createProvider(
      'ws://localhost:4000',
      'destroy-room',
      doc,
    );

    provider.connect();
    latestFakeWebSocket().open();

    const socket = latestFakeWebSocket();

    doc.getText('content').insert(0, 'live');

    expect(socket.sentMessages).toHaveLength(3);

    provider.destroy();
    const sentBeforeStaleUpdate = socket.sentMessages.length;
    doc.getText('content').insert(4, ' stale');

    expect(socket.sentMessages).toHaveLength(sentBeforeStaleUpdate);
  });
});
