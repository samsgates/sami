use crate::{
    auth::Principal,
    crypto::{canonical, digest},
    error::{Error, Result},
    service::Service,
    storage::Write,
};
use chrono::Utc;
use serde_json::{json, Value};
use std::time::Duration;

fn wr(kind: &str, id: &str, payload: Value, expected: Option<i64>) -> Write {
    Write {
        kind: kind.into(),
        id: id.into(),
        payload,
        expected,
        immutable: false,
    }
}
fn required<'a>(v: &'a Value, k: &str) -> Result<&'a str> {
    v[k].as_str()
        .ok_or_else(|| Error::bad(format!("{k} must be a string.")))
}
pub fn validate_schema(schema: &Value, value: &Value) -> Result<()> {
    if schema["type"] != "object" || !value.is_object() {
        return Err(Error::bad("The tool requires a typed object."));
    }
    if let Some(required) = schema["required"].as_array() {
        for k in required {
            let name = k
                .as_str()
                .ok_or_else(|| Error::bad("Invalid required field schema."))?;
            if value.get(name).is_none() {
                return Err(Error::bad(format!("Required tool field: {name}")));
            }
        }
    }
    if let Some(props) = schema["properties"].as_object() {
        for (key, v) in value.as_object().ok_or_else(Error::internal)? {
            let Some(p) = props.get(key) else {
                if schema["additionalProperties"].as_bool() != Some(true) {
                    return Err(Error::bad(format!("Unknown tool field: {key}")));
                }
                continue;
            };
            let valid = match p["type"].as_str() {
                Some("string") => v
                    .as_str()
                    .is_some_and(|s| s.len() <= p["maxLength"].as_u64().unwrap_or(4000) as usize),
                Some("number") => v.is_number(),
                Some("integer") => v.is_i64() || v.is_u64(),
                Some("boolean") => v.is_boolean(),
                Some("array") => v.is_array(),
                Some("object") => v.is_object(),
                _ => false,
            };
            if !valid {
                return Err(Error::bad(format!("Invalid tool field type: {key}")));
            }
            if let Some(e) = p["enum"].as_array() {
                if !e.contains(v) {
                    return Err(Error::bad(format!("Invalid tool field enum: {key}")));
                }
            }
        }
    }
    Ok(())
}
fn binding(v: &Value) -> Value {
    json!({"tool_id":v["tool_id"],"tool_revision":v["tool_revision"],"decision_id":v["decision_id"],"arguments_digest":v["arguments_digest"],"source_epochs":v["source_epochs"],"pack_revision":v["pack_revision"]})
}
impl Service {
    async fn decision_eligible(&self, p: &Principal, id: &str) -> Result<Value> {
        let d = self.store.get(&p.tenant, "decisions", id).await?;
        if d.payload["eligible"] != true {
            return Err(Error::conflict(
                "Decision authority was invalidated; evaluate again.",
            ));
        }
        if !["resolved", "needs_human"].contains(&d.payload["status"].as_str().unwrap_or("")) {
            return Err(Error::forbidden(
                "The decision has unresolved or insufficient evidence.",
            ));
        }
        if d.payload["evidence"]["complete"] == false
            || d.payload["evidence"]["missing"]
                .as_array()
                .is_some_and(|a| !a.is_empty())
        {
            return Err(Error::forbidden("Mandatory evidence is incomplete."));
        }
        if let Some(sources) = d.payload["source_epochs"].as_object() {
            for (id, stamp) in sources {
                let s = self.store.get(&p.tenant, "sources", id).await?;
                if s.revision != stamp["revision"].as_i64().unwrap_or(-1)
                    || s.payload["epoch"] != stamp["epoch"]
                    || s.payload["status"] != "approved"
                    || !p.can_read(&s.payload, "service_prequalification")
                {
                    return Err(Error::conflict("Decision source authority changed."));
                }
                if s.payload["authority_mode"] != "local"
                    && crate::auth::timestamp(s.payload.get("permission_checked_at")).unwrap_or(0)
                        + s.payload["max_age_seconds"]
                            .as_i64()
                            .unwrap_or(300)
                            .clamp(1, 300)
                        <= Utc::now().timestamp()
                {
                    return Err(Error::forbidden("Upstream permission evidence is stale."));
                }
            }
        }
        let pack = self
            .store
            .get(
                &p.tenant,
                "packs",
                d.payload["pack_id"].as_str().unwrap_or(""),
            )
            .await?;
        if pack.revision != d.payload["pack_revision"].as_i64().unwrap_or(-1)
            || pack.payload["status"] != "approved"
        {
            return Err(Error::conflict("Domain pack authority changed."));
        }
        Ok(d.payload)
    }
    pub async fn propose_action(&self, p: &Principal, body: Value) -> Result<Value> {
        p.require("action.propose")?;
        let decision_id = required(&body, "decision_id")?;
        let tool_id = required(&body, "tool_id")?;
        let d = self.decision_eligible(p, decision_id).await?;
        let tool = self.store.get(&p.tenant, "tools", tool_id).await?;
        self.authorize_record(p, &tool.payload, "service_prequalification")
            .await?;
        if tool.payload["effect_class"] != "administrative_write"
            || tool.payload["enabled"] == false
        {
            return Err(Error::forbidden(
                "Only registered enabled administrative writes are permitted.",
            ));
        }
        validate_schema(&tool.payload["input_schema"], &body["arguments"])?;
        for scope in tool.payload["required_scopes"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
        {
            p.require(scope)?;
        }
        let id = uuid::Uuid::new_v4().to_string();
        let v = json!({"id":id,"state":"awaiting_approval","decision_id":decision_id,"tool_id":tool_id,"tool_revision":tool.revision,"arguments":body["arguments"],"arguments_digest":digest(&canonical(&body["arguments"])),"source_epochs":d["source_epochs"],"pack_revision":d["pack_revision"],"source_ids":d["source_ids"],"owner_ref":digest(p.subject.as_bytes()),"created_by_ref":digest(p.subject.as_bytes()),"created_at":Utc::now().to_rfc3339()});
        self.store
            .commit(
                &p.tenant,
                &p.subject,
                "action.propose",
                vec![wr("actions", &id, v.clone(), Some(0))],
                None,
            )
            .await?;
        Ok(v)
    }
    pub async fn approve_action(&self, p: &Principal, id: &str, body: Value) -> Result<Value> {
        p.require("action.approve")?;
        let reason = required(&body, "reason")?;
        if reason.is_empty() {
            return Err(Error::bad("Approval reason is required."));
        }
        let epoch = self.store.epoch(&p.tenant).await?;
        let r = self.store.get(&p.tenant, "actions", id).await?;
        let mut v = r.payload;
        if !["awaiting_approval", "proposed"].contains(&v["state"].as_str().unwrap_or("")) {
            return Err(Error::conflict("Action is not awaiting approval."));
        }
        self.decision_eligible(p, required(&v, "decision_id")?)
            .await?;
        let expiry = Utc::now().timestamp()
            + body["expires_in_seconds"]
                .as_i64()
                .unwrap_or(300)
                .clamp(30, 3600);
        let approval = json!({"action_id":id,"binding":binding(&v),"actor_ref":digest(p.subject.as_bytes()),"tenant_ref":digest(p.tenant.as_bytes()),"expires_at":expiry,"nonce":uuid::Uuid::new_v4().to_string(),"reason_digest":digest(reason.as_bytes()),"approver":p});
        let aid = uuid::Uuid::new_v4().to_string();
        let receipt = json!({"envelope":approval,"signature":self.config.crypto.sign(&approval),"public_key":self.config.crypto.public_key()});
        v["state"] = json!("authorized");
        v["approval_id"] = json!(aid);
        v["approval_expires_at"] = json!(expiry);
        v["approval_binding"] = json!(digest(&canonical(&binding(&v))));
        let mut aw = wr("approvals", &aid, receipt, Some(0));
        aw.immutable = true;
        self.store
            .commit(
                &p.tenant,
                &p.subject,
                "action.approve",
                vec![wr("actions", id, v.clone(), Some(r.revision)), aw],
                Some(epoch),
            )
            .await?;
        Ok(v)
    }
    pub async fn execute_action(
        &self,
        p: &Principal,
        id: &str,
        idempotency: &str,
    ) -> Result<Value> {
        p.require("action.execute")?;
        if idempotency.len() < 8 || idempotency.len() > 255 {
            return Err(Error::bad(
                "Execution needs an Idempotency-Key of 8-255 characters.",
            ));
        }
        let epoch = self.store.epoch(&p.tenant).await?;
        self.auth.still_authorized(p).await?;
        let controls = self.controls(p).await?;
        if controls["bundle_expires_at"]
            .as_i64()
            .is_some_and(|n| n <= Utc::now().timestamp())
        {
            return Err(Error::forbidden("Offline lease expired."));
        }
        if controls["effects_enabled"] != true {
            return Err(Error::forbidden("Tool effects are suspended."));
        }
        let r = self.store.get(&p.tenant, "actions", id).await?;
        let mut action = r.payload;
        if let Some(k) = action["idempotency_digest"].as_str() {
            if k != digest(idempotency.as_bytes()) {
                return Err(Error::conflict("An action's retry key cannot change."));
            }
        }
        if action["state"] == "succeeded" {
            return Ok(action);
        }
        if ["executing", "unknown_completion"].contains(&action["state"].as_str().unwrap_or("")) {
            return self.reconcile_action(p, id).await;
        }
        if action["state"] != "authorized" {
            return Err(Error::forbidden("Fresh approval is required."));
        }
        if action["approval_expires_at"].as_i64().unwrap_or(0) <= Utc::now().timestamp() {
            return Err(Error::forbidden("Action approval expired."));
        }
        if action["approval_binding"] != digest(&canonical(&binding(&action))) {
            return Err(Error::conflict("Approved effect binding changed."));
        }
        let approval = self
            .store
            .get(&p.tenant, "approvals", required(&action, "approval_id")?)
            .await?;
        if !self.config.crypto.verify(
            &approval.payload["envelope"],
            approval.payload["signature"].as_str().unwrap_or(""),
        ) {
            return Err(Error::forbidden("Approval signature is invalid."));
        }
        let approver: Principal =
            serde_json::from_value(approval.payload["envelope"]["approver"].clone())?;
        if approver.tenant != p.tenant {
            return Err(Error::forbidden("Approval tenant mismatch."));
        }
        approver.require("action.approve")?;
        self.auth.still_authorized(&approver).await?;
        self.decision_eligible(&approver, required(&action, "decision_id")?)
            .await?;
        self.decision_eligible(p, required(&action, "decision_id")?)
            .await?;
        let tool = self
            .store
            .get(&p.tenant, "tools", required(&action, "tool_id")?)
            .await?;
        if tool.revision != action["tool_revision"].as_i64().unwrap_or(-1)
            || tool.payload["enabled"] == false
        {
            return Err(Error::conflict("Tool authority changed."));
        }
        for scope in tool.payload["required_scopes"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
        {
            p.require(scope)?;
        }
        validate_schema(&tool.payload["input_schema"], &action["arguments"])?;
        if tool.payload["adapter"] == "http_json" {
            if tool.payload["idempotency_supported"] != true {
                return Err(Error::bad(
                    "Remote effects require declared destination idempotency.",
                ));
            }
            self.validate_destination(required(&tool.payload, "url")?)
                .await?;
            if let Some(env) = tool.payload["secret_env"].as_str() {
                if !env.starts_with("SAMI_CONNECTOR_SECRET_") || std::env::var(env).is_err() {
                    return Err(Error::unavailable(
                        "Connector secret binding is invalid or not configured.",
                    ));
                }
            }
        }
        action["state"] = json!("executing");
        action["idempotency_digest"] = json!(digest(idempotency.as_bytes()));
        action["operation_id"] = json!(format!("sami_{id}"));
        action["dispatch_committed_at"] = json!(Utc::now().to_rfc3339());
        if tool.payload["adapter"] == "internal_ticket" {
            let ticket_id = format!("ticket_{id}");
            let result = json!({"ticket_id":ticket_id,"confirmed":true});
            let ticket = json!({"arguments":action["arguments"],"action_id":id,"created_at":Utc::now().to_rfc3339(),"status":"open"});
            action["state"] = json!("succeeded");
            action["result"] = result.clone();
            action["completed_at"] = json!(Utc::now().to_rfc3339());
            let receipt = json!({"action_id":id,"operation_id":action["operation_id"],"result_digest":digest(&canonical(&result)),"confirmed_at":Utc::now().to_rfc3339(),"state":"succeeded"});
            let mut rw = wr(
                "effect_receipts",
                &format!("effect_{id}"),
                json!({"envelope":receipt,"signature":self.config.crypto.sign(&receipt),"public_key":self.config.crypto.public_key()}),
                Some(0),
            );
            rw.immutable = true;
            self.store
                .commit(
                    &p.tenant,
                    &p.subject,
                    "action.internal_effect",
                    vec![
                        wr("actions", id, action.clone(), Some(r.revision)),
                        wr("tickets", &ticket_id, ticket, Some(0)),
                        rw,
                    ],
                    Some(epoch),
                )
                .await?;
            return Ok(action);
        }
        // Serializes with source publication, policy changes, ACL changes and credential revocation.
        self.store
            .commit(
                &p.tenant,
                &p.subject,
                "action.dispatch_committed",
                vec![wr("actions", id, action.clone(), Some(r.revision))],
                Some(epoch),
            )
            .await?;
        match tool.payload["adapter"].as_str() {
            Some("internal_ticket") => {
                let ticket_id = format!("ticket_{id}");
                let ticket = json!({"id":ticket_id,"arguments":action["arguments"],"action_id":id,"created_at":Utc::now().to_rfc3339(),"status":"open"});
                self.complete_action(
                    p,
                    id,
                    action,
                    json!({"ticket_id":ticket_id,"confirmed":true}),
                    Some(wr("tickets", &ticket_id, ticket, Some(0))),
                )
                .await
            }
            Some("http_json") => {
                if tool.payload["idempotency_supported"] != true {
                    return self
                        .unknown_action(
                            p,
                            id,
                            action,
                            "Connector idempotency capability has not been declared.",
                        )
                        .await;
                }
                let u = self
                    .validate_destination(required(&tool.payload, "url")?)
                    .await?;
                let host = u.host_str().ok_or_else(Error::internal)?;
                let addrs: Vec<_> =
                    tokio::net::lookup_host((host, u.port_or_known_default().unwrap_or(443)))
                        .await
                        .map_err(|_| Error::unavailable("Connector DNS unavailable."))?
                        .collect();
                // Validate every resolved address and pin the lookup into this request client.
                if addrs.is_empty() || addrs.iter().any(|a| unsafe_address(a.ip())) {
                    return self
                        .unknown_action(
                            p,
                            id,
                            action,
                            "Connector address policy rejected dispatch.",
                        )
                        .await;
                }
                let client = reqwest::Client::builder()
                    .timeout(Duration::from_secs(10))
                    .redirect(reqwest::redirect::Policy::none())
                    .resolve_to_addrs(host, &addrs)
                    .build()
                    .map_err(|_| Error::internal())?;
                let mut request = client
                    .post(u)
                    .header("Idempotency-Key", format!("sami_{id}"))
                    .json(&action["arguments"]);
                if let Some(env) = tool.payload["secret_env"].as_str() {
                    if !env.starts_with("SAMI_CONNECTOR_SECRET_") {
                        return self
                            .unknown_action(p, id, action, "Invalid connector secret binding.")
                            .await;
                    }
                    let secret = std::env::var(env)
                        .map_err(|_| Error::unavailable("Connector secret is not configured."))?;
                    request = request.bearer_auth(secret);
                }
                if let Some(version) = action["arguments"]["resource_version"].as_str() {
                    request = request.header("If-Match", version);
                }
                match request.send().await {
                    Ok(resp) if resp.status().is_success() => {
                        let result = read_bounded_json(resp).await;
                        match result{Ok(v)=>self.complete_action(p,id,action,v,None).await,Err(_)=>self.unknown_action(p,id,action,"External response was invalid or too large; reconcile before retry.").await}
                    }
                    Ok(resp) if resp.status().as_u16() == 409 || resp.status().as_u16() == 412 => {
                        let mut v = action;
                        v["state"] = json!("failed");
                        v["reason"] =
                            json!("External precondition conflict; new evaluation required.");
                        self.store
                            .commit(
                                &p.tenant,
                                &p.subject,
                                "action.failed",
                                vec![wr("actions", id, v.clone(), None)],
                                None,
                            )
                            .await?;
                        Ok(v)
                    }
                    _ => {
                        self.unknown_action(
                            p,
                            id,
                            action,
                            "External completion is uncertain; reconcile before any retry.",
                        )
                        .await
                    }
                }
            }
            _ => {
                self.unknown_action(p, id, action, "Unsupported connector adapter.")
                    .await
            }
        }
    }
    async fn complete_action(
        &self,
        p: &Principal,
        id: &str,
        mut action: Value,
        result: Value,
        extra: Option<Write>,
    ) -> Result<Value> {
        let current = self.store.get(&p.tenant, "actions", id).await?;
        if current.payload["state"] == "succeeded" {
            return Ok(current.payload);
        }
        if !["executing", "unknown_completion"]
            .contains(&current.payload["state"].as_str().unwrap_or(""))
        {
            return Err(Error::conflict("Completion cannot regress action state."));
        }
        action["state"] = json!("succeeded");
        action["completed_at"] = json!(Utc::now().to_rfc3339());
        action["result"] = result.clone();
        let receipt = json!({"action_id":id,"operation_id":action["operation_id"],"result_digest":digest(&canonical(&result)),"confirmed_at":Utc::now().to_rfc3339(),"state":"succeeded"});
        let rid = format!("effect_{id}");
        let mut receipt_write = wr(
            "effect_receipts",
            &rid,
            json!({"envelope":receipt,"signature":self.config.crypto.sign(&receipt),"public_key":self.config.crypto.public_key()}),
            Some(0),
        );
        receipt_write.immutable = true;
        let mut writes = vec![
            wr("actions", id, action.clone(), Some(current.revision)),
            receipt_write,
        ];
        if let Some(w) = extra {
            writes.push(w)
        }
        self.store
            .commit(&p.tenant, &p.subject, "action.succeeded", writes, None)
            .await?;
        Ok(action)
    }
    async fn unknown_action(
        &self,
        p: &Principal,
        id: &str,
        mut action: Value,
        reason: &str,
    ) -> Result<Value> {
        let current = self.store.get(&p.tenant, "actions", id).await?;
        if current.payload["state"] == "succeeded" {
            return Ok(current.payload);
        }
        action["state"] = json!("unknown_completion");
        action["reason"] = json!(reason);
        self.store
            .commit(
                &p.tenant,
                &p.subject,
                "action.unknown",
                vec![wr("actions", id, action.clone(), Some(current.revision))],
                None,
            )
            .await?;
        Ok(action)
    }
    pub async fn reconcile_action(&self, p: &Principal, id: &str) -> Result<Value> {
        p.require("action.reconcile")?;
        let r = self.store.get(&p.tenant, "actions", id).await?;
        let mut v = r.payload;
        if v["state"] == "succeeded" {
            return Ok(v);
        }
        if !["executing", "unknown_completion"].contains(&v["state"].as_str().unwrap_or("")) {
            return Err(Error::conflict("Action does not need reconciliation."));
        }
        let tool = self
            .store
            .get(&p.tenant, "tools", required(&v, "tool_id")?)
            .await?;
        if tool.payload["adapter"] == "internal_ticket" {
            let tid = format!("ticket_{id}");
            if let Ok(t) = self.store.get(&p.tenant, "tickets", &tid).await {
                return self
                    .complete_action(
                        p,
                        id,
                        v,
                        json!({"ticket_id":t.id,"confirmed":true,"reconciled":true}),
                        None,
                    )
                    .await;
            }
            // Internal ticket and completion are one transaction: absence proves this local effect did not commit.
            v["state"] = json!("authorized");
            v["reason"] = json!(
                "Confirmed local absence; retry requires still-fresh authority and same key."
            );
            self.store
                .commit(
                    &p.tenant,
                    &p.subject,
                    "action.reconciled_absent",
                    vec![wr("actions", id, v.clone(), Some(r.revision))],
                    None,
                )
                .await?;
            return Ok(v);
        }
        if let Some(base) = tool.payload["reconcile_url"].as_str() {
            let mut u = self.validate_destination(base).await?;
            u.query_pairs_mut()
                .append_pair("operation_id", &format!("sami_{id}"));
            let host = u.host_str().ok_or_else(Error::internal)?;
            let addrs: Vec<_> =
                tokio::net::lookup_host((host, u.port_or_known_default().unwrap_or(443)))
                    .await
                    .map_err(|_| Error::unavailable("Reconciliation DNS unavailable."))?
                    .collect();
            if addrs.is_empty() || addrs.iter().any(|a| unsafe_address(a.ip())) {
                return Err(Error::forbidden("Reconciliation address denied."));
            }
            let client = reqwest::Client::builder()
                .redirect(reqwest::redirect::Policy::none())
                .timeout(Duration::from_secs(10))
                .resolve_to_addrs(host, &addrs)
                .build()
                .map_err(|_| Error::internal())?;
            let mut request = client.get(u);
            if let Some(env) = tool.payload["secret_env"].as_str() {
                if !env.starts_with("SAMI_CONNECTOR_SECRET_") {
                    return Err(Error::forbidden("Invalid connector secret binding."));
                }
                request = request.bearer_auth(
                    std::env::var(env)
                        .map_err(|_| Error::unavailable("Connector secret missing."))?,
                )
            }
            let response = request
                .send()
                .await
                .map_err(|_| Error::unavailable("Reconciliation endpoint unavailable."))?;
            if response.status().is_success() {
                let result = read_bounded_json(response).await?;
                if result["confirmed"] == true {
                    return self.complete_action(p, id, v, result, None).await;
                }
            }
        }
        v["state"] = json!("unknown_completion");
        v["reason"]=json!("No authoritative completion evidence; human resolution required. No effect was retried.");
        self.store
            .commit(
                &p.tenant,
                &p.subject,
                "action.reconciliation_pending",
                vec![wr("actions", id, v.clone(), Some(r.revision))],
                None,
            )
            .await?;
        Ok(v)
    }
}
pub fn unsafe_address(ip: std::net::IpAddr) -> bool {
    match ip {
        std::net::IpAddr::V4(v) => {
            let o = v.octets();
            v.is_private()
                || v.is_loopback()
                || v.is_link_local()
                || v.is_unspecified()
                || v.is_multicast()
                || v.is_broadcast()
                || o[0] == 0
                || o[0] >= 240
                || (o[0] == 100 && (64..=127).contains(&o[1]))
                || (o[0] == 192 && o[1] == 0)
                || (o[0] == 198 && (o[1] == 18 || o[1] == 19 || o[1] == 51))
                || (o[0] == 203 && o[1] == 0 && o[2] == 113)
        }
        std::net::IpAddr::V6(v) => v
            .to_ipv4_mapped()
            .map(|v| unsafe_address(std::net::IpAddr::V4(v)))
            .unwrap_or_else(|| {
                v.is_loopback()
                    || v.is_unspecified()
                    || v.is_multicast()
                    || v.segments()[0] & 0xe000 != 0x2000
                    || v.segments()[0] == 0x2001
                        && (v.segments()[1] == 0x0db8 || v.segments()[1] == 0)
            }),
    }
}
async fn read_bounded_json(mut response: reqwest::Response) -> Result<Value> {
    let mut bytes = Vec::new();
    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|_| Error::unavailable("Connector response could not be read."))?
    {
        if bytes.len() + chunk.len() > 1_048_576 {
            return Err(Error::bad("Connector response exceeds 1 MiB."));
        }
        bytes.extend(chunk);
    }
    Ok(serde_json::from_slice(&bytes)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn schemas_reject_unapproved_fields() {
        let s = json!({"type":"object","required":["title"],"properties":{"title":{"type":"string","maxLength":4}}});
        assert!(validate_schema(&s, &json!({"title":"safe"})).is_ok());
        assert!(validate_schema(&s, &json!({"title":"safe","shell":"rm"})).is_err());
        assert!(validate_schema(&s, &json!({"title":4})).is_err());
    }
}
