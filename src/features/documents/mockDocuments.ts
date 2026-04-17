import type { DocumentSummary } from '@/shared/types/document';

export const mockDocuments: DocumentSummary[] = [
  {
    id: 'launch-plan',
    title: 'Launch plan',
    summary: 'Cross-functional draft for release milestones, owner handoff, and collaborative review.',
    updatedAt: '2026-04-17T08:30:00.000Z',
    collaborators: 4,
    status: 'active',
  },
  {
    id: 'design-review-notes',
    title: 'Design review notes',
    summary: 'Living note for editor UX feedback, import gaps, and backend contract follow-ups.',
    updatedAt: '2026-04-15T11:45:00.000Z',
    collaborators: 2,
    status: 'draft',
  },
];
