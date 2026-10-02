import { SamiClient } from '../sdk/typescript/dist/index.js';
const client = new SamiClient(process.env.SAMI_API_KEY ?? '', process.env.SAMI_API_URL ?? 'http://127.0.0.1:8080/v1');
const decision = await client.decide('warranty_prequalification', { message: 'Review warranty for serial 1600', asset_id: 'asset:unit_84', as_of: '2026-10-02' });
console.log(JSON.stringify({ status: decision.status, value: decision.value, response: decision.response }, null, 2));
