import type { CollaborationUser } from '@/shared/types/collaboration';

const USER_COLORS = ['#0f8b8d', '#ff7f50', '#1d4ed8', '#d97706'];
const USER_NAMES = ['Atlas', 'Nova', 'Mina', 'Theo'];

function createFallbackUserId() {
  return `user-${Date.now().toString(36)}-${Math.random().toString(36).slice(2, 10)}`;
}

function createCollaborationUserId() {
  if (typeof globalThis.crypto?.randomUUID === 'function') {
    return globalThis.crypto.randomUUID();
  }

  return createFallbackUserId();
}

export function createPlaceholderCollaborationUser(): CollaborationUser {
  const color =
    USER_COLORS[Math.floor(Math.random() * USER_COLORS.length)] ?? '#0f8b8d';
  const name =
    USER_NAMES[Math.floor(Math.random() * USER_NAMES.length)] ?? 'Atlas';

  return {
    id: createCollaborationUserId(),
    color,
    name,
  };
}
