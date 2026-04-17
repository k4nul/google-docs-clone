import type { CollaborationUser } from '@/shared/types/collaboration';

const USER_COLORS = ['#0f8b8d', '#ff7f50', '#1d4ed8', '#d97706'];
const USER_NAMES = ['Atlas', 'Nova', 'Mina', 'Theo'];

export function createPlaceholderCollaborationUser(): CollaborationUser {
  const color = USER_COLORS[Math.floor(Math.random() * USER_COLORS.length)] ?? '#0f8b8d';
  const name = USER_NAMES[Math.floor(Math.random() * USER_NAMES.length)] ?? 'Atlas';

  return {
    id: typeof crypto !== 'undefined' ? crypto.randomUUID() : `${Date.now()}`,
    color,
    name,
  };
}
