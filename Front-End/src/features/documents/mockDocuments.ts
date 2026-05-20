import type { DocumentSummary } from '@/shared/types/document';

export const mockDocuments: DocumentSummary[] = [
  {
    id: '56420a3e-fb8d-4e5c-b0b5-7425affa7e71',
    title: 'Launch plan',
    summary: 'Cross-functional draft for release milestones, owner handoff, and collaborative review.',
    updatedAt: '2026-04-17T08:30:00.000Z',
    collaborators: 4,
    status: 'active',
    source: 'sample',
  },
  {
    id: '608803d8-36af-465b-94c2-2dfee2efb790',
    title: 'Design review notes',
    summary: 'Living note for editor UX feedback, import gaps, and backend contract follow-ups.',
    updatedAt: '2026-04-15T11:45:00.000Z',
    collaborators: 2,
    status: 'draft',
    source: 'sample',
  },
];
