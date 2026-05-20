import { cleanup, render, screen } from '@testing-library/react';
import { MemoryRouter } from 'react-router-dom';
import { afterEach, describe, expect, it, vi } from 'vitest';

import { HomePage } from './HomePage';

function jsonResponse(body: unknown) {
  return new Response(JSON.stringify(body), {
    status: 200,
    headers: {
      'Content-Type': 'application/json',
    },
  });
}

describe('HomePage', () => {
  afterEach(() => {
    cleanup();
    vi.unstubAllGlobals();
  });

  it('renders local sample documents when the backend list is unavailable', async () => {
    vi.stubGlobal('fetch', vi.fn().mockRejectedValue(new Error('offline')));

    render(
      <MemoryRouter>
        <HomePage />
      </MemoryRouter>,
    );

    expect(
      screen.getByRole('heading', { name: /collaborative document workspace/i }),
    ).toBeInTheDocument();
    expect(screen.getByRole('link', { name: /open sample editor/i })).toBeInTheDocument();
    expect(await screen.findByText(/backend list unavailable: offline/i)).toBeInTheDocument();
  });

  it('renders backend documents returned by the list API', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn().mockResolvedValue(
        jsonResponse({
          documents: [
            {
              id: '55555555-5555-4555-8555-555555555555',
              title: 'Backend launch notes',
              created_at: '2026-04-05T10:00:00.000Z',
              updated_at: '2026-04-05T10:30:00.000Z',
            },
          ],
        }),
      ),
    );

    render(
      <MemoryRouter>
        <HomePage />
      </MemoryRouter>,
    );

    expect(await screen.findByRole('heading', { name: /backend launch notes/i })).toBeInTheDocument();
    expect(screen.getByText(/1 backend document/i)).toBeInTheDocument();
    expect(screen.queryByRole('heading', { name: /launch plan/i })).not.toBeInTheDocument();
  });
});
