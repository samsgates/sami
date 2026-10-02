import { test } from 'node:test';
import assert from 'node:assert/strict';
import { SamiClient, SamiError } from '../dist/index.js';

test('action uses encoded identifier and caller idempotency key', async () => {
  const original = globalThis.fetch; let observed;
  globalThis.fetch = async (url, options) => { observed = { url, options }; return new Response('{"id":"a"}'); };
  try {
    await new SamiClient('supplied-token').executeAction('a/b', 'stable-key');
    assert.equal(observed.url, 'http://127.0.0.1:8080/v1/actions/a%2Fb/execute');
    assert.equal(observed.options.headers.Authorization, 'Bearer supplied-token');
    assert.equal(observed.options.headers['Idempotency-Key'], 'stable-key');
  } finally { globalThis.fetch = original; }
});
test('rejects a failed write once without retries', async () => {
  const original = globalThis.fetch; let count = 0;
  globalThis.fetch = async () => { count++; return new Response('{"error":{"message":"expired"}}', { status: 409 }); };
  try { await assert.rejects(new SamiClient('').approveAction('a', 'reviewed'), SamiError); assert.equal(count, 1); }
  finally { globalThis.fetch = original; }
});

test('redirect cannot forward a credential or repeat a write', async () => {
  const { createServer } = await import('node:http');
  let requests = 0;
  const server = createServer((req, res) => { requests++; res.writeHead(302, { Location: '/sink' }); res.end(); });
  await new Promise(resolve => server.listen(0, '127.0.0.1', resolve));
  try {
    const client = new SamiClient('generated-test-only', `http://127.0.0.1:${server.address().port}/v1`);
    await assert.rejects(client.request('GET', '/redirect'), SamiError);
    assert.equal(requests, 1);
    assert.throws(() => new SamiClient('x', 'http://remote.example/v1'));
  } finally { await new Promise(resolve => server.close(resolve)); }
});
