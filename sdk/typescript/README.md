# SAMI TypeScript client

Requires Node 22+ or a browser with `fetch`. Run `npm ci && npm run build` in this directory. The package's build emits `dist/index.js` and type declarations. `npm pack` can prepare a local package; publishing it is a separate maintainer action.

```typescript
import { SamiClient } from '@sami/client';

const client = new SamiClient(process.env.SAMI_API_KEY!, 'https://your-reviewed-deployment.example/v1');
const result = await client.decide('service_triage', {message: 'Your case description', asset_id: 'Your confirmed asset'});
console.log(result);
```

Supply a real trusted origin and scoped credential. The constructor takes **token first, base URL second**. Remote connections require HTTPS and redirects fail closed. Calls do not automatically retry writes. Keep action ID/idempotency key, inspect state and reconcile ambiguous effects. `SamiError` preserves status and structured details/request ID. See the parent repository's API/SDK guides and `examples/client.mjs`.

Run `npm test`. Apache-2.0; installing this SDK does not authorize sharing customer data or enabling unsupported effects.
