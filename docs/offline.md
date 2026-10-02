# Offline device tutorial

Run the native server on an edge/local host without a language API. A bundle is a signed approved snapshot; disconnected revocations propagate only on synchronization or lease expiry. This version imports a bundle into a new tenant store rather than merging an existing customer snapshot. Import has optimistic create-only checks and will fail on existing resource IDs. Use a separate empty device installation for this example.

## Provision keys

On the trusted issuing host, using its existing database-encryption secret:

```sh
sami derive-device-key --tenant default --device-id edge-device-001
```

This privileged CLI prints a **secret** per-device AES key and the issuer's public Ed25519 key. Provision the secret through an independent secure channel. Do not save it to Git or deliver it inside the encrypted bundle. Other devices/tenants derive different keys. The issuer master encryption key and private signing seed remain on the issuer. Rotating the issuer encryption seed requires securely provisioning a new device key before using new exports.

On the edge host configure:

```text
SAMI_ENV=production
SAMI_DEVICE_ID=edge-device-001
SAMI_BUNDLE_TRUST_KEYS=<pinned issuer public key>
SAMI_BUNDLE_ENCRYPTION_KEY=<this device's decryption secret>
SAMI_ENCRYPTION_KEY=<separate random edge database encryption key>
SAMI_SIGNING_KEY=<separate random edge receipt signing seed>
SAMI_BOOTSTRAP_KEY=<separate generated edge installation credential>
SAMI_DATABASE_URL=sqlite://edge.db?mode=rwc
```

Replace placeholders through secret provisioning; they are not runnable credentials. Run `sami init` without demo seeding, then serve behind the trusted local origin. The edge uses its own database/signing keys. It does not receive the issuer's private keys. `SAMI_BUNDLE_TRUST_KEYS` allows explicit issuer rotation with more than one pinned public key; remove compromised keys and expire affected devices through your incident procedure.

## Export, import and use

Call the issuing API's `POST /v1/bundles/export` with:

```json
{"device_id":"edge-device-001","pack_id":"industrial-service","lease_seconds":3600}
```

The response contains manifest, signature and encrypted payload. Only currently authorized/fresh approved evidence and the selected approved pack are exported. The manifest binds tenant, device, profile, runtime version, source epoch, digest and expiry. Export rejects a concurrent authority change. Keep the signed JSON intact; changes invalidate verification.

Submit the response object to the edge's `POST /v1/bundles/import` with a scoped `bundle.import` credential. The production edge verifies pinned issuer trust, signature, tenant/device, exact supported runtime version, expiry, authenticated encryption and digest, then atomically creates the snapshot. Existing deletion tombstones are respected. Import disables external effects and learning promotion. Execute a supported decision using `/v1/decisions` and inspect its local signed receipt. After expiry, source serving and decisions fail closed. Draft workflows do not authorize external writes.

The network-free in-process import/decision test is `bundle_import_uses_device_secret_and_public_trust_without_issuer_private_keys` in `tests/runtime.rs`. It proves two distinct installations, public-only issuer trust, wrong-issuer rejection, local decision execution and disabled effects without network calls. It does not simulate a customer's OS firewall or a complete fleet sync protocol.

## Synchronize responsibly

Do not merge offline claims by last-writer-wins. Reconnect to the authoritative installation, refresh source permission/revision, review changed evidence, upload permitted draft work as candidates, and reconcile uncertain destination effects before approving new work. Automatic bidirectional fleet conflict reconciliation and remote wiping of fully disconnected copies are not implemented. Retention of exported bundle files is an operator obligation. The exact-version check intentionally requires regenerating/reviewing a bundle after a runtime upgrade.
