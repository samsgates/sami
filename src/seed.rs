//! Generated examples are opt-in and do not certify customer policies.
use crate::{
    auth::Principal,
    error::{Error, Result},
    service::Service,
    storage::Write,
};
use serde_json::{json, Value};
impl Service {
    pub async fn seed_demo(&self, tenant: &str) -> Result<Value> {
        if self.config.production {
            return Err(Error::bad(
                "Demo seeding is disabled in production. Use a separate development database.",
            ));
        }
        if self.store.count(tenant, "packs").await? > 0 {
            if self
                .store
                .get(tenant, "packs", "industrial-service")
                .await
                .is_ok_and(|p| p.payload["demo_only"] == true)
            {
                return Ok(
                    json!({"status":"already_seeded","tenant":tenant,"existing_data_preserved":true}),
                );
            }
            return Err(Error::conflict(
                "Seed only an empty tenant to avoid changing existing policy.",
            ));
        }
        let fixture: Value =
            serde_json::from_str(include_str!("../packs/industrial-service/fixtures.json"))?;
        let mut writes = Vec::new();
        for mut s in fixture["sources"].as_array().cloned().unwrap_or_default() {
            let id = s["id"]
                .as_str()
                .ok_or_else(|| Error::bad("Invalid demo source"))?
                .to_string();
            s["authority_mode"] = json!("local");
            s["local_authority"] = json!(true);
            s["permission_checked_at"] = json!(chrono::Utc::now().to_rfc3339());
            s["license"] = json!("Apache-2.0 generated demonstration content");
            s["status"] = json!("approved");
            writes.push(Write {
                kind: "sources".into(),
                id,
                payload: s,
                expected: Some(0),
                immutable: false,
            });
        }
        for mut c in fixture["claims"].as_array().cloned().unwrap_or_default() {
            let id = c["id"]
                .as_str()
                .ok_or_else(|| Error::bad("Invalid demo claim"))?
                .to_string();
            c["source_revision"] = json!(1);
            writes.push(Write {
                kind: "claims".into(),
                id,
                payload: c,
                expected: Some(0),
                immutable: false,
            });
        }
        let mut pack = sami_intelligence::default_pack();
        pack["status"] = json!("approved");
        pack["demo_only"] = json!(true);
        let report = sami_research::dispatch("pack.validate", &pack)
            .map_err(|e| Error::bad(e.to_string()))?;
        if report["valid"] == false {
            return Err(Error::bad(format!("Demo pack validation failed: {report}")));
        }
        pack["validation"] = report;
        for (id, wf) in pack["workflows"].as_object().into_iter().flatten() {
            let mut v = wf.clone();
            v["status"] = json!("approved");
            writes.push(Write {
                kind: "workflows".into(),
                id: id.clone(),
                payload: v,
                expected: Some(0),
                immutable: false,
            });
        }
        writes.push(Write {
            kind: "packs".into(),
            id: "industrial-service".into(),
            payload: pack,
            expected: Some(0),
            immutable: false,
        });
        writes.push(Write{kind:"tools".into(),id:"internal.ticket".into(),payload:json!({"adapter":"internal_ticket","effect_class":"administrative_write","enabled":true,"required_scopes":["ticket.create"],"input_schema":{"type":"object","required":["title","description","queue"],"properties":{"title":{"type":"string","maxLength":200},"description":{"type":"string","maxLength":4000},"queue":{"type":"string","maxLength":100}},"additionalProperties":false}}),expected:Some(0),immutable:false});
        writes.push(Write{kind:"controls".into(),id:"runtime".into(),payload:json!({"effects_enabled":true,"learning_enabled":false,"sources_enabled":true}),expected:Some(0),immutable:false});
        let epoch = self
            .store
            .commit(tenant, "demo-seeder", "demo.seed", writes, None)
            .await?;
        Ok(
            json!({"status":"seeded","tenant":tenant,"epoch":epoch,"classification":"generated_demo_not_customer_validation"}),
        )
    }
    pub async fn worker_once(&self, p: &Principal) -> Result<Value> {
        p.require("action.reconcile")?;
        self.auth.still_authorized(p).await?;
        let mut results = Vec::new();
        for r in self.store.list(&p.tenant, "actions", 100, 0).await? {
            if ["executing", "unknown_completion"]
                .contains(&r.payload["state"].as_str().unwrap_or(""))
                && crate::auth::timestamp(r.payload.get("dispatch_committed_at")).unwrap_or(0) + 30
                    < chrono::Utc::now().timestamp()
            {
                let result = self.reconcile_action(p, &r.id).await;
                results.push(match result {
                    Ok(v) => json!({"action_id":r.id,"state":v["state"]}),
                    Err(e) => json!({"action_id":r.id,"error":e.code}),
                });
            }
        }
        let mut timers = 0;
        if p.allows("workflow.execute") {
            for run in self.store.list(&p.tenant, "workflow_runs", 1000, 0).await? {
                if run.payload["status"] == "waiting_timer"
                    && run.payload["wake_at"]
                        .as_i64()
                        .is_some_and(|n| n <= chrono::Utc::now().timestamp())
                {
                    let mut v = run.payload;
                    v["status"] = json!("awaiting_confirmation");
                    v["timer_fired_at"] = json!(chrono::Utc::now().to_rfc3339());
                    self.store
                        .commit(
                            &p.tenant,
                            &p.subject,
                            "workflow.timer_fired",
                            vec![Write {
                                kind: "workflow_runs".into(),
                                id: run.id,
                                payload: v,
                                expected: Some(run.revision),
                                immutable: false,
                            }],
                            None,
                        )
                        .await?;
                    timers += 1;
                }
            }
        }
        Ok(json!({"reconciliations":results,"retry_effects":false,"timers_fired":timers}))
    }
}
