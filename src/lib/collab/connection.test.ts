import { afterEach, describe, expect, it, vi } from 'vitest';

import {
  connectCollaborationConnection,
  createCollaborationConnection,
  scheduleCollaborationConnectionDestroy,
} from './connection';

describe('collaboration connection lifecycle', () => {
  afterEach(() => {
    vi.useRealTimers();
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
});
