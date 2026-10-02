use sami::{
    config::Config,
    crypto::{digest, Crypto},
    service::Service,
    storage::Credential,
};
use serde_json::json;
use std::path::PathBuf;
/// Requires an isolated empty test database. Never point this at a customer database.
#[tokio::test]
#[ignore = "requires SAMI_TEST_POSTGRES_URL pointing at an isolated empty database"]
async fn postgres_transactions_evidence_receipts_actions_and_repair() {
    let url =
        std::env::var("SAMI_TEST_POSTGRES_URL").expect("Provide isolated test PostgreSQL URL");
    assert!(url.starts_with("postgres"));
    let config = Config {
        database_url: url,
        data_dir: PathBuf::from("/tmp"),
        crypto: Crypto::new(&"11".repeat(32), &"22".repeat(32)).unwrap(),
        production: false,
        bootstrap_key: None,
        console_dir: PathBuf::from("console/dist"),
        oidc_issuer: None,
        oidc_audience: None,
        oidc_jwks: None,
        allowed_hosts: vec![],
        device_id: None,
        bundle_trust_keys: vec![],
        bundle_decryption_key: None,
    };
    let s = Service::new(config).await.unwrap();
    assert_eq!(
        s.store.count("default", "packs").await.unwrap(),
        0,
        "Database must be empty"
    );
    let key = "generated-postgres-test-only-credential-00000";
    s.store
        .add_credential(
            &digest(key.as_bytes()),
            &Credential {
                id: "pg-test".into(),
                tenant: "default".into(),
                principal: "pg-test".into(),
                scopes: vec!["*".into()],
                groups: vec![],
                expires_at: 0,
                revoked: false,
            },
        )
        .await
        .unwrap();
    s.seed_demo("default").await.unwrap();
    let p = s.auth.authenticate(key).await.unwrap();
    let d=s.decide(&p,json!({"task_id":"service_triage","input":{"asset_id":"asset:unit_84","message":"Serial 1600 is overheating","as_of":"2026-10-02","known_at":"2026-10-02T06:00:00Z"}})).await.unwrap();
    assert_eq!(d["status"], "resolved");
    let rid = d["receipt_id"].as_str().unwrap();
    assert_eq!(
        s.receipt(&p, rid, true).await.unwrap()["structured_match"],
        true
    );
    let a=s.propose_action(&p,json!({"decision_id":d["decision_id"],"tool_id":"internal.ticket","arguments":{"title":"PG validation","description":"generated evidence","queue":"thermal_service_team"}})).await.unwrap();
    let aid = a["id"].as_str().unwrap();
    s.approve_action(&p, aid, json!({"reason":"Generated test"}))
        .await
        .unwrap();
    assert_eq!(
        s.execute_action(&p, aid, "pg-test-stable-key")
            .await
            .unwrap()["state"],
        "succeeded"
    );
    assert_eq!(s.store.count("default", "tickets").await.unwrap(), 1);
    s.publish_source(
        &p,
        "bulletins",
        json!({"reason":"Generated retraction"}),
        true,
    )
    .await
    .unwrap();
    assert_eq!(
        s.store
            .get("default", "decisions", d["decision_id"].as_str().unwrap())
            .await
            .unwrap()
            .payload["eligible"],
        false
    );
    assert_eq!(
        s.receipt(&p, rid, true).await.unwrap()["structured_match"],
        true
    );
    assert!(s.store.backup().await.unwrap()["payload"].is_string());
    s.store.revoke("default", "pg-test").await.unwrap();
    assert!(s.auth.still_authorized(&p).await.is_err());
}
