export interface DocumentSummary {
  id: string;
  title: string;
  summary: string;
  updatedAt: string;
  collaborators: number;
  status: 'active' | 'draft';
}
