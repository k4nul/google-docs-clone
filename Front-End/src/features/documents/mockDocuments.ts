import type { DocumentSummary } from '@/shared/types/document';

export const mockDocuments: DocumentSummary[] = [
  {
    id: '56420a3e-fb8d-4e5c-b0b5-7425affa7e71',
    title: 'Moonlit recipe notes',
    summary:
      'A quiet draft about cardamom tea, late-night revisions, and a kitchen window full of rain.',
    updatedAt: '2026-04-17T08:30:00.000Z',
    collaborators: 4,
  },
  {
    id: '608803d8-36af-465b-94c2-2dfee2efb790',
    title: 'Garden story outline',
    summary:
      'Scene sketches for a neighborly mystery told through seed packets and handwritten notes.',
    updatedAt: '2026-04-15T11:45:00.000Z',
    collaborators: 2,
  },
];
