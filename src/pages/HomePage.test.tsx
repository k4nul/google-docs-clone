import { render, screen } from '@testing-library/react';
import { MemoryRouter } from 'react-router-dom';
import { describe, expect, it } from 'vitest';

import { HomePage } from './HomePage';

describe('HomePage', () => {
  it('renders the document list placeholder and entry point', () => {
    render(
      <MemoryRouter>
        <HomePage />
      </MemoryRouter>,
    );

    expect(
      screen.getByRole('heading', { name: /collaborative document workspace/i }),
    ).toBeInTheDocument();
    expect(screen.getByRole('link', { name: /open mock editor/i })).toBeInTheDocument();
  });
});
