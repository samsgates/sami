use crate::{
    config::Config,
    crypto::digest,
    error::{Error, Result},
    storage::{Credential, Store},
};
use chrono::{DateTime, Utc};
use jsonwebtoken::{jwk::JwkSet, Algorithm, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::{Mutex, RwLock};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Principal {
    pub tenant: String,
    pub subject: String,
    pub scopes: Vec<String>,
    pub groups: Vec<String>,
    pub credential_id: String,
    #[serde(default)]
    pub expires_at: i64,
}
impl Principal {
    pub fn allows(&self, scope: &str) -> bool {
        self.scopes.iter().any(|s| {
            s == "*"
                || s == scope
                || s.strip_suffix(".*")
                    .is_some_and(|p| scope.starts_with(&format!("{p}.")))
        })
    }
    pub fn require(&self, scope: &str) -> Result<()> {
        if self.allows(scope) {
            Ok(())
        } else {
            Err(Error::forbidden(
                "The credential does not permit this operation.",
            ))
        }
    }
    pub fn can_read(&self, value: &Value, purpose: &str) -> bool {
        if self.allows("*") {
            return true;
        }
        let acl = value.get("acl").and_then(Value::as_array);
        let acl_ok = acl.is_none_or(|a| {
            a.is_empty()
                || a.iter().filter_map(Value::as_str).any(|g| {
                    self.groups.iter().any(|x| x == g) || self.allows(&format!("source.{g}"))
                })
        });
        let purposes = value.get("purpose").and_then(Value::as_array);
        acl_ok
            && purposes
                .is_none_or(|a| a.is_empty() || a.iter().any(|p| p.as_str() == Some(purpose)))
    }
}
#[derive(Clone)]
pub struct Auth {
    store: Store,
    config: Config,
    client: reqwest::Client,
    keys: Arc<RwLock<Option<(Instant, JwkSet)>>>,
    rate: Arc<Mutex<HashMap<String, (Instant, u32)>>>,
}
impl Auth {
    pub fn new(store: Store, config: Config) -> Result<Self> {
        Ok(Self {
            store,
            config,
            client: reqwest::Client::builder()
                .timeout(Duration::from_secs(5))
                .redirect(reqwest::redirect::Policy::none())
                .build()
                .map_err(|_| Error::internal())?,
            keys: Arc::new(RwLock::new(None)),
            rate: Arc::new(Mutex::new(HashMap::new())),
        })
    }
    pub async fn authenticate(&self, bearer: &str) -> Result<Principal> {
        if bearer.len() > 8192 {
            return Err(Error::forbidden("Invalid credential."));
        }
        let p = if bearer.matches('.').count() == 2 && self.config.oidc_issuer.is_some() {
            self.oidc(bearer).await?
        } else {
            let c = self.store.credential(&digest(bearer.as_bytes())).await?;
            if c.revoked || (c.expires_at != 0 && c.expires_at <= Utc::now().timestamp()) {
                return Err(Error::forbidden("Credential expired or revoked."));
            }
            Principal {
                tenant: c.tenant,
                subject: c.principal,
                scopes: c.scopes,
                groups: c.groups,
                credential_id: c.id,
                expires_at: c.expires_at,
            }
        };
        let mut rates = self.rate.lock().await;
        if rates.len() > 10000 {
            rates.retain(|_, (t, _)| t.elapsed() < Duration::from_secs(60));
        }
        let e = rates
            .entry(format!("{}/{}", p.tenant, p.credential_id))
            .or_insert((Instant::now(), 0));
        if e.0.elapsed() > Duration::from_secs(60) {
            *e = (Instant::now(), 0)
        }
        e.1 += 1;
        if e.1 > 600 {
            return Err(Error {
                status: axum::http::StatusCode::TOO_MANY_REQUESTS,
                code: "RATE_LIMITED",
                message: "Credential request budget exceeded; retry after the minute window."
                    .into(),
            });
        }
        Ok(p)
    }
    pub async fn still_authorized(&self, p: &Principal) -> Result<()> {
        if p.expires_at != 0 && p.expires_at <= Utc::now().timestamp() {
            return Err(Error::forbidden("Credential expired."));
        }
        if p.credential_id.starts_with("oidc:") {
            let m = self
                .store
                .get(&p.tenant, "identities", &p.credential_id[5..])
                .await?;
            if m.payload
                .get("disabled")
                .and_then(Value::as_bool)
                .unwrap_or(false)
                || m.payload["tenant"] != p.tenant
                || m.payload["scopes"] != serde_json::json!(p.scopes)
                || m.payload["groups"] != serde_json::json!(p.groups)
            {
                return Err(Error::forbidden("Identity authority changed."));
            }
            return Ok(());
        }
        let c = self
            .store
            .credentials(&p.tenant)
            .await?
            .into_iter()
            .find(|c| c.id == p.credential_id)
            .ok_or_else(Error::missing)?;
        if c.revoked || (c.expires_at != 0 && c.expires_at <= Utc::now().timestamp()) {
            return Err(Error::forbidden(
                "Credential authority is no longer current.",
            ));
        }
        Ok(())
    }
    async fn oidc(&self, token: &str) -> Result<Principal> {
        let hdr = jsonwebtoken::decode_header(token)
            .map_err(|_| Error::forbidden("Invalid OIDC token."))?;
        if !matches!(hdr.alg, Algorithm::RS256 | Algorithm::ES256) {
            return Err(Error::forbidden("OIDC algorithm is unsupported."));
        }
        let kid = hdr
            .kid
            .ok_or_else(|| Error::forbidden("OIDC key ID is required."))?;
        let needs = self
            .keys
            .read()
            .await
            .as_ref()
            .is_none_or(|(at, _)| at.elapsed() > Duration::from_secs(300));
        if needs {
            let endpoint = self
                .config
                .oidc_jwks
                .as_deref()
                .ok_or_else(|| Error::unavailable("OIDC JWKS endpoint is not configured."))?;
            if !endpoint.starts_with("https://") {
                return Err(Error::bad("OIDC JWKS must use HTTPS."));
            }
            let keys = self
                .client
                .get(endpoint)
                .send()
                .await
                .map_err(|_| Error::unavailable("OIDC key refresh failed."))?
                .error_for_status()
                .map_err(|_| Error::unavailable("OIDC key refresh failed."))?
                .json::<JwkSet>()
                .await
                .map_err(|_| Error::unavailable("Invalid OIDC key set."))?;
            *self.keys.write().await = Some((Instant::now(), keys));
        }
        let guard = self.keys.read().await;
        let keys = &guard.as_ref().ok_or_else(Error::internal)?.1;
        let jwk = keys
            .find(&kid)
            .ok_or_else(|| Error::forbidden("OIDC key ID is unknown; retry after key refresh."))?;
        let key = DecodingKey::from_jwk(jwk)
            .map_err(|_| Error::forbidden("Invalid OIDC signing key."))?;
        let mut v = Validation::new(hdr.alg);
        v.set_issuer(&[self
            .config
            .oidc_issuer
            .as_deref()
            .ok_or_else(Error::internal)?]);
        v.set_audience(&[self
            .config
            .oidc_audience
            .as_deref()
            .ok_or_else(|| Error::bad("OIDC audience is required."))?]);
        v.leeway = 30;
        let decoded = jsonwebtoken::decode::<Value>(token, &key, &v).map_err(|_| {
            Error::forbidden("OIDC signature, issuer, audience or expiry is invalid.")
        })?;
        let sub = decoded.claims["sub"]
            .as_str()
            .ok_or_else(|| Error::forbidden("OIDC subject missing."))?;
        // Tenant and scopes come from a provisioned mapping, never token body tenant claims.
        let mapping_id = digest(
            format!(
                "{}|{sub}",
                self.config.oidc_issuer.as_deref().unwrap_or_default()
            )
            .as_bytes(),
        );
        let map = self
            .store
            .get("_identity_registry", "identities", &mapping_id)
            .await
            .map_err(|_| Error::forbidden("OIDC identity has not been provisioned."))?;
        let tenant = map.payload["tenant"]
            .as_str()
            .ok_or_else(Error::internal)?
            .to_string();
        let map = self
            .store
            .get(&tenant, "identities", &mapping_id)
            .await
            .map_err(|_| Error::forbidden("Identity publication incomplete."))?;
        if map.payload["disabled"].as_bool().unwrap_or(false) {
            return Err(Error::forbidden("OIDC identity disabled."));
        }
        let scopes = serde_json::from_value(map.payload["scopes"].clone())?;
        let groups = serde_json::from_value(
            map.payload
                .get("groups")
                .cloned()
                .unwrap_or_else(|| serde_json::json!([])),
        )?;
        Ok(Principal {
            tenant,
            subject: sub.into(),
            scopes,
            groups,
            credential_id: format!("oidc:{mapping_id}"),
            expires_at: decoded.claims["exp"].as_i64().unwrap_or(0),
        })
    }
}
pub fn timestamp(v: Option<&Value>) -> Option<i64> {
    v.and_then(|v| {
        v.as_i64().or_else(|| {
            v.as_str()
                .and_then(|s| DateTime::parse_from_rfc3339(s).ok().map(|d| d.timestamp()))
        })
    })
}
pub async fn bootstrap(store: &Store, config: &Config) -> Result<()> {
    if let Some(key) = &config.bootstrap_key {
        store
            .add_credential(
                &digest(key.as_bytes()),
                &Credential {
                    id: format!("bootstrap_{}", &digest(key.as_bytes())[..16]),
                    tenant: "default".into(),
                    principal: "bootstrap-owner".into(),
                    scopes: vec!["*".into()],
                    groups: vec![],
                    expires_at: 0,
                    revoked: false,
                },
            )
            .await?;
    }
    Ok(())
}
