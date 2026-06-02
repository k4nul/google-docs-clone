import { cleanup, fireEvent, render, screen } from '@testing-library/react';
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

  it('renders fallback documents with user-facing unavailable messaging', async () => {
    vi.stubGlobal('fetch', vi.fn().mockRejectedValue(new Error('offline')));

    render(
      <MemoryRouter>
        <HomePage />
      </MemoryRouter>,
    );

    expect(
      screen.getByRole('heading', {
        name: /collaborative document workspace/i,
      }),
    ).toBeInTheDocument();
    expect(
      await screen.findByRole('heading', {
        name: /documents are temporarily unavailable/i,
      }),
    ).toBeInTheDocument();
    expect(screen.getByText(/offline/i)).toBeInTheDocument();
    expect(
      screen.getByRole('heading', { name: /moonlit recipe notes/i }),
    ).toBeInTheDocument();
    expect(
      screen.queryByRole('link', { name: /open sample editor/i }),
    ).not.toBeInTheDocument();
    expect(screen.queryByText(/using local sample/i)).not.toBeInTheDocument();
    expect(screen.queryByText(/source:/i)).not.toBeInTheDocument();
  });

  it('renders documents returned by the list API without source labels', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn().mockResolvedValue(
        jsonResponse({
          documents: [
            {
              id: '55555555-5555-4555-8555-555555555555',
              title: 'Budget notes',
              created_at: '2026-04-05T10:00:00.000Z',
              updated_at: '2026-04-05T10:30:00.000Z',
              preview: 'Quarterly planning notes and edits.',
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

    expect(
      await screen.findByRole('heading', { name: /budget notes/i }),
    ).toBeInTheDocument();
    expect(screen.getByText(/1 document/i)).toBeInTheDocument();
    expect(
      screen.getByText(/quarterly planning notes and edits/i),
    ).toBeInTheDocument();
    expect(
      screen.queryByRole('heading', { name: /moonlit recipe notes/i }),
    ).not.toBeInTheDocument();
    expect(screen.queryByText(/^Backend$/i)).not.toBeInTheDocument();
    expect(screen.queryByText(/source:/i)).not.toBeInTheDocument();
  });

  it('retries the document list through the same backend loading path', async () => {
    const fetchMock = vi
      .fn()
      .mockRejectedValueOnce(new Error('offline'))
      .mockResolvedValueOnce(
        jsonResponse({
          documents: [
            {
              id: '55555555-5555-4555-8555-555555555555',
              title: 'Budget notes',
              created_at: '2026-04-05T10:00:00.000Z',
              updated_at: '2026-04-05T10:30:00.000Z',
              preview: 'Quarterly planning notes and edits.',
            },
          ],
        }),
      );
    vi.stubGlobal('fetch', fetchMock);

    render(
      <MemoryRouter>
        <HomePage />
      </MemoryRouter>,
    );

    expect(
      await screen.findByRole('heading', {
        name: /documents are temporarily unavailable/i,
      }),
    ).toBeInTheDocument();

    fireEvent.click(screen.getByRole('button', { name: /retry/i }));

    expect(
      await screen.findByRole('heading', { name: /budget notes/i }),
    ).toBeInTheDocument();
    expect(screen.getByText(/1 document/i)).toBeInTheDocument();
    expect(
      screen.queryByRole('heading', {
        name: /documents are temporarily unavailable/i,
      }),
    ).not.toBeInTheDocument();
    expect(fetchMock).toHaveBeenCalledTimes(2);
  });
});
