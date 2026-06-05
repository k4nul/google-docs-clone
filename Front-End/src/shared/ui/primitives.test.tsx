import { render, screen } from '@testing-library/react';
import { MemoryRouter } from 'react-router-dom';
import { describe, expect, it } from 'vitest';

import {
  Badge,
  Button,
  Card,
  EmptyState,
  ErrorState,
  LinkButton,
  LoadingState,
  SearchInput,
  Tooltip,
} from '@/shared/ui';

describe('shared UI primitives', () => {
  it('renders the shadcn-style action, card, search, and badge primitives', () => {
    render(
      <MemoryRouter>
        <Card aria-label="Document tools">
          <Button>Save</Button>
          <LinkButton to="/docs/doc-1">Open</LinkButton>
          <SearchInput label="Search" placeholder="Search documents" />
          <Badge tone="success">Autosave-ready</Badge>
        </Card>
      </MemoryRouter>,
    );

    expect(screen.getByLabelText('Document tools')).toHaveClass(
      'shadcn-card',
    );
    expect(screen.getByRole('button', { name: 'Save' })).toHaveClass(
      'shadcn-button',
    );
    expect(screen.getByRole('link', { name: 'Open' })).toHaveClass(
      'shadcn-button',
    );
    expect(screen.getByLabelText('Search')).toHaveClass('shadcn-input');
    expect(screen.getByText('Autosave-ready')).toHaveClass('shadcn-badge');
  });

  it('renders feedback, skeleton, and tooltip primitives with accessible text', () => {
    render(
      <>
        <EmptyState
          description="Create a document to start writing."
          title="No documents yet"
        />
        <ErrorState
          description="Try again to refresh your recent documents."
          title="Documents are temporarily unavailable"
        />
        <LoadingState title="Loading documents" />
        <Tooltip content="Bold">
          <button type="button">B</button>
        </Tooltip>
      </>,
    );

    expect(
      screen.getByRole('heading', { name: 'No documents yet' }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole('heading', {
        name: 'Documents are temporarily unavailable',
      }),
    ).toBeInTheDocument();
    expect(screen.getByText('Loading documents')).toHaveClass('sr-only');
    expect(screen.getByRole('tooltip', { name: 'Bold' })).toBeInTheDocument();
  });
});
