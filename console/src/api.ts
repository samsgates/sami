export type Json = null | boolean | number | string | Json[] | { [key: string]: Json };
export type RecordValue = { [key: string]: Json };

export class ApiError extends Error {
  constructor(public readonly status: number, message: string, public readonly details: unknown) {
    super(message);
    this.name = 'ApiError';
  }
}

export function parseJson(value: string): Json {
  try { return JSON.parse(value) as Json; }
  catch { throw new Error('Enter valid JSON. Check commas, quotation marks and closing brackets.'); }
}

export function objectValue(value: unknown): RecordValue {
  return value !== null && typeof value === 'object' && !Array.isArray(value) ? value as RecordValue : {};
}

export function records(value: unknown): RecordValue[] {
  const obj = objectValue(value);
  const data = Array.isArray(value) ? value : obj.items ?? obj.records ?? obj.data ?? [];
  return Array.isArray(data) ? data.map(objectValue) : [];
}

export function pretty(value: unknown): string { return JSON.stringify(value, null, 2); }

export function identifier(value: Json | undefined): string {
  if (typeof value !== 'string' || value.trim() === '') throw new Error('A record identifier is required.');
  return encodeURIComponent(value);
}

export class Api {
  constructor(private readonly token: string, private readonly base = '/v1') {}

  async request<T = Json>(method: string, path: string, body?: Json, timeoutMs = 30_000, idempotencyKey?: string): Promise<T> {
    const controller = new AbortController();
    const timeout = setTimeout(() => controller.abort(), timeoutMs);
    try {
      const headers: Record<string, string> = { Accept: 'application/json' };
      if (this.token.trim()) headers.Authorization = `Bearer ${this.token.trim()}`;
      if (body !== undefined) headers['Content-Type'] = 'application/json';
      if (idempotencyKey) headers['Idempotency-Key'] = idempotencyKey;
      const response = await fetch(`${this.base}${path}`, {
        method,
        headers,
        body: body === undefined ? undefined : JSON.stringify(body),
        signal: controller.signal,
        credentials: 'same-origin',
      });
      const raw = await response.text();
      let value: unknown = null;
      if (raw) { try { value = JSON.parse(raw); } catch { value = { message: raw.slice(0, 500) }; } }
      if (!response.ok) {
        const obj = objectValue(value);
        const error = objectValue(obj.error);
        const message = String(error.message ?? obj.message ?? `Request failed (${response.status}).`);
        throw new ApiError(response.status, message, value);
      }
      return value as T;
    } catch (error) {
      if (error instanceof Error && error.name === 'AbortError') {
        throw new Error('Request timed out. For a write, inspect its state before retrying: the server may have completed it.');
      }
      throw error;
    } finally { clearTimeout(timeout); }
  }

  get(path: string) { return this.request('GET', path); }
  post(path: string, body: Json) { return this.request('POST', path, body); }
  delete(path: string) { return this.request('DELETE', path); }
  list(kind: string) { return this.get(`/admin/${encodeURIComponent(kind)}`); }
  save(kind: string, body: Json) { return this.post(`/admin/${encodeURIComponent(kind)}`, body); }
}
