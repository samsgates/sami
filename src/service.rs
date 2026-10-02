use crate::{
    auth::{timestamp, Auth, Principal},
    config::Config,
    crypto::{canonical, digest, token},
    error::{Error, Result},
    storage::{Credential, Record, Store, Write},
};
use chrono::Utc;
use serde_json::{json, Value};
use std::{
    collections::{BTreeMap, HashSet},
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    time::Duration,
};

#[derive(Clone)]
pub struct Service {
    pub store: Store,
    pub auth: Auth,
    pub config: Config,
    pub client: reqwest::Client,
    pub requests: Arc<AtomicU64>,
    pub decisions: Arc<AtomicU64>,
    pub compute: Arc<tokio::sync::Semaphore>,
}
fn string(v: &Value, k: &str) -> Result<String> {
    v[k].as_str()
        .filter(|s| !s.is_empty() && s.len() <= 200)
        .map(str::to_string)
        .ok_or_else(|| {
            Error::bad(format!(
                "{k} must be a nonempty string (maximum 200 characters)."
            ))
        })
}
fn write(kind: &str, id: &str, v: Value, expected: Option<i64>) -> Write {
    Write {
        kind: kind.into(),
        id: id.into(),
        payload: v,
        expected,
        immutable: false,
    }
}
fn values(records: Vec<Record>) -> Vec<Value> {
    records.into_iter().map(|r| r.payload).collect()
}
pub const KINDS: &[&str] = &[
    "sources",
    "claims",
    "packs",
    "rules",
    "workflows",
    "tools",
    "sessions",
    "feedback",
    "episodes",
    "calibrators",
    "procedures",
    "evaluations",
    "deployments",
    "controls",
    "identities",
    "tickets",
    "decisions",
    "receipts",
    "actions",
    "promotions",
    "workflow_runs",
    "schemas",
    "personas",
    "entities",
    "relations",
    "conflicts",
    "decision_schemas",
];
impl Service {
    pub async fn new(config: Config) -> Result<Self> {
        let store = Store::connect(&config.database_url, config.crypto.clone()).await?;
        crate::auth::bootstrap(&store, &config).await?;
        let auth = Auth::new(store.clone(), config.clone())?;
        Ok(Self {
            store,
            auth,
            config,
            client: reqwest::Client::builder()
                .timeout(Duration::from_secs(10))
                .redirect(reqwest::redirect::Policy::none())
                .build()
                .map_err(|_| Error::internal())?,
            requests: Arc::new(AtomicU64::new(0)),
            decisions: Arc::new(AtomicU64::new(0)),
            compute: Arc::new(tokio::sync::Semaphore::new(4)),
        })
    }
    pub async fn controls(&self, p: &Principal) -> Result<Value> {
        match self.store.get(&p.tenant, "controls", "runtime").await {
            Ok(r) => Ok(r.payload),
            Err(e) if e.status == axum::http::StatusCode::NOT_FOUND => {
                Ok(json!({"effects_enabled":true,"learning_enabled":false,"sources_enabled":true}))
            }
            Err(e) => Err(e),
        }
    }
    pub async fn list(&self, p: &Principal, kind: &str, limit: i64, offset: i64) -> Result<Value> {
        if !KINDS.contains(&kind) {
            return Err(Error::missing());
        }
        p.require(&format!("admin.{kind}.read"))?;
        let list = self.store.list(&p.tenant, kind, limit, offset).await?;
        let mut items = Vec::new();
        for r in list {
            if self
                .authorize_record(p, &r.payload, "administration")
                .await
                .is_ok()
            {
                items.push(r.payload)
            }
        }
        Ok(json!({"items":items,"limit":limit,"offset":offset,"next_offset":offset+limit}))
    }
    pub async fn get(&self, p: &Principal, kind: &str, id: &str) -> Result<Value> {
        if !KINDS.contains(&kind) {
            return Err(Error::missing());
        }
        p.require(&format!("admin.{kind}.read"))?;
        let r = self.store.get(&p.tenant, kind, id).await?;
        self.authorize_record(p, &r.payload, "administration")
            .await?;
        Ok(r.payload)
    }
    pub async fn save(&self, p: &Principal, kind: &str, mut body: Value) -> Result<Value> {
        if !KINDS.contains(&kind)
            || [
                "decisions",
                "receipts",
                "actions",
                "episodes",
                "tickets",
                "promotions",
                "workflow_runs",
            ]
            .contains(&kind)
        {
            return Err(Error::bad(
                "This resource requires its dedicated lifecycle endpoint.",
            ));
        }
        p.require(&format!("admin.{kind}.write"))?;
        if !body.is_object() {
            return Err(Error::bad("An object is required."));
        }
        if body
            .get("tenant_id")
            .is_some_and(|v| v.as_str() != Some(&p.tenant))
        {
            return Err(Error::forbidden("Cross-tenant mutation is prohibited."));
        }
        if kind == "identities" {
            let issuer = self
                .config
                .oidc_issuer
                .as_deref()
                .ok_or_else(|| Error::bad("Configure OIDC issuer."))?;
            let sub = string(&body, "subject")?;
            body["id"] = json!(digest(format!("{issuer}|{sub}").as_bytes()));
        }
        let id = body
            .get("id")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        let id = if id.is_empty() {
            uuid::Uuid::new_v4().to_string()
        } else {
            id
        };
        if id.len() > 200 || id.contains('/') || id.contains('\\') {
            return Err(Error::bad("Invalid resource ID."));
        }
        if kind == "sources" && self.store.get(&p.tenant, "tombstones", &id).await.is_ok() {
            return Err(Error::forbidden(
                "Deleted sources cannot be resurrected by ingestion. Use a new reviewed source ID.",
            ));
        }
        let old = match self.store.get(&p.tenant, kind, &id).await {
            Ok(r) => Some(r),
            Err(e) if e.status == axum::http::StatusCode::NOT_FOUND => None,
            Err(e) => return Err(e),
        };
        let expected = body.get("revision").and_then(Value::as_i64).unwrap_or(0);
        if old
            .as_ref()
            .is_some_and(|r| !p.can_read(&r.payload, "administration"))
        {
            return Err(Error::missing());
        }
        if old.is_some() && expected == 0 {
            return Err(Error::conflict(
                "An existing resource requires its current revision.",
            ));
        }
        body["created_by"] = old
            .as_ref()
            .and_then(|r| r.payload.get("created_by"))
            .cloned()
            .unwrap_or_else(|| json!(digest(p.subject.as_bytes())));
        body["updated_at"] = json!(Utc::now().to_rfc3339());
        body["tenant_id"] = json!(p.tenant);
        if kind == "sources" {
            if body["content"].as_str().is_none() {
                return Err(Error::bad("Source content must be text."));
            }
            if body["license"].as_str().is_none_or(|l| l.is_empty()) {
                return Err(Error::bad(
                    "A source license or usage-rights declaration is required.",
                ));
            }
            if body["status"].as_str() == Some("approved") {
                return Err(Error::bad(
                    "Publish the candidate through /sources/{id}/publish.",
                ));
            }
            body["status"] = json!("candidate");
            body["epoch"] = json!(
                old.as_ref()
                    .and_then(|r| r.payload["epoch"].as_i64())
                    .unwrap_or(0)
                    + 1
            );
            body["permission_checked_at"] = json!(Utc::now().to_rfc3339());
            body["authority_mode"] = json!("local");
            body["local_authority"] = json!(true);
        }
        if kind == "claims" {
            self.validate_claim(p, &mut body).await?;
        }
        if kind == "packs" {
            let validation = sami_research::dispatch("pack.validate", &body)
                .map_err(|e| Error::bad(e.to_string()))?;
            body["validation"] = validation;
            body["status"] = json!("candidate");
        }
        if kind == "tools" {
            if body["effect_class"].as_str() != Some("administrative_write") {
                return Err(Error::bad("Only administrative writes are supported; financial and physical actuation are excluded."));
            }
            let adapter = body["adapter"].as_str().unwrap_or("");
            if !["internal_ticket", "http_json"].contains(&adapter) {
                return Err(Error::bad("Unsupported tool adapter."));
            }
            if adapter == "http_json" {
                self.validate_destination(&string(&body, "url")?).await?;
            }
            if body["input_schema"].as_object().is_none() {
                return Err(Error::bad("Tool input_schema is mandatory."));
            }
        }
        if kind == "identities" {
            p.require("identity.provision")?;
            body["tenant"] = json!(p.tenant);
            let scopes: Vec<String> = serde_json::from_value(body["scopes"].clone())?;
            let groups: Vec<String> =
                serde_json::from_value(body.get("groups").cloned().unwrap_or_else(|| json!([])))?;
            if scopes.is_empty()
                || scopes.iter().any(|s| !p.allows(s))
                || (!p.allows("*") && groups.iter().any(|g| !p.groups.contains(g)))
            {
                return Err(Error::forbidden(
                    "Identity cannot exceed provisioner authority.",
                ));
            }
            body["groups"] = json!(groups);
            let issuer = self.config.oidc_issuer.as_deref().ok_or_else(|| {
                Error::bad("Configure an OIDC issuer before provisioning identities.")
            })?;
            let sub = string(&body, "subject")?;
            let oidc_id = digest(format!("{issuer}|{sub}").as_bytes());
            if let Ok(existing) = self
                .store
                .get("_identity_registry", "identities", &oidc_id)
                .await
            {
                if existing.payload["tenant"] != p.tenant {
                    return Err(Error::forbidden("Identity belongs to another tenant."));
                }
            }
            self.store
                .commit(
                    "_identity_registry",
                    &p.subject,
                    "identity.route",
                    vec![write(
                        "identities",
                        &oidc_id,
                        json!({"tenant":p.tenant}),
                        None,
                    )],
                    None,
                )
                .await?;
        }
        let mut writes = vec![write(kind, &id, body.clone(), Some(expected))];
        if [
            "claims",
            "sources",
            "packs",
            "rules",
            "controls",
            "tools",
            "identities",
        ]
        .contains(&kind)
        {
            writes.extend(
                self.invalidation(
                    p,
                    if kind == "sources" { Some(&id) } else { None },
                    if kind == "claims" { Some(&id) } else { None },
                    kind,
                )
                .await?,
            );
        }
        self.store
            .commit(&p.tenant, &p.subject, &format!("{kind}.save"), writes, None)
            .await?;
        Ok(self.store.get(&p.tenant, kind, &id).await?.payload)
    }
    async fn validate_claim(&self, p: &Principal, v: &mut Value) -> Result<()> {
        for k in ["subject", "predicate", "source_id", "locator"] {
            string(v, k)?;
        }
        if v.get("value").is_none() {
            return Err(Error::bad("A typed claim value is required."));
        }
        let source = self
            .store
            .get(
                &p.tenant,
                "sources",
                v["source_id"].as_str().unwrap_or_default(),
            )
            .await?;
        if !p.can_read(&source.payload, "administration") {
            return Err(Error::forbidden("Source access denied."));
        }
        let status = v["status"].as_str().unwrap_or("candidate");
        if status == "approved" {
            p.require("claims.approve")?;
            if source.payload["status"] != "approved" {
                return Err(Error::bad("Approved claims require an approved source."));
            }
        }
        if ![
            "candidate",
            "approved",
            "observed",
            "disputed",
            "superseded",
            "retracted",
        ]
        .contains(&status)
        {
            return Err(Error::bad("Invalid claim lifecycle state."));
        }
        v["status"] = json!(status);
        v["source_revision"] = json!(source.revision);
        v["source_family"] = source
            .payload
            .get("source_family")
            .cloned()
            .unwrap_or_else(|| json!(source.id));
        v["acl"] = source
            .payload
            .get("acl")
            .cloned()
            .unwrap_or_else(|| json!([]));
        v["purpose"] = source
            .payload
            .get("purpose")
            .cloned()
            .unwrap_or_else(|| json!([]));
        v["system_from"] = json!(Utc::now().to_rfc3339());
        if v["depends_on"].as_array().is_some_and(|a| a.len() > 200) {
            return Err(Error::bad("Claim dependency budget exceeded."));
        }
        Ok(())
    }
    pub async fn publish_source(
        &self,
        p: &Principal,
        id: &str,
        body: Value,
        retract: bool,
    ) -> Result<Value> {
        p.require(if retract {
            "sources.retract"
        } else {
            "sources.publish"
        })?;
        string(&body, "reason")?;
        let source = self.store.get(&p.tenant, "sources", id).await?;
        if !p.can_read(&source.payload, "administration") {
            return Err(Error::missing());
        }
        let mut v = source.payload.clone();
        v["status"] = json!(if retract { "retracted" } else { "approved" });
        v["epoch"] = json!(v["epoch"].as_i64().unwrap_or(0) + 1);
        v["permission_checked_at"] = json!(Utc::now().to_rfc3339());
        let mut writes = vec![write("sources", id, v, Some(source.revision))];
        writes.extend(
            self.invalidation(p, Some(id), None, "source_revision")
                .await?,
        );
        if let Some(claims) = body["claims"].as_array() {
            if retract {
                return Err(Error::bad(
                    "Cannot approve claims while retracting a source.",
                ));
            }
            p.require("claims.approve")?;
            for claim in claims {
                let mut c = claim.clone();
                let cid = string(&c, "id")?;
                let existing = self.store.get(&p.tenant, "claims", &cid).await.ok();
                if let Some(old) = &existing {
                    self.authorize_record(p, &old.payload, "administration")
                        .await?;
                    if claim["revision"].as_i64() != Some(old.revision) {
                        return Err(Error::conflict(
                            "Supply the current claim revision for publication.",
                        ));
                    }
                }
                for key in ["subject", "predicate", "locator"] {
                    string(&c, key)?;
                }
                if c.get("value").is_none() {
                    return Err(Error::bad("Claim value required."));
                }
                c["source_id"] = json!(id);
                c["source_revision"] = json!(source.revision + 1);
                c["status"] = json!("approved");
                c["source_family"] = source
                    .payload
                    .get("source_family")
                    .cloned()
                    .unwrap_or_else(|| json!(id));
                c["acl"] = source
                    .payload
                    .get("acl")
                    .cloned()
                    .unwrap_or_else(|| json!([]));
                c["purpose"] = source
                    .payload
                    .get("purpose")
                    .cloned()
                    .unwrap_or_else(|| json!([]));
                c["system_from"] = json!(Utc::now().to_rfc3339());
                writes.retain(|w| w.kind != "claims" || w.id != cid);
                writes.push(write(
                    "claims",
                    &cid,
                    c,
                    Some(existing.map_or(0, |r| r.revision)),
                ));
            }
        }
        let epoch = self
            .store
            .commit(
                &p.tenant,
                &p.subject,
                if retract {
                    "source.retract"
                } else {
                    "source.publish"
                },
                writes,
                None,
            )
            .await?;
        Ok(
            json!({"source":self.store.get(&p.tenant,"sources",id).await?.payload,"epoch":epoch,"repair":"dependent authority invalidated"}),
        )
    }
    async fn invalidation(
        &self,
        p: &Principal,
        source: Option<&str>,
        claim: Option<&str>,
        reason: &str,
    ) -> Result<Vec<Write>> {
        let all = self.store.list(&p.tenant, "claims", 10000, 0).await?;
        if self.store.count(&p.tenant, "claims").await? > 10000 {
            return Err(Error::unavailable("Repair exceeds the native 10000-claim budget; archive or extend the repair service before publication."));
        }
        let mut invalid: HashSet<String> = claim.into_iter().map(str::to_string).collect();
        for r in &all {
            if source.is_some_and(|id| r.payload["source_id"].as_str() == Some(id)) {
                invalid.insert(r.id.clone());
            }
        }
        loop {
            let before = invalid.len();
            for r in &all {
                let deps = r.payload["depends_on"].as_array();
                let intersects = deps.is_some_and(|a| {
                    a.iter()
                        .filter_map(Value::as_str)
                        .any(|d| invalid.contains(d))
                });
                if intersects {
                    let alternative = r.payload["support_sets"].as_array().is_some_and(|sets| {
                        sets.iter().any(|s| {
                            s.as_array().is_some_and(|a| {
                                !a.is_empty()
                                    && a.iter().all(|v| {
                                        v.as_str().is_some_and(|id| {
                                            !invalid.contains(id)
                                                && all.iter().any(|r| {
                                                    r.id == id && r.payload["status"] == "approved"
                                                })
                                        })
                                    })
                            })
                        })
                    });
                    if !alternative {
                        invalid.insert(r.id.clone());
                    }
                }
            }
            if invalid.len() == before {
                break;
            }
        }
        for kind in ["decisions", "actions"] {
            if self.store.count(&p.tenant, kind).await? > 10000 {
                return Err(Error::unavailable(
                    "Repair capacity exceeded; archival required before publication.",
                ));
            }
        }
        let mut writes = Vec::new();
        for r in all {
            if invalid.contains(&r.id) && claim != Some(r.id.as_str()) {
                let mut v = r.payload;
                v["status"] = json!("disputed");
                v["invalidation_reason"] = json!(reason);
                writes.push(write("claims", &r.id, v, Some(r.revision)));
            }
        }
        // Negative conclusions and policy changes depend on the query watermark: conservative tenant-wide decision invalidation.
        for r in self.store.list(&p.tenant, "decisions", 10000, 0).await? {
            if r.payload["eligible"] != false {
                let mut v = r.payload;
                v["eligible"] = json!(false);
                v["invalidation_reason"] = json!(reason);
                writes.push(write("decisions", &r.id, v, Some(r.revision)));
            }
        }
        for r in self.store.list(&p.tenant, "actions", 10000, 0).await? {
            let mut v = r.payload;
            if ["proposed", "awaiting_approval", "authorized"]
                .contains(&v["state"].as_str().unwrap_or(""))
            {
                v["state"] = json!("invalidated");
                v["reason"] = json!(reason);
                writes.push(write("actions", &r.id, v, Some(r.revision)));
            }
        }
        Ok(writes)
    }
    pub async fn delete_source(&self, p: &Principal, id: &str) -> Result<Value> {
        p.require("sources.delete")?;
        let source = self.store.get(&p.tenant, "sources", id).await?;
        if !p.can_read(&source.payload, "administration") {
            return Err(Error::missing());
        }
        let mut writes = self
            .invalidation(p, Some(id), None, "source_deleted")
            .await?;
        writes.push(write(
            "tombstones",
            id,
            json!({"source_id":id,"at":Utc::now().to_rfc3339()}),
            Some(0),
        ));
        let mut s = source.payload;
        s["status"] = json!("deleted");
        s["content"] = json!("");
        writes.push(write("sources", id, s, Some(source.revision)));
        self.store
            .commit(&p.tenant, &p.subject, "source.delete", writes, None)
            .await?;
        let claims = self.store.list(&p.tenant, "claims", 10000, 0).await?;
        let mut affected: HashSet<String> = claims
            .iter()
            .filter(|c| c.payload["source_id"].as_str() == Some(id))
            .map(|c| c.id.clone())
            .collect();
        loop {
            let before = affected.len();
            for c in &claims {
                if c.payload["depends_on"].as_array().is_some_and(|a| {
                    a.iter()
                        .filter_map(Value::as_str)
                        .any(|x| affected.contains(x))
                }) {
                    affected.insert(c.id.clone());
                }
            }
            if before == affected.len() {
                break;
            }
        }
        for cid in affected {
            self.store.purge(&p.tenant, "claims", &cid).await?;
        }
        for r in self
            .store
            .list(&p.tenant, "receipt_payloads", 10000, 0)
            .await?
        {
            if r.payload["source_ids"]
                .as_array()
                .is_some_and(|a| a.iter().any(|x| x.as_str() == Some(id)))
            {
                self.store
                    .purge(&p.tenant, "receipt_payloads", &r.id)
                    .await?;
            }
        }
        for kind in [
            "decisions",
            "actions",
            "tickets",
            "approvals",
            "workflow_runs",
            "sessions",
            "feedback",
            "episodes",
            "procedures",
            "promotions",
            "calibrators",
            "evaluations",
        ] {
            // Free-text derivatives can contain source facts without complete structured lineage.
            // Purge the tenant's derivative payloads conservatively; audit hashes remain minimal.
            if self.store.count(&p.tenant, kind).await? > 10000 {
                return Err(Error::unavailable("Derivative deletion requires batched operator purge; serving remains excluded."));
            }
            for r in self.store.list(&p.tenant, kind, 10000, 0).await? {
                self.store.purge(&p.tenant, kind, &r.id).await?;
            }
        }
        self.store.purge(&p.tenant, "sources", id).await?;
        Ok(
            json!({"status":"deleted","serving_excluded":true,"active_payloads_purged":true,"backup_retention":"Operator-configured; restore must apply tombstones."}),
        )
    }
    pub fn source_current(&self, s: &Value) -> bool {
        if s["status"] != "approved" {
            return false;
        }
        if s["authority_mode"].as_str() == Some("local") {
            return true;
        }
        let checked = timestamp(s.get("permission_checked_at")).unwrap_or(0);
        let max = s["max_age_seconds"].as_i64().unwrap_or(300).clamp(1, 300);
        checked + max > Utc::now().timestamp()
    }
    pub async fn decide(&self, p: &Principal, body: Value) -> Result<Value> {
        p.require("decision.invoke")?;
        let controls = self.controls(p).await?;
        if controls["bundle_expires_at"]
            .as_i64()
            .is_some_and(|n| n <= Utc::now().timestamp())
        {
            return Err(Error::forbidden(
                "Offline authority lease expired. Synchronize before serving.",
            ));
        }
        if controls["sources_enabled"] == false {
            return Err(Error::unavailable("Source authority is suspended."));
        }
        let task = body
            .get("task_id")
            .or_else(|| body.get("task"))
            .and_then(Value::as_str)
            .unwrap_or("service_triage");
        let pack_id = body["pack_id"].as_str().unwrap_or("industrial-service");
        let epoch = self.store.epoch(&p.tenant).await?;
        let pack = self.store.get(&p.tenant, "packs", pack_id).await?;
        if pack.payload["status"] != "approved" {
            return Err(Error::unavailable("The domain pack has not been approved."));
        }
        self.authorize_record(
            p,
            &pack.payload,
            body["purpose"]
                .as_str()
                .unwrap_or("service_prequalification"),
        )
        .await?;
        if self.store.count(&p.tenant, "claims").await? > 10000
            || self.store.count(&p.tenant, "sources").await? > 5000
        {
            return Err(Error::unavailable(
                "Evidence search exceeds the configured serving budget.",
            ));
        }
        let purpose = body["purpose"]
            .as_str()
            .unwrap_or("service_prequalification");
        let sources: Vec<_> = values(self.store.list(&p.tenant, "sources", 5000, 0).await?)
            .into_iter()
            .filter(|v| p.can_read(v, purpose) && self.source_current(v))
            .collect();
        let mut input = body.get("input").cloned().unwrap_or_else(|| body.clone());
        if !input.is_object() {
            return Err(Error::bad("Decision input must be an object."));
        }
        input["sources"] = json!(sources);
        input["known_at"] = input
            .get("known_at")
            .cloned()
            .unwrap_or_else(|| json!(Utc::now().to_rfc3339()));
        input["as_of"] = input
            .get("as_of")
            .cloned()
            .unwrap_or_else(|| json!(Utc::now().date_naive().to_string()));
        let claims: Vec<_> = values(self.store.list(&p.tenant, "claims", 10000, 0).await?)
            .into_iter()
            .filter(|v| {
                p.can_read(v, purpose)
                    && v["status"] == "approved"
                    && sources
                        .iter()
                        .any(|s| s["id"] == v["source_id"] && s["revision"] == v["source_revision"])
            })
            .collect();
        let task_owned = task.to_string();
        let input_clone = input.clone();
        let claims_clone = claims.clone();
        let pack_clone = pack.payload.clone();
        let permit =
            self.compute.clone().try_acquire_owned().map_err(|_| {
                Error::unavailable("Compute admission budget exhausted; retry later.")
            })?;
        let result = tokio::time::timeout(
            Duration::from_millis(500),
            tokio::task::spawn_blocking(move || {
                let _permit = permit;
                sami_intelligence::decide(&task_owned, &input_clone, &claims_clone, &pack_clone)
            }),
        )
        .await
        .map_err(|_| Error::unavailable("Decision execution budget exhausted."))?
        .map_err(|_| Error::internal())?
        .map_err(|e| Error::bad(e.to_string()))?;
        let id = uuid::Uuid::new_v4().to_string();
        let rid = format!("receipt_{id}");
        let epochs: BTreeMap<String, Value> = sources
            .iter()
            .filter_map(|s| {
                s["id"].as_str().map(|id| {
                    (
                        id.to_string(),
                        json!({"revision":s["revision"],"epoch":s["epoch"]}),
                    )
                })
            })
            .collect();
        let envelope = json!({"version":"1.0","id":rid,"tenant_ref":digest(p.tenant.as_bytes()),"principal_ref":digest(p.subject.as_bytes()),"task":task,"input_digest":digest(&canonical(&input)),"output_digest":digest(&canonical(&result)),"snapshot_epoch":epoch,"pack_ref":digest(&canonical(&pack.payload)),"created_at":Utc::now().to_rfc3339(),"scope":"typed checks relative to declared premises; not proof of source truth"});
        let receipt = json!({"envelope":envelope,"signature":self.config.crypto.sign(&envelope),"signing":"Ed25519","public_key":self.config.crypto.public_key(),"payload_id":rid});
        let mut output = result.clone();
        output["decision_id"] = json!(id);
        output["receipt_id"] = json!(rid);
        output["task_id"] = json!(task);
        output["eligible"] = json!(true);
        output["source_epochs"] = json!(epochs);
        output["pack_id"] = json!(pack_id);
        output["pack_revision"] = json!(pack.revision);
        output["profile"] = json!("native_strict");
        output["owner_ref"] = json!(digest(p.subject.as_bytes()));
        output["source_ids"] = json!(sources
            .iter()
            .filter_map(|s| s["id"].as_str())
            .collect::<Vec<_>>());
        let payload = json!({"owner_ref":digest(p.subject.as_bytes()),"input":input,"claims":claims,"pack":pack.payload,"result":result,"source_ids":sources.iter().filter_map(|s|s["id"].as_str()).collect::<Vec<_>>()});
        let mut writes = vec![
            write("decisions", &id, output.clone(), Some(0)),
            write("receipt_payloads", &rid, payload, Some(0)),
        ];
        let mut rw = write("receipts", &rid, receipt, Some(0));
        rw.immutable = true;
        writes.push(rw);
        self.store
            .commit(
                &p.tenant,
                &p.subject,
                "decision.record",
                writes,
                Some(epoch),
            )
            .await?;
        self.decisions.fetch_add(1, Ordering::Relaxed);
        Ok(output)
    }
    pub async fn receipt(&self, p: &Principal, id: &str, replay: bool) -> Result<Value> {
        p.require("receipt.read")?;
        let r = self.store.get(&p.tenant, "receipts", id).await?;
        let envelope = r.payload["envelope"].clone();
        let verified = self
            .config
            .crypto
            .verify(&envelope, r.payload["signature"].as_str().unwrap_or(""));
        let payload = match self.store.get(&p.tenant, "receipt_payloads", id).await {
            Ok(p) => p,
            Err(e) if e.status == axum::http::StatusCode::NOT_FOUND => {
                return Ok(
                    json!({"receipt":r.payload,"signature_valid":verified,"replay_status":"not_replayable_due_to_deletion"}),
                )
            }
            Err(e) => return Err(e),
        };
        self.authorize_record(p, &payload.payload, "service_prequalification")
            .await?;
        if !replay {
            return Ok(
                json!({"receipt":r.payload,"signature_valid":verified,"evidence":payload.payload,"current_epoch":self.store.epoch(&p.tenant).await?}),
            );
        }
        p.require("receipt.replay")?;
        let q = payload.payload;
        let task = envelope["task"].as_str().unwrap_or("");
        let claims: Vec<Value> = serde_json::from_value(q["claims"].clone())?;
        let result = sami_intelligence::decide(task, &q["input"], &claims, &q["pack"])
            .map_err(|e| Error::bad(e.to_string()))?;
        Ok(
            json!({"status":"replayed","structured_match":canonical(&result)==canonical(&q["result"]),"signature_valid":verified,"result":result,"external_effects":false,"snapshot_epoch":envelope["snapshot_epoch"],"current_epoch":self.store.epoch(&p.tenant).await?}),
        )
    }
    pub async fn publish_pack(&self, p: &Principal, id: &str, body: Value) -> Result<Value> {
        p.require("packs.publish")?;
        string(&body, "reason")?;
        let r = self.store.get(&p.tenant, "packs", id).await?;
        self.authorize_record(p, &r.payload, "administration")
            .await?;
        if !p.allows("*") && r.payload["created_by"].as_str() == Some(&digest(p.subject.as_bytes()))
        {
            return Err(Error::forbidden(
                "A different authorized reviewer must publish this pack.",
            ));
        }
        let validation = sami_research::dispatch("pack.validate", &r.payload)
            .map_err(|e| Error::bad(e.to_string()))?;
        if validation
            .get("publication_eligible")
            .and_then(Value::as_bool)
            != Some(true)
        {
            return Err(Error::bad(format!(
                "Pack validation failed: {}",
                validation.get("errors").unwrap_or(&validation)
            )));
        }
        let mut v = r.payload;
        v["status"] = json!("approved");
        v["publication_reason"] = body["reason"].clone();
        v["validation"] = validation;
        let mut writes = self.invalidation(p, None, None, "pack_publication").await?;
        writes.push(write("packs", id, v, Some(r.revision)));
        self.store
            .commit(&p.tenant, &p.subject, "pack.publish", writes, None)
            .await?;
        Ok(self.store.get(&p.tenant, "packs", id).await?.payload)
    }
    pub async fn feedback(&self, p: &Principal, mut body: Value) -> Result<Value> {
        p.require("feedback.write")?;
        if !body.is_object() {
            return Err(Error::bad("Feedback must be an object."));
        }
        let id = uuid::Uuid::new_v4().to_string();
        body["status"] = json!("quarantined");
        body["principal_ref"] = json!(digest(p.subject.as_bytes()));
        body["created_at"] = json!(Utc::now().to_rfc3339());
        self.store
            .commit(
                &p.tenant,
                &p.subject,
                "feedback.quarantine",
                vec![write("feedback", &id, body.clone(), Some(0))],
                None,
            )
            .await?;
        body["id"] = json!(id);
        Ok(body)
    }
    pub async fn promote(&self, p: &Principal, id: &str, body: Value) -> Result<Value> {
        p.require("learning.promote")?;
        if self.controls(p).await?["learning_enabled"] != true {
            return Err(Error::forbidden("Learning promotion is disabled."));
        }
        string(&body, "reason")?;
        let f = self.store.get(&p.tenant, "feedback", id).await?;
        let report=sami_research::dispatch("feedback.validate",&json!({"feedback":f.payload,"validation":body["validation"],"holdout":body["holdout"]})).map_err(|e|Error::bad(e.to_string()))?;
        if !body["validation"]["approved"].as_bool().unwrap_or(false)
            || !body["holdout"]["passed"].as_bool().unwrap_or(false)
        {
            return Err(Error::bad(
                "Promotion requires an approved validation and separate passing holdout report.",
            ));
        }
        let v = json!({"feedback_id":id,"report":report,"candidate":body["candidate"],"status":"shadow","reviewer_ref":digest(p.subject.as_bytes()),"reason":body["reason"],"at":Utc::now().to_rfc3339()});
        let pid = uuid::Uuid::new_v4().to_string();
        let mut w = write("promotions", &pid, v.clone(), Some(0));
        w.immutable = true;
        self.store
            .commit(&p.tenant, &p.subject, "learning.promote", vec![w], None)
            .await?;
        Ok(v)
    }
    pub async fn create_key(&self, p: &Principal, body: Value) -> Result<Value> {
        p.require("keys.manage")?;
        let principal = string(&body, "principal")?;
        let scopes: Vec<String> =
            serde_json::from_value(body.get("scopes").cloned().unwrap_or_else(|| json!([])))?;
        if scopes.is_empty() || scopes.iter().any(|s| !p.allows(s)) {
            return Err(Error::forbidden(
                "Cannot grant scopes outside the creator's authority.",
            ));
        }
        let groups: Vec<String> =
            serde_json::from_value(body.get("groups").cloned().unwrap_or_else(|| json!([])))?;
        if !p.allows("*") && groups.iter().any(|g| !p.groups.contains(g)) {
            return Err(Error::forbidden("Cannot grant unowned group access."));
        }
        let key = token();
        let c = Credential {
            id: uuid::Uuid::new_v4().to_string(),
            tenant: p.tenant.clone(),
            principal,
            scopes,
            groups,
            expires_at: timestamp(body.get("expires_at")).unwrap_or(0),
            revoked: false,
        };
        self.store
            .add_credential(&digest(key.as_bytes()), &c)
            .await?;
        self.store
            .commit(&p.tenant, &p.subject, "key.created", vec![], None)
            .await?;
        Ok(
            json!({"key":key,"credential":c,"notice":"Store this secret now; it is never returned again."}),
        )
    }
    pub async fn authorize_record(&self, p: &Principal, v: &Value, purpose: &str) -> Result<()> {
        if !p.can_read(v, purpose) {
            return Err(Error::missing());
        }
        if v["owner_ref"]
            .as_str()
            .is_some_and(|owner| owner != digest(p.subject.as_bytes()))
            && !p.allows("data.read_all")
        {
            return Err(Error::missing());
        }
        for id in v["source_ids"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
        {
            let src = self.store.get(&p.tenant, "sources", id).await?;
            if !p.can_read(&src.payload, purpose) {
                return Err(Error::missing());
            }
        }
        Ok(())
    }
    pub async fn authorized_evidence(
        &self,
        p: &Principal,
        purpose: &str,
    ) -> Result<(Vec<Value>, Vec<Value>)> {
        let controls = self.controls(p).await?;
        if controls["sources_enabled"] == false
            || controls["bundle_expires_at"]
                .as_i64()
                .is_some_and(|n| n <= Utc::now().timestamp())
        {
            return Err(Error::forbidden("Source serving authority is unavailable."));
        }
        if self.store.count(&p.tenant, "sources").await? > 5000
            || self.store.count(&p.tenant, "claims").await? > 10000
        {
            return Err(Error::unavailable("Evidence snapshot budget exceeded."));
        }
        let sources: Vec<_> = values(self.store.list(&p.tenant, "sources", 5000, 0).await?)
            .into_iter()
            .filter(|v| p.can_read(v, purpose) && self.source_current(v))
            .collect();
        let claims: Vec<_> = values(self.store.list(&p.tenant, "claims", 10000, 0).await?)
            .into_iter()
            .filter(|v| {
                p.can_read(v, purpose)
                    && v["status"] == "approved"
                    && sources
                        .iter()
                        .any(|s| s["id"] == v["source_id"] && s["revision"] == v["source_revision"])
            })
            .collect();
        Ok((sources, claims))
    }
    pub async fn validate_destination(&self, destination: &str) -> Result<url::Url> {
        let u = url::Url::parse(destination).map_err(|_| Error::bad("Invalid connector URL."))?;
        if u.scheme() != "https" || !u.username().is_empty() || u.password().is_some() {
            return Err(Error::bad(
                "Connector destinations require HTTPS and no embedded credentials.",
            ));
        }
        let host = u
            .host_str()
            .ok_or_else(|| Error::bad("Connector host missing."))?;
        if !self.config.allowed_hosts.iter().any(|s| s == host) {
            return Err(Error::forbidden("Connector host is not allowlisted."));
        }
        let addresses = tokio::net::lookup_host((host, u.port_or_known_default().unwrap_or(443)))
            .await
            .map_err(|_| Error::unavailable("Connector DNS failed."))?;
        let mut any = false;
        for a in addresses {
            any = true;
            if crate::actions::unsafe_address(a.ip()) {
                return Err(Error::forbidden(
                    "Private/internal connector addresses are prohibited.",
                ));
            }
        }
        if !any {
            return Err(Error::unavailable("Connector DNS returned no addresses."));
        }
        Ok(u)
    }
}
