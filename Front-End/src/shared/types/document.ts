export interface BackendDocument {
  id: string;
  title: string;
  createdAt: string;
  updatedAt: string;
  hidePreview: boolean;
}

export interface DocumentSummary {
  id: string;
  title: string;
  summary: string;
  updatedAt: string;
  createdAt?: string;
  collaborators: number;
}
