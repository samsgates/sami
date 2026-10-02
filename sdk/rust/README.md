# SAMI Rust client

Requires Rust 1.86+. This independent Cargo package exposes `sami_client::SamiClient`. Use a local path dependency on `sdk/rust` from your application; review package metadata before a registry release.

```rust
use sami_client::SamiClient;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
let client = SamiClient::new("https://your-reviewed-deployment.example/v1", std::env::var("SAMI_API_KEY")?)?;
let result = client.decide("service_triage", json!({"message":"Your case description","asset_id":"Your confirmed asset"}), "industrial-service").await?;
println!("{}", result["status"]);
Ok(())
}
```

Configure a real trusted deployment origin and scoped secret. HTTPS is required for remote credentials; redirects and automatic mutation retries are disabled. Preserve stable action IDs/keys and reconcile uncertain effects. Typed API errors retain status/details, including request ID. See the parent API/SDK guides. Run `cargo test --locked`. Apache-2.0.
