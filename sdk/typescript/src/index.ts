export type Json = null | string | number | boolean | Json[] | { [key: string]: Json };
export type JsonObject = { [key: string]: Json };
export class SamiError extends Error {
  constructor(message: string, public readonly status: number | null, public readonly details: unknown = null) { super(message); this.name = 'SamiError'; }
}
export class SamiClient {
  private readonly base: string;
  constructor(private readonly token: string, baseUrl = 'http://127.0.0.1:8080/v1', private readonly timeoutMs = 30_000) {
    const url = new URL(baseUrl);
    if (!['http:', 'https:'].includes(url.protocol) || url.username || url.password || url.search || url.hash) throw new Error('Use an HTTP(S) API URL without embedded credentials, query or fragment.');
    if (url.protocol === 'http:' && !['localhost', '127.0.0.1', '[::1]'].includes(url.hostname)) throw new Error('Use HTTPS for a remote API credential.');
    if (timeoutMs <= 0) throw new Error('Timeout must be positive.');
    this.base = baseUrl.replace(/\/$/, '');
  }
  private id(value: string) { if (!value) throw new Error('Identifier must not be empty.'); return encodeURIComponent(value); }
  async request<T = Json>(method: string, path: string, body?: Json, idempotencyKey?: string): Promise<T> {
    if (!path.startsWith('/') || path.includes('?') || path.includes('#')) throw new Error('Use a relative API path without query or fragment.');
    const controller = new AbortController(); const timeout = setTimeout(() => controller.abort(), this.timeoutMs);
    try {
      const headers: Record<string, string> = { Accept: 'application/json' };
      if (this.token) headers.Authorization = `Bearer ${this.token}`;
      if (body !== undefined) headers['Content-Type'] = 'application/json';
      if (idempotencyKey) headers['Idempotency-Key'] = idempotencyKey;
      const response = await fetch(this.base + path, { method, redirect: 'error', headers, body: body === undefined ? undefined : JSON.stringify(body), signal: controller.signal });
      const raw = await response.text(); let result: unknown;
      try { result = raw ? JSON.parse(raw) : null; } catch { result = { message: raw.slice(0, 500) }; }
      if (!response.ok) { const obj = result as { error?: { message?: string }; message?: string }; throw new SamiError(obj?.error?.message ?? obj?.message ?? `SAMI request failed (${response.status})`, response.status, result); }
      return result as T;
    } catch (error) {
      if (error instanceof SamiError) throw error;
      throw new SamiError('Transport failed. A mutation may have completed; inspect/reconcile before retrying.', null);
    } finally { clearTimeout(timeout); }
  }
  decide(taskId: string, input: JsonObject, packId = 'industrial-service') { return this.request('POST', '/decisions', { task_id: taskId, input, pack_id: packId }); }
  createSession(state: JsonObject = {}) { return this.request('POST', '/sessions', { state }); }
  respond(sessionId: string, message: string, revision?: number) { return this.request('POST', '/respond', { session_id: sessionId, message, ...(revision === undefined ? {} : { revision }) }); }
  list(kind: string) { return this.request('GET', `/admin/${this.id(kind)}`); }
  get(kind: string, id: string) { return this.request('GET', `/admin/${this.id(kind)}/${this.id(id)}`); }
  save(kind: string, record: JsonObject) { return this.request('POST', `/admin/${this.id(kind)}`, record); }
  publishSource(id: string, reason: string) { return this.request('POST', `/sources/${this.id(id)}/publish`, { reason }); }
  retractSource(id: string, reason: string) { return this.request('POST', `/sources/${this.id(id)}/retract`, { reason }); }
  deleteSource(id: string) { return this.request('DELETE', `/sources/${this.id(id)}`); }
  receipt(id: string) { return this.request('GET', `/receipts/${this.id(id)}`); }
  replay(id: string) { return this.request('POST', `/receipts/${this.id(id)}/replay`, {}); }
  proposeAction(decisionId: string, toolId: string, arguments_: JsonObject) { return this.request('POST', '/actions/propose', { decision_id: decisionId, tool_id: toolId, arguments: arguments_ }); }
  approveAction(id: string, reason: string, expiresInSeconds = 300) { return this.request('POST', `/actions/${this.id(id)}/approve`, { reason, expires_in_seconds: expiresInSeconds }); }
  executeAction(id: string, idempotencyKey: string) { if (!idempotencyKey) throw new Error('Execution requires an idempotency key.'); return this.request('POST', `/actions/${this.id(id)}/execute`, {}, idempotencyKey); }
  reconcileAction(id: string) { return this.request('POST', `/actions/${this.id(id)}/reconcile`, {}); }
  feedback(event: JsonObject) { return this.request('POST', '/feedback', event); }
  research(operation: string, input: JsonObject) { return this.request('POST', `/research/${this.id(operation)}`, input); }
  createKey(principal: string, scopes: string[], groups: string[] = []) { return this.request('POST', '/keys', { principal, scopes, groups }); }
  revokeKey(id: string) { return this.request('POST', `/keys/${this.id(id)}/revoke`, {}); }
}
