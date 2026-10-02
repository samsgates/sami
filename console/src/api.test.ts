import { afterEach, describe, expect, it, vi } from 'vitest';
import { Api, ApiError, identifier, parseJson, records } from './api';

afterEach(() => vi.unstubAllGlobals());
describe('API contract', () => {
  it('attaches the provided credential and sends a real JSON request', async () => {
    const fetch = vi.fn().mockResolvedValue(new Response('{"id":"source_1"}', { status: 201 }));
    vi.stubGlobal('fetch', fetch);
    const result = await new Api('user-supplied').save('sources', { id: 'source_1' });
    expect(result).toEqual({ id: 'source_1' });
    expect(fetch.mock.calls[0][0]).toBe('/v1/admin/sources');
    expect(fetch.mock.calls[0][1].headers.Authorization).toBe('Bearer user-supplied');
    expect(fetch.mock.calls[0][1].body).toBe('{"id":"source_1"}');
  });
  it('does not silently retry a rejected write', async () => {
    const fetch = vi.fn().mockResolvedValue(new Response('{"error":{"message":"Approval expired"}}', { status: 409 }));
    vi.stubGlobal('fetch', fetch);
    await expect(new Api('').post('/actions/a/execute', {})).rejects.toEqual(expect.any(ApiError));
    expect(fetch).toHaveBeenCalledTimes(1);
  });
  it('normalizes list envelopes and validates identifiers', () => {
    expect(records({ items: [{ id: 'x' }] })).toEqual([{ id: 'x' }]);
    expect(records([{ id: 'y' }])).toEqual([{ id: 'y' }]);
    expect(identifier('a/b')).toBe('a%2Fb');
    expect(() => identifier('')).toThrow();
    expect(() => parseJson('{broken')).toThrow('valid JSON');
  });
});
