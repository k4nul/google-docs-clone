import { afterEach, describe, expect, it, vi } from 'vitest';

import { ApiRequestError, apiGet, apiPost, buildApiUrl } from './httpClient';

function jsonResponse(body: unknown, init?: ResponseInit) {
  return new Response(JSON.stringify(body), {
    status: 200,
    ...init,
    headers: {
      'Content-Type': 'application/json',
      ...(init?.headers ?? {}),
    },
  });
}

function textResponse(body: string, init?: ResponseInit) {
  return new Response(body, init);
}

describe('HTTP API client', () => {
  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it('builds runtime API URLs from the browser location', () => {
    expect(buildApiUrl('/documents')).toBe(
      'http://localhost:3000/api/documents',
    );
    expect(buildApiUrl('/documents/first%20draft')).toBe(
      'http://localhost:3000/api/documents/first%20draft',
    );
  });

  it('GETs JSON resources through the runtime API base URL', async () => {
    const fetchMock = vi.fn().mockResolvedValue(jsonResponse({ ok: true }));
    vi.stubGlobal('fetch', fetchMock);

    await expect(apiGet<{ ok: boolean }>('/documents')).resolves.toEqual({
      ok: true,
    });

    expect(fetchMock).toHaveBeenCalledWith(
      expect.stringContaining('/api/documents'),
      expect.objectContaining({
        headers: {
          Accept: 'application/json',
        },
      }),
    );
  });

  it('POSTs JSON bodies while preserving caller-provided headers', async () => {
    const fetchMock = vi
      .fn()
      .mockResolvedValue(jsonResponse({ created: true }));
    vi.stubGlobal('fetch', fetchMock);

    await expect(
      apiPost<{ created: boolean }>(
        '/documents',
        { title: 'Realtime draft' },
        {
          headers: {
            Authorization: 'Bearer test-token',
          },
        },
      ),
    ).resolves.toEqual({ created: true });

    expect(fetchMock).toHaveBeenCalledWith(
      expect.stringContaining('/api/documents'),
      expect.objectContaining({
        method: 'POST',
        body: JSON.stringify({ title: 'Realtime draft' }),
        headers: {
          Accept: 'application/json',
          'Content-Type': 'application/json',
          Authorization: 'Bearer test-token',
        },
      }),
    );
  });

  it('throws ApiRequestError with JSON payload details for failed requests', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn().mockResolvedValue(
        jsonResponse(
          {
            error: 'document_not_found',
            message: 'Document was not found',
          },
          { status: 404 },
        ),
      ),
    );

    await expect(apiGet('/documents/missing')).rejects.toMatchObject({
      name: 'ApiRequestError',
      status: 404,
      payload: {
        error: 'document_not_found',
        message: 'Document was not found',
      },
      message: 'Document was not found',
    });
  });

  it('throws ApiRequestError with a null payload for non-JSON failures', async () => {
    vi.stubGlobal(
      'fetch',
      vi
        .fn()
        .mockResolvedValue(
          textResponse('service unavailable', { status: 503 }),
        ),
    );

    await expect(apiGet('/documents')).rejects.toMatchObject({
      name: 'ApiRequestError',
      status: 503,
      payload: null,
      message: 'API request failed: 503',
    });
  });

  it('throws ApiRequestError with a null payload when error JSON cannot be parsed', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn().mockResolvedValue(
        textResponse('not json', {
          status: 502,
          headers: {
            'Content-Type': 'application/json',
          },
        }),
      ),
    );
    const request = apiGet('/documents');

    await expect(request).rejects.toBeInstanceOf(ApiRequestError);
    await expect(request).rejects.toMatchObject({
      status: 502,
      payload: null,
      message: 'API request failed: 502',
    });
  });
});
