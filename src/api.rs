use crate::{
    auth::Principal,
    crypto::{canonical, digest, verify_public, Crypto},
    error::{Error, Result},
    service::Service,
    storage::Write,
};
use axum::{
    extract::{DefaultBodyLimit, Path, Query, Request, State},
    http::{header, HeaderMap, StatusCode},
    middleware::{self, Next},
    response::{
        sse::{Event, KeepAlive, Sse},
        IntoResponse, Response,
    },
    routing::{delete, get, post},
    Extension, Json, Router,
};
use serde::Deserialize;
use serde_json::{json, Value};
use std::{
    convert::Infallible,
    sync::{atomic::Ordering, Arc},
    time::Duration,
};

type App = Arc<Service>;
pub fn router(service: App) -> Router {
    let protected = Router::new()
        .route("/v1/capabilities", get(capabilities))
        .route("/v1/me", get(me))
        .route("/v1/openapi", get(openapi))
        .route("/v1/admin/:kind", get(admin_list).post(admin_save))
        .route("/v1/admin/:kind/:id", get(admin_get))
        .route("/v1/admin/:kind/:id/history", get(history))
        .route("/v1/sources/:id", delete(source_delete))
        .route("/v1/sources/:id/publish", post(source_publish))
        .route("/v1/sources/:id/retract", post(source_retract))
        .route("/v1/sources/:id/permissions", post(source_permissions))
        .route("/v1/claims/candidates", post(claim_candidate))
        .route("/v1/packs/:id/publish", post(pack_publish))
        .route("/v1/decisions", post(decide))
        .route("/v1/decide", post(decide))
        .route("/v1/respond", post(respond))
        .route("/v1/chat/completions", post(compatibility))
        .route("/v1/receipts/:id", get(receipt))
        .route("/v1/receipts/:id/replay", post(replay))
        .route("/v1/actions/propose", post(action_propose))
        .route("/v1/actions/:id/approve", post(action_approve))
        .route("/v1/actions/:id/execute", post(action_execute))
        .route("/v1/actions/:id/reconcile", post(action_reconcile))
        .route("/v1/actions/:id", get(action_get))
        .route("/v1/feedback", post(feedback))
        .route("/v1/feedback/:id/promote", post(promote))
        .route("/v1/sessions", post(session_create))
        .route("/v1/sessions/:id", get(session_get))
        .route("/v1/knowledge/search", post(search))
        .route("/v1/memory/query", post(search))
        .route("/v1/memory/write", post(claim_candidate))
        .route("/v1/graph/query", post(graph))
        .route("/v1/research/:operation", post(research))
        .route("/v1/keys", get(keys).post(key_create))
        .route("/v1/keys/:id/revoke", post(key_revoke))
        .route("/v1/audit", get(audit))
        .route("/v1/events", get(events))
        .route("/v1/export", get(export))
        .route("/v1/bundles/export", post(bundle_export))
        .route("/v1/bundles/import", post(bundle_import))
        .route("/v1/workflows/:id/run", post(workflow_start))
        .route("/v1/workflow-runs/:id/advance", post(workflow_advance))
        .route("/v1/workflow-runs/:id/cancel", post(workflow_cancel))
        .route("/v1/roleplay", post(roleplay))
        .route("/v1/policy/diff", post(policy_diff))
        .route_layer(middleware::from_fn_with_state(
            service.clone(),
            authenticate,
        ));
    Router::new()
        .route("/health", get(health))
        .route("/ready", get(ready))
        .route("/metrics", get(metrics))
        .merge(protected)
        .nest_service(
            "/console",
            tower_http::services::ServeDir::new(service.config.console_dir.clone())
                .append_index_html_on_directories(true),
        )
        .layer(DefaultBodyLimit::max(4 * 1024 * 1024))
        .layer(tower_http::trace::TraceLayer::new_for_http())
        .layer(middleware::from_fn(security_headers))
        .with_state(service)
}
async fn authenticate(State(s): State<App>, mut request: Request, next: Next) -> Result<Response> {
    let bearer = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .ok_or_else(|| Error {
            status: StatusCode::UNAUTHORIZED,
            code: "AUTH_REQUIRED",
            message: "A bearer credential is required.".into(),
        })?;
    let p = s.auth.authenticate(bearer).await?;
    s.requests.fetch_add(1, Ordering::Relaxed);
    request.extensions_mut().insert(p);
    Ok(next.run(request).await)
}
async fn security_headers(request: Request, next: Next) -> Response {
    let mut r = next.run(request).await;
    let h = r.headers_mut();
    for (k,v) in [("x-content-type-options","nosniff"),("referrer-policy","no-referrer"),("x-frame-options","DENY"),("cache-control","no-store"),("content-security-policy","default-src 'self'; script-src 'self'; style-src 'self'; connect-src 'self'; img-src 'self' data:; frame-ancestors 'none'; base-uri 'self'")]{h.insert(axum::http::HeaderName::from_static(k),axum::http::HeaderValue::from_static(v));}
    r
}
async fn openapi() -> Json<Value> {
    Json(
        serde_json::from_str(include_str!("../docs/openapi.json"))
            .expect("checked OpenAPI descriptor"),
    )
}
async fn health() -> Json<Value> {
    Json(json!({"status":"ok","service":"sami","version":env!("CARGO_PKG_VERSION")}))
}
async fn ready(State(s): State<App>) -> Result<Json<Value>> {
    s.store
        .pool
        .acquire()
        .await
        .map_err(|_| Error::unavailable("Durable store unavailable."))?;
    Ok(Json(json!({"status":"ready","profile":"native_strict"})))
}
async fn metrics(State(s): State<App>, headers: HeaderMap) -> Result<String> {
    let bearer = headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or_else(|| Error::forbidden("Metrics require authentication."))?;
    let p = s.auth.authenticate(bearer).await?;
    p.require("metrics.read")?;
    Ok(format!("# TYPE sami_requests_total counter\nsami_requests_total {}\n# TYPE sami_decisions_total counter\nsami_decisions_total {}\n",s.requests.load(Ordering::Relaxed),s.decisions.load(Ordering::Relaxed)))
}
async fn me(Extension(p): Extension<Principal>) -> Json<Value> {
    Json(json!(p))
}
async fn capabilities(
    State(s): State<App>,
    Extension(p): Extension<Principal>,
) -> Result<Json<Value>> {
    Ok(Json(
        json!({"profile":"native_strict","version":env!("CARGO_PKG_VERSION"),"tenant":p.tenant,"signing_public_key":s.config.crypto.public_key(),"research":sami_research::dispatch("capabilities",&json!({})).map_err(|e|Error::bad(e.to_string()))?,"readiness":"software tests do not certify customer deployment or PRD GA targets","limits":{"max_body_bytes":4194304,"max_claims_per_native_snapshot":10000,"max_sources_per_snapshot":5000},"optional_models":false}),
    ))
}
#[derive(Deserialize)]
struct Page {
    #[serde(default = "default_limit")]
    limit: i64,
    #[serde(default)]
    offset: i64,
    #[serde(default)]
    after: i64,
}
fn default_limit() -> i64 {
    100
}
async fn admin_list(
    State(s): State<App>,
    Extension(p): Extension<Principal>,
    Path(kind): Path<String>,
    Query(q): Query<Page>,
) -> Result<Json<Value>> {
    Ok(Json(
        s.list(&p, &kind, q.limit.clamp(1, 500), q.offset).await?,
    ))
}
async fn admin_get(
    State(s): State<App>,
    Extension(p): Extension<Principal>,
    Path((k, id)): Path<(String, String)>,
) -> Result<Json<Value>> {
    Ok(Json(s.get(&p, &k, &id).await?))
}
async fn history(
    State(s): State<App>,
    Extension(p): Extension<Principal>,
    Path((k, id)): Path<(String, String)>,
) -> Result<Json<Value>> {
    s.get(&p, &k, &id).await?;
    let items: Vec<_> = s
        .store
        .history(&p.tenant, &k, &id)
        .await?
        .into_iter()
        .filter(|v| p.can_read(&v["payload"], "administration"))
        .collect();
    Ok(Json(json!({"items":items})))
}
async fn admin_save(
    State(s): State<App>,
    Extension(p): Extension<Principal>,
    Path(k): Path<String>,
    Json(v): Json<Value>,
) -> Result<Json<Value>> {
    Ok(Json(s.save(&p, &k, v).await?))
}
async fn source_publish(
    State(s): State<App>,
    Extension(p): Extension<Principal>,
    Path(id): Path<String>,
    Json(v): Json<Value>,
) -> Result<Json<Value>> {
    Ok(Json(s.publish_source(&p, &id, v, false).await?))
}
async fn source_retract(
    State(s): State<App>,
    Extension(p): Extension<Principal>,
    Path(id): Path<String>,
    Json(v): Json<Value>,
) -> Result<Json<Value>> {
    Ok(Json(s.publish_source(&p, &id, v, true).await?))
}
async fn source_delete(
    State(s): State<App>,
    Extension(p): Extension<Principal>,
    Path(id): Path<String>,
) -> Result<Json<Value>> {
    Ok(Json(s.delete_source(&p, &id).await?))
}
async fn source_permissions(
    State(s): State<App>,
    Extension(p): Extension<Principal>,
    Path(id): Path<String>,
    Json(v): Json<Value>,
) -> Result<Json<Value>> {
    p.require("sources.permissions")?;
    let r = s.store.get(&p.tenant, "sources", &id).await?;
    s.authorize_record(&p, &r.payload, "administration").await?;
    let mut src = r.payload;
    src["acl"] = v.get("acl").cloned().unwrap_or(src["acl"].clone());
    src["permission_checked_at"] = json!(chrono::Utc::now().to_rfc3339());
    src["permission_cursor"] = v["cursor"].clone();
    src["authority_mode"] = json!("upstream");
    src["epoch"] = json!(src["epoch"].as_i64().unwrap_or(0) + 1);
    s.store
        .commit(
            &p.tenant,
            &p.subject,
            "source.permissions",
            vec![Write {
                kind: "sources".into(),
                id: id.clone(),
                payload: src,
                expected: Some(r.revision),
                immutable: false,
            }],
            None,
        )
        .await?;
    Ok(Json(
        json!({"status":"updated","source":s.store.get(&p.tenant,"sources",&id).await?.payload}),
    ))
}
async fn claim_candidate(
    State(s): State<App>,
    Extension(p): Extension<Principal>,
    Json(mut v): Json<Value>,
) -> Result<Json<Value>> {
    if !v.is_object() {
        return Err(Error::bad("Claim candidate must be an object."));
    }
    v["status"] = json!("candidate");
    Ok(Json(s.save(&p, "claims", v).await?))
}
async fn pack_publish(
    State(s): State<App>,
    Extension(p): Extension<Principal>,
    Path(id): Path<String>,
    Json(v): Json<Value>,
) -> Result<Json<Value>> {
    Ok(Json(s.publish_pack(&p, &id, v).await?))
}
async fn decide(
    State(s): State<App>,
    Extension(p): Extension<Principal>,
    Json(v): Json<Value>,
) -> Result<Json<Value>> {
    Ok(Json(s.decide(&p, v).await?))
}
async fn receipt(
    State(s): State<App>,
    Extension(p): Extension<Principal>,
    Path(id): Path<String>,
) -> Result<Json<Value>> {
    Ok(Json(s.receipt(&p, &id, false).await?))
}
async fn replay(
    State(s): State<App>,
    Extension(p): Extension<Principal>,
    Path(id): Path<String>,
) -> Result<Json<Value>> {
    Ok(Json(s.receipt(&p, &id, true).await?))
}
async fn action_propose(
    State(s): State<App>,
    Extension(p): Extension<Principal>,
    Json(v): Json<Value>,
) -> Result<Json<Value>> {
    Ok(Json(s.propose_action(&p, v).await?))
}
async fn action_approve(
    State(s): State<App>,
    Extension(p): Extension<Principal>,
    Path(id): Path<String>,
    Json(v): Json<Value>,
) -> Result<Json<Value>> {
    Ok(Json(s.approve_action(&p, &id, v).await?))
}
async fn action_execute(
    State(s): State<App>,
    Extension(p): Extension<Principal>,
    Path(id): Path<String>,
    headers: HeaderMap,
) -> Result<Json<Value>> {
    let key = headers
        .get("Idempotency-Key")
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| Error::bad("Idempotency-Key is required."))?;
    Ok(Json(s.execute_action(&p, &id, key).await?))
}
async fn action_reconcile(
    State(s): State<App>,
    Extension(p): Extension<Principal>,
    Path(id): Path<String>,
) -> Result<Json<Value>> {
    Ok(Json(s.reconcile_action(&p, &id).await?))
}
async fn action_get(
    State(s): State<App>,
    Extension(p): Extension<Principal>,
    Path(id): Path<String>,
) -> Result<Json<Value>> {
    p.require("action.read")?;
    let action = s.store.get(&p.tenant, "actions", &id).await?.payload;
    s.authorize_record(&p, &action, "service_prequalification")
        .await?;
    Ok(Json(action))
}
async fn feedback(
    State(s): State<App>,
    Extension(p): Extension<Principal>,
    Json(v): Json<Value>,
) -> Result<Json<Value>> {
    Ok(Json(s.feedback(&p, v).await?))
}
async fn promote(
    State(s): State<App>,
    Extension(p): Extension<Principal>,
    Path(id): Path<String>,
    Json(v): Json<Value>,
) -> Result<Json<Value>> {
    Ok(Json(s.promote(&p, &id, v).await?))
}
async fn keys(State(s): State<App>, Extension(p): Extension<Principal>) -> Result<Json<Value>> {
    p.require("keys.manage")?;
    Ok(Json(json!({"items":s.store.credentials(&p.tenant).await?})))
}
async fn key_create(
    State(s): State<App>,
    Extension(p): Extension<Principal>,
    Json(v): Json<Value>,
) -> Result<Json<Value>> {
    Ok(Json(s.create_key(&p, v).await?))
}
async fn key_revoke(
    State(s): State<App>,
    Extension(p): Extension<Principal>,
    Path(id): Path<String>,
) -> Result<Json<Value>> {
    p.require("keys.manage")?;
    s.store.revoke(&p.tenant, &id).await?;
    Ok(Json(json!({"status":"revoked"})))
}
async fn audit(
    State(s): State<App>,
    Extension(p): Extension<Principal>,
    Query(q): Query<Page>,
) -> Result<Json<Value>> {
    p.require("audit.read")?;
    Ok(Json(
        json!({"items":s.store.events(&p.tenant,q.after,q.limit).await?}),
    ))
}
async fn session_create(
    State(s): State<App>,
    Extension(p): Extension<Principal>,
    Json(mut v): Json<Value>,
) -> Result<Json<Value>> {
    p.require("session.write")?;
    if !v.is_object() {
        return Err(Error::bad("Session must be an object."));
    }
    v["state"] = v.get("state").cloned().unwrap_or_else(|| json!({}));
    v["turns"] = json!([]);
    v["owner_ref"] = json!(digest(p.subject.as_bytes()));
    let id = uuid::Uuid::new_v4().to_string();
    s.store
        .commit(
            &p.tenant,
            &p.subject,
            "session.create",
            vec![Write {
                kind: "sessions".into(),
                id: id.clone(),
                payload: v,
                expected: Some(0),
                immutable: false,
            }],
            None,
        )
        .await?;
    Ok(Json(s.store.get(&p.tenant, "sessions", &id).await?.payload))
}
async fn session_get(
    State(s): State<App>,
    Extension(p): Extension<Principal>,
    Path(id): Path<String>,
) -> Result<Json<Value>> {
    p.require("session.read")?;
    let r = s.store.get(&p.tenant, "sessions", &id).await?;
    s.authorize_record(&p, &r.payload, "service_prequalification")
        .await?;
    Ok(Json(r.payload))
}
async fn respond(
    State(s): State<App>,
    Extension(p): Extension<Principal>,
    Json(v): Json<Value>,
) -> Result<Json<Value>> {
    p.require("session.write")?;
    let sid = v["session_id"]
        .as_str()
        .ok_or_else(|| Error::bad("session_id required."))?;
    let message = v["message"]
        .as_str()
        .ok_or_else(|| Error::bad("message required."))?;
    if message.len() > 16000 {
        return Err(Error::bad("Message exceeds supported length."));
    }
    let r = s.store.get(&p.tenant, "sessions", sid).await?;
    s.authorize_record(&p, &r.payload, "service_prequalification")
        .await?;
    if v["revision"].as_i64().is_some_and(|n| n != r.revision) {
        return Err(Error::conflict("Session revision changed."));
    }
    let pack = s
        .store
        .get(
            &p.tenant,
            "packs",
            v["pack_id"].as_str().unwrap_or("industrial-service"),
        )
        .await?;
    let parse = sami_intelligence::parse(message, &r.payload["state"], &pack.payload);
    let mut fields = r.payload["state"]["fields"].clone();
    if !fields.is_object() {
        fields = json!({})
    }
    if parse["fields"]["serial_number"]
        .as_str()
        .is_some_and(|serial| {
            fields["serial_number"]
                .as_str()
                .is_some_and(|old| old != serial)
        })
    {
        fields = json!({});
    }
    if let Some(new) = parse["fields"].as_object() {
        for (k, value) in new {
            fields[k] = value.clone();
        }
    }
    let mut input = json!({"message":message,"fields":fields,"state":r.payload["state"]});
    if let Some(fields) = v["fields"].as_object() {
        for (k, value) in fields {
            input["fields"][k] = value.clone();
        }
    }
    let d=s.decide(&p,json!({"task_id":v["task_id"].as_str().unwrap_or("service_triage"),"input":input.clone(),"pack_id":v["pack_id"].as_str().unwrap_or("industrial-service")})).await?;
    let mut state = r.payload;
    state["source_ids"] = d["source_ids"].clone();
    state["state"] = json!({"fields":input["fields"],"last_decision":d["decision_id"],"parse_status":parse["status"],"assertion_status":"unconfirmed","conflicts":d["evidence"]["conflicts"],"provenance":"conversation assertion; approved asset claims remain separate"});
    let turns = state["turns"]
        .as_array_mut()
        .ok_or_else(|| Error::bad("Invalid session turns."))?;
    if turns.len() >= 200 {
        return Err(Error::bad(
            "Session turn limit reached; create a new session.",
        ));
    }
    turns.push(json!({"input":message,"response":d["response"],"decision_id":d["decision_id"],"at":chrono::Utc::now().to_rfc3339()}));
    s.store
        .commit(
            &p.tenant,
            &p.subject,
            "session.turn",
            vec![Write {
                kind: "sessions".into(),
                id: sid.into(),
                payload: state,
                expected: Some(r.revision),
                immutable: false,
            }],
            None,
        )
        .await?;
    Ok(Json(
        json!({"response":d["response"],"decision":d,"parse":parse,"session_id":sid,"revision":r.revision+1}),
    ))
}
async fn search(
    State(s): State<App>,
    Extension(p): Extension<Principal>,
    Json(v): Json<Value>,
) -> Result<Json<Value>> {
    p.require("knowledge.search")?;
    let (docs, _) = s
        .authorized_evidence(
            &p,
            v["purpose"].as_str().unwrap_or("service_prequalification"),
        )
        .await?;
    Ok(Json(sami_intelligence::search(
        v["query"].as_str().unwrap_or(""),
        &docs,
        v["limit"].as_u64().unwrap_or(20).clamp(1, 30) as usize,
    )))
}
async fn graph(
    State(s): State<App>,
    Extension(p): Extension<Principal>,
    Json(v): Json<Value>,
) -> Result<Json<Value>> {
    p.require("knowledge.search")?;
    let (_, claims) = s
        .authorized_evidence(
            &p,
            v["purpose"].as_str().unwrap_or("service_prequalification"),
        )
        .await?;
    Ok(Json(
        sami_intelligence::graph_query(&v, &claims, 3).map_err(|e| Error::bad(e.to_string()))?,
    ))
}
async fn research(
    State(s): State<App>,
    Extension(p): Extension<Principal>,
    Path(op): Path<String>,
    Json(v): Json<Value>,
) -> Result<Json<Value>> {
    p.require("research.execute")?;
    let permit = s
        .compute
        .clone()
        .try_acquire_owned()
        .map_err(|_| Error::unavailable("Compute admission budget exhausted."))?;
    let result = tokio::time::timeout(
        Duration::from_secs(5),
        tokio::task::spawn_blocking(move || {
            let _permit = permit;
            sami_research::dispatch(&op, &v)
        }),
    )
    .await
    .map_err(|_| Error::unavailable("Research time budget exceeded."))?
    .map_err(|_| Error::internal())?
    .map_err(|e| Error::bad(e.to_string()))?;
    Ok(Json(result))
}
async fn events(
    State(s): State<App>,
    Extension(p): Extension<Principal>,
    Query(q): Query<Page>,
    headers: HeaderMap,
) -> Result<Sse<impl futures_core::Stream<Item = std::result::Result<Event, Infallible>>>> {
    p.require("audit.read")?;
    let after = headers
        .get("Last-Event-ID")
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.parse().ok())
        .unwrap_or(q.after);
    let items = s.store.events(&p.tenant, after, 1000).await?;
    let stream = futures_util::stream::iter(items.into_iter().map(|v| {
        let id = v["envelope"]["epoch"].to_string();
        Ok(Event::default().id(id).event("audit").data(v.to_string()))
    }));
    Ok(Sse::new(stream).keep_alive(KeepAlive::default()))
}
async fn compatibility(
    State(s): State<App>,
    Extension(p): Extension<Principal>,
    Json(v): Json<Value>,
) -> Result<Response> {
    for k in v
        .as_object()
        .ok_or_else(|| Error::bad("Object required."))?
        .keys()
    {
        if !["model", "messages", "stream", "max_tokens", "metadata"].contains(&k.as_str()) {
            return Err(Error::bad(format!(
                "Unsupported compatibility field {k}; use native task contracts."
            )));
        }
    }
    let msg = v["messages"]
        .as_array()
        .and_then(|a| a.iter().rev().find(|x| x["role"] == "user"))
        .and_then(|x| x["content"].as_str())
        .ok_or_else(|| Error::bad("A user text message is required."))?;
    let d=s.decide(&p,json!({"task_id":"service_triage","input":{"message":msg},"pack_id":"industrial-service"})).await?;
    let text = d["response"]
        .as_str()
        .unwrap_or("Review the structured decision.");
    let text: String = text
        .chars()
        .take(v["max_tokens"].as_u64().unwrap_or(512).min(1024) as usize * 4)
        .collect();
    if v["stream"] == true {
        let chunk = json!({"id":d["decision_id"],"object":"chat.completion.chunk","choices":[{"index":0,"delta":{"content":text},"finish_reason":null}],"sami":{"profile":"native_strict","status":d["status"]}});
        return Ok(Sse::new(futures_util::stream::iter(vec![
            Ok::<_, Infallible>(Event::default().data(chunk.to_string())),
            Ok(Event::default().data("[DONE]")),
        ]))
        .into_response());
    }
    Ok(Json(json!({"id":d["decision_id"],"object":"chat.completion","choices":[{"index":0,"message":{"role":"assistant","content":text},"finish_reason":"stop"}],"sami":d,"usage":null})).into_response())
}
async fn export(State(s): State<App>, Extension(p): Extension<Principal>) -> Result<Json<Value>> {
    p.require("data.export")?;
    let mut data = serde_json::Map::new();
    for k in crate::service::KINDS {
        if ["identities"].contains(k) {
            continue;
        }
        let mut vals = Vec::new();
        let mut offset = 0;
        loop {
            let page = s.store.list(&p.tenant, k, 500, offset).await?;
            let n = page.len();
            for r in page {
                if s.authorize_record(&p, &r.payload, "administration")
                    .await
                    .is_ok()
                {
                    vals.push(r.payload)
                }
            }
            if n < 500 {
                break;
            }
            offset += 500;
            if offset > 100000 {
                return Err(Error::unavailable(
                    "Use incremental export for datasets above 100000 records.",
                ));
            }
        }
        data.insert((*k).into(), json!(vals));
    }
    Ok(Json(
        json!({"tenant":p.tenant,"format_version":1,"records":data,"audit_page":s.store.events(&p.tenant,0,1000).await?,"audit_pagination":"Use /v1/audit?after=<last_epoch> for remaining audit events"}),
    ))
}
async fn bundle_export(
    State(s): State<App>,
    Extension(p): Extension<Principal>,
    Json(v): Json<Value>,
) -> Result<Json<Value>> {
    p.require("bundle.export")?;
    let device = v["device_id"]
        .as_str()
        .ok_or_else(|| Error::bad("device_id required."))?;
    let pack_id = v["pack_id"].as_str().unwrap_or("industrial-service");
    let epoch = s.store.epoch(&p.tenant).await?;
    let pack = s.store.get(&p.tenant, "packs", pack_id).await?;
    s.authorize_record(&p, &pack.payload, "service_prequalification")
        .await?;
    if pack.payload["status"] != "approved" {
        return Err(Error::forbidden("Export an approved pack."));
    }
    if device.is_empty() || device.len() > 255 {
        return Err(Error::bad(
            "Device identity must contain 1..255 characters.",
        ));
    }
    let (sources, claims) = s
        .authorized_evidence(&p, "service_prequalification")
        .await?;
    let payload = json!({"records":{"sources":sources,"claims":claims,"packs":[pack.payload]}});
    if s.store.epoch(&p.tenant).await? != epoch {
        return Err(Error::conflict(
            "Authority changed during export; retry a new snapshot.",
        ));
    }
    let manifest = json!({"format_version":1,"min_runtime":env!("CARGO_PKG_VERSION"),"tenant":p.tenant,"device_id":device,"pack_id":pack_id,"profile":"native_strict","source_epoch":epoch,"created_at":chrono::Utc::now().to_rfc3339(),"expires_at":chrono::Utc::now().timestamp()+v["lease_seconds"].as_i64().unwrap_or(3600).clamp(60,86400),"payload_digest":digest(&canonical(&payload)),"effects_allowed":false,"public_key":s.config.crypto.public_key()});
    let key = s.config.crypto.device_key(&p.tenant, device);
    Ok(Json(
        json!({"manifest":manifest,"signature":s.config.crypto.sign(&manifest),"payload":Crypto::seal_device(&payload,&format!("bundle/{}/{device}",p.tenant),&key)?,"encryption":"AES-256-GCM; separately provision the per-device key and pinned issuer public key"}),
    ))
}

async fn bundle_import(
    State(s): State<App>,
    Extension(p): Extension<Principal>,
    Json(v): Json<Value>,
) -> Result<Json<Value>> {
    p.require("bundle.import")?;
    let m = &v["manifest"];
    let issuer = m["public_key"].as_str().unwrap_or("");
    let trusted = s.config.bundle_trust_keys.iter().any(|k| k == issuer)
        || (!s.config.production
            && s.config.bundle_trust_keys.is_empty()
            && issuer == s.config.crypto.public_key());
    if m["tenant"] != p.tenant
        || m["format_version"] != 1
        || m["min_runtime"] != env!("CARGO_PKG_VERSION")
        || m["effects_allowed"] != false
        || m["expires_at"].as_i64().unwrap_or(0) <= chrono::Utc::now().timestamp()
        || !trusted
        || !verify_public(m, v["signature"].as_str().unwrap_or(""), issuer)
    {
        return Err(Error::forbidden(
            "Bundle issuer, scope, runtime, signature or lease invalid.",
        ));
    }
    let device = m["device_id"]
        .as_str()
        .ok_or_else(|| Error::bad("device missing."))?;
    if s.config.device_id.as_deref() != Some(device) {
        return Err(Error::forbidden(
            "Bundle device must match configured SAMI_DEVICE_ID.",
        ));
    }
    let key = match &s.config.bundle_decryption_key {
        Some(key) => key.clone(),
        None if !s.config.production => s.config.crypto.device_key(&p.tenant, device),
        None => {
            return Err(Error::forbidden(
                "Provision SAMI_BUNDLE_ENCRYPTION_KEY for this device.",
            ))
        }
    };
    let payload = Crypto::open_device(
        v["payload"].as_str().unwrap_or(""),
        &format!("bundle/{}/{device}", p.tenant),
        &key,
    )
    .map_err(|_| Error::forbidden("Bundle ciphertext or device key invalid."))?;
    if digest(&canonical(&payload)) != m["payload_digest"] {
        return Err(Error::bad("Bundle payload checksum differs."));
    }
    let mut writes = Vec::new();
    for k in ["sources", "claims", "packs"] {
        for r in payload["records"][k].as_array().into_iter().flatten() {
            let id = r["id"]
                .as_str()
                .ok_or_else(|| Error::bad("Bundle record lacks id."))?;
            if s.store.get(&p.tenant, "tombstones", id).await.is_ok() {
                continue;
            }
            let mut record = r.clone();
            if k == "claims" {
                record["source_revision"] = json!(1)
            }
            writes.push(Write {
                kind: k.into(),
                id: id.into(),
                payload: record,
                expected: Some(0),
                immutable: false,
            });
        }
    }
    writes.push(Write{kind:"controls".into(),id:"runtime".into(),payload:json!({"effects_enabled":false,"learning_enabled":false,"sources_enabled":true,"bundle_expires_at":m["expires_at"],"device_id":device}),expected:None,immutable:false});
    s.store
        .commit(&p.tenant, &p.subject, "bundle.import", writes, None)
        .await?;
    Ok(Json(json!({"status":"imported","effects_enabled":false})))
}
async fn workflow_start(
    State(s): State<App>,
    Extension(p): Extension<Principal>,
    Path(id): Path<String>,
    Json(v): Json<Value>,
) -> Result<Json<Value>> {
    Ok(Json(s.start_workflow(&p, &id, v).await?))
}
async fn workflow_advance(
    State(s): State<App>,
    Extension(p): Extension<Principal>,
    Path(id): Path<String>,
    Json(v): Json<Value>,
) -> Result<Json<Value>> {
    Ok(Json(s.advance_workflow(&p, &id, v).await?))
}
async fn workflow_cancel(
    State(s): State<App>,
    Extension(p): Extension<Principal>,
    Path(id): Path<String>,
) -> Result<Json<Value>> {
    p.require("workflow.execute")?;
    let r = s.store.get(&p.tenant, "workflow_runs", &id).await?;
    let mut v = r.payload;
    if ["completed", "cancelled", "failed"].contains(&v["status"].as_str().unwrap_or("")) {
        return Err(Error::conflict("Workflow is already terminal."));
    }
    v["status"] = json!("cancelled");
    s.store
        .commit(
            &p.tenant,
            &p.subject,
            "workflow.cancel",
            vec![Write {
                kind: "workflow_runs".into(),
                id,
                payload: v.clone(),
                expected: Some(r.revision),
                immutable: false,
            }],
            None,
        )
        .await?;
    Ok(Json(v))
}
async fn roleplay(
    State(s): State<App>,
    Extension(p): Extension<Principal>,
    Json(v): Json<Value>,
) -> Result<Json<Value>> {
    p.require("simulation.invoke")?;
    let sid = v["session_id"]
        .as_str()
        .ok_or_else(|| Error::bad("session_id required"))?;
    let r = s.store.get(&p.tenant, "sessions", sid).await?;
    s.authorize_record(&p, &r.payload, "service_prequalification")
        .await?;
    let result = sami_intelligence::roleplay(
        v["message"].as_str().unwrap_or(""),
        &r.payload["state"],
        &v["persona"],
    );
    let mut session = r.payload;
    session["state"] = result
        .get("state")
        .cloned()
        .unwrap_or_else(|| result.clone());
    s.store
        .commit(
            &p.tenant,
            &p.subject,
            "simulation.turn",
            vec![Write {
                kind: "sessions".into(),
                id: sid.into(),
                payload: session,
                expected: Some(r.revision),
                immutable: false,
            }],
            None,
        )
        .await?;
    Ok(Json(result))
}
async fn policy_diff(
    State(_s): State<App>,
    Extension(p): Extension<Principal>,
    Json(v): Json<Value>,
) -> Result<Json<Value>> {
    p.require("evaluation.execute")?;
    let task = v["task_id"].as_str().unwrap_or("service_triage");
    let claims: Vec<Value> =
        serde_json::from_value(v.get("claims").cloned().unwrap_or_else(|| json!([])))?;
    let before = sami_intelligence::decide(task, &v["input"], &claims, &v["before_pack"])
        .map_err(|e| Error::bad(e.to_string()))?;
    let after = sami_intelligence::decide(task, &v["input"], &claims, &v["after_pack"])
        .map_err(|e| Error::bad(e.to_string()))?;
    Ok(Json(
        json!({"simulation_only":true,"external_effects":false,"changed":before!=after,"before":before,"after":after}),
    ))
}
