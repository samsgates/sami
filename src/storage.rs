use crate::{
    crypto::{canonical, digest, Crypto},
    error::{Error, Result},
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::{any::AnyPoolOptions, AnyPool, Row};

#[derive(Clone)]
pub struct Store {
    pub pool: AnyPool,
    pub crypto: Crypto,
    pub postgres: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Record {
    pub id: String,
    pub revision: i64,
    pub epoch: i64,
    pub payload: Value,
}
#[derive(Clone)]
pub struct Write {
    pub kind: String,
    pub id: String,
    pub payload: Value,
    pub expected: Option<i64>,
    pub immutable: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Credential {
    pub id: String,
    pub tenant: String,
    pub principal: String,
    pub scopes: Vec<String>,
    pub groups: Vec<String>,
    pub expires_at: i64,
    pub revoked: bool,
}
impl Store {
    pub async fn connect(url: &str, crypto: Crypto) -> Result<Self> {
        sqlx::any::install_default_drivers();
        let pool = AnyPoolOptions::new()
            .max_connections(if url.starts_with("sqlite:") { 1 } else { 12 })
            .connect(url)
            .await?;
        let s = Self {
            pool,
            crypto,
            postgres: url.starts_with("postgres"),
        };
        s.migrate().await?;
        Ok(s)
    }
    pub fn sql(&self, q: &str) -> String {
        if !self.postgres {
            return q.into();
        }
        let mut n = 0;
        q.chars()
            .map(|c| {
                if c == '?' {
                    n += 1;
                    format!("${n}")
                } else {
                    c.to_string()
                }
            })
            .collect()
    }
    async fn migrate(&self) -> Result<()> {
        let statements=[
            "CREATE TABLE IF NOT EXISTS sami_epochs (tenant TEXT PRIMARY KEY, epoch BIGINT NOT NULL, last_hash TEXT NOT NULL)",
            "CREATE TABLE IF NOT EXISTS sami_records (tenant TEXT NOT NULL, kind TEXT NOT NULL, id TEXT NOT NULL, revision BIGINT NOT NULL, epoch BIGINT NOT NULL, payload TEXT NOT NULL, deleted BIGINT NOT NULL DEFAULT 0, PRIMARY KEY(tenant,kind,id))",
            "CREATE TABLE IF NOT EXISTS sami_versions (tenant TEXT NOT NULL, kind TEXT NOT NULL, id TEXT NOT NULL, revision BIGINT NOT NULL, epoch BIGINT NOT NULL, recorded_at TEXT NOT NULL, payload TEXT NOT NULL, PRIMARY KEY(tenant,kind,id,revision))",
            "CREATE TABLE IF NOT EXISTS sami_events (tenant TEXT NOT NULL, epoch BIGINT NOT NULL, envelope TEXT NOT NULL, PRIMARY KEY(tenant,epoch))",
            "CREATE TABLE IF NOT EXISTS sami_credentials (token_hash TEXT PRIMARY KEY, id TEXT UNIQUE NOT NULL, tenant TEXT NOT NULL, principal TEXT NOT NULL, scopes TEXT NOT NULL, groups_json TEXT NOT NULL, expires_at BIGINT NOT NULL, revoked BIGINT NOT NULL DEFAULT 0)",
            "CREATE INDEX IF NOT EXISTS sami_records_scope ON sami_records(tenant,kind,deleted)",
            "CREATE INDEX IF NOT EXISTS sami_versions_scope ON sami_versions(tenant,epoch)",
        ];
        for q in statements {
            sqlx::query(q).execute(&self.pool).await?;
        }
        if !self.postgres {
            sqlx::query("PRAGMA journal_mode=WAL")
                .execute(&self.pool)
                .await?;
            sqlx::query("PRAGMA busy_timeout=5000")
                .execute(&self.pool)
                .await?;
            sqlx::query("PRAGMA secure_delete=ON")
                .execute(&self.pool)
                .await?;
        }
        Ok(())
    }
    pub async fn ensure_tenant(&self, tenant: &str) -> Result<()> {
        sqlx::query(&self.sql("INSERT INTO sami_epochs(tenant,epoch,last_hash) VALUES(?,0,'') ON CONFLICT(tenant) DO NOTHING")).bind(tenant).execute(&self.pool).await?;
        Ok(())
    }
    pub async fn epoch(&self, tenant: &str) -> Result<i64> {
        let r = sqlx::query(&self.sql("SELECT epoch FROM sami_epochs WHERE tenant=?"))
            .bind(tenant)
            .fetch_optional(&self.pool)
            .await?;
        Ok(r.map(|r| r.get::<i64, _>(0)).unwrap_or(0))
    }
    pub async fn get(&self, tenant: &str, kind: &str, id: &str) -> Result<Record> {
        let r=sqlx::query(&self.sql("SELECT revision,epoch,payload FROM sami_records WHERE tenant=? AND kind=? AND id=? AND deleted=0")).bind(tenant).bind(kind).bind(id).fetch_optional(&self.pool).await?.ok_or_else(Error::missing)?;
        self.decode(tenant, kind, id, r.get(0), r.get(1), r.get(2))
    }
    fn decode(
        &self,
        t: &str,
        k: &str,
        id: &str,
        rev: i64,
        epoch: i64,
        cipher: String,
    ) -> Result<Record> {
        let mut p = self.crypto.open(&cipher, &format!("{t}/{k}/{id}/{rev}"))?;
        if let Some(o) = p.as_object_mut() {
            o.insert("id".into(), json!(id));
            o.insert("revision".into(), json!(rev));
        }
        Ok(Record {
            id: id.into(),
            revision: rev,
            epoch,
            payload: p,
        })
    }
    pub async fn list(&self, t: &str, k: &str, limit: i64, offset: i64) -> Result<Vec<Record>> {
        let rows=sqlx::query(&self.sql("SELECT id,revision,epoch,payload FROM sami_records WHERE tenant=? AND kind=? AND deleted=0 ORDER BY id LIMIT ? OFFSET ?")).bind(t).bind(k).bind(limit.clamp(1,10000)).bind(offset.max(0)).fetch_all(&self.pool).await?;
        rows.into_iter()
            .map(|r| {
                self.decode(
                    t,
                    k,
                    r.get::<String, _>(0).as_str(),
                    r.get(1),
                    r.get(2),
                    r.get(3),
                )
            })
            .collect()
    }
    pub async fn count(&self, t: &str, k: &str) -> Result<i64> {
        let row = sqlx::query(
            &self.sql("SELECT COUNT(*) FROM sami_records WHERE tenant=? AND kind=? AND deleted=0"),
        )
        .bind(t)
        .bind(k)
        .fetch_one(&self.pool)
        .await?;
        Ok(row.get(0))
    }
    pub async fn history(&self, t: &str, k: &str, id: &str) -> Result<Vec<Value>> {
        let rows=sqlx::query(&self.sql("SELECT revision,epoch,recorded_at,payload FROM sami_versions WHERE tenant=? AND kind=? AND id=? ORDER BY revision")).bind(t).bind(k).bind(id).fetch_all(&self.pool).await?;
        rows.into_iter().map(|r|{let rev:i64=r.get(0);let value=self.crypto.open(&r.get::<String,_>(3),&format!("{t}/{k}/{id}/{rev}"))?;Ok(json!({"revision":rev,"epoch":r.get::<i64,_>(1),"recorded_at":r.get::<String,_>(2),"payload":value}))}).collect()
    }
    /// The tenant epoch row is the serialization point for publication and dispatch.
    pub async fn commit(
        &self,
        t: &str,
        actor: &str,
        operation: &str,
        writes: Vec<Write>,
        expected_epoch: Option<i64>,
    ) -> Result<i64> {
        self.ensure_tenant(t).await?;
        let mut tx = self.pool.begin().await?;
        sqlx::query(&self.sql("UPDATE sami_epochs SET epoch=epoch WHERE tenant=?"))
            .bind(t)
            .execute(&mut *tx)
            .await?;
        let r = sqlx::query(&self.sql("SELECT epoch,last_hash FROM sami_epochs WHERE tenant=?"))
            .bind(t)
            .fetch_one(&mut *tx)
            .await?;
        let old: i64 = r.get(0);
        let previous: String = r.get(1);
        if expected_epoch.is_some_and(|e| e != old) {
            return Err(Error::conflict(
                "The evidence snapshot changed; evaluate again.",
            ));
        }
        let epoch = old + 1;
        let mut refs = Vec::new();
        for w in writes {
            let oldrev = sqlx::query(
                &self.sql("SELECT revision FROM sami_records WHERE tenant=? AND kind=? AND id=?"),
            )
            .bind(t)
            .bind(&w.kind)
            .bind(&w.id)
            .fetch_optional(&mut *tx)
            .await?
            .map(|r| r.get::<i64, _>(0))
            .unwrap_or(0);
            if w.immutable && oldrev != 0 {
                return Err(Error::conflict("An immutable artifact already exists."));
            }
            if w.expected.is_some_and(|x| x != oldrev) {
                return Err(Error::conflict("The resource revision changed."));
            }
            let rev = oldrev + 1;
            let mut value = w.payload;
            if let Some(o) = value.as_object_mut() {
                o.insert("id".into(), json!(w.id));
                o.insert("revision".into(), json!(rev));
            }
            let ciphertext = self
                .crypto
                .seal(&value, &format!("{t}/{}/{}/{rev}", w.kind, w.id))?;
            sqlx::query(&self.sql("INSERT INTO sami_records(tenant,kind,id,revision,epoch,payload,deleted) VALUES(?,?,?,?,?,?,0) ON CONFLICT(tenant,kind,id) DO UPDATE SET revision=excluded.revision,epoch=excluded.epoch,payload=excluded.payload,deleted=0")).bind(t).bind(&w.kind).bind(&w.id).bind(rev).bind(epoch).bind(&ciphertext).execute(&mut *tx).await?;
            sqlx::query(&self.sql("INSERT INTO sami_versions(tenant,kind,id,revision,epoch,recorded_at,payload) VALUES(?,?,?,?,?,?,?)")).bind(t).bind(&w.kind).bind(&w.id).bind(rev).bind(epoch).bind(chrono::Utc::now().to_rfc3339()).bind(ciphertext).execute(&mut *tx).await?;
            refs.push(json!({"kind":w.kind,"id":w.id,"revision":rev}));
        }
        let envelope = json!({"tenant_ref":digest(t.as_bytes()),"actor_ref":digest(actor.as_bytes()),"epoch":epoch,"operation":operation,"refs":refs,"previous":previous,"at":chrono::Utc::now().to_rfc3339()});
        let hash = digest(&canonical(&envelope));
        let event =
            json!({"envelope":envelope,"hash":hash,"signature":self.crypto.sign(&envelope)});
        sqlx::query(&self.sql("INSERT INTO sami_events(tenant,epoch,envelope) VALUES(?,?,?)"))
            .bind(t)
            .bind(epoch)
            .bind(serde_json::to_string(&event)?)
            .execute(&mut *tx)
            .await?;
        sqlx::query(&self.sql("UPDATE sami_epochs SET epoch=?,last_hash=? WHERE tenant=?"))
            .bind(epoch)
            .bind(hash)
            .bind(t)
            .execute(&mut *tx)
            .await?;
        tx.commit().await?;
        Ok(epoch)
    }
    pub async fn events(&self, t: &str, after: i64, limit: i64) -> Result<Vec<Value>> {
        let rows = sqlx::query(&self.sql(
            "SELECT envelope FROM sami_events WHERE tenant=? AND epoch>? ORDER BY epoch LIMIT ?",
        ))
        .bind(t)
        .bind(after)
        .bind(limit.clamp(1, 1000))
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter()
            .map(|r| Ok(serde_json::from_str(&r.get::<String, _>(0))?))
            .collect()
    }
    pub async fn purge(&self, t: &str, kind: &str, id: &str) -> Result<()> {
        let mut tx = self.pool.begin().await?;
        sqlx::query(&self.sql("UPDATE sami_epochs SET epoch=epoch WHERE tenant=?"))
            .bind(t)
            .execute(&mut *tx)
            .await?;
        sqlx::query(&self.sql(
            "UPDATE sami_records SET payload='',deleted=1 WHERE tenant=? AND kind=? AND id=?",
        ))
        .bind(t)
        .bind(kind)
        .bind(id)
        .execute(&mut *tx)
        .await?;
        sqlx::query(&self.sql("DELETE FROM sami_versions WHERE tenant=? AND kind=? AND id=?"))
            .bind(t)
            .bind(kind)
            .bind(id)
            .execute(&mut *tx)
            .await?;
        tx.commit().await?;
        Ok(())
    }
    pub async fn add_credential(&self, hash: &str, c: &Credential) -> Result<()> {
        self.ensure_tenant(&c.tenant).await?;
        sqlx::query(&self.sql("INSERT INTO sami_credentials(token_hash,id,tenant,principal,scopes,groups_json,expires_at,revoked) VALUES(?,?,?,?,?,?,?,0) ON CONFLICT(token_hash) DO NOTHING")).bind(hash).bind(&c.id).bind(&c.tenant).bind(&c.principal).bind(serde_json::to_string(&c.scopes)?).bind(serde_json::to_string(&c.groups)?).bind(c.expires_at).execute(&self.pool).await?;
        Ok(())
    }
    pub async fn credential(&self, hash: &str) -> Result<Credential> {
        let r=sqlx::query(&self.sql("SELECT id,tenant,principal,scopes,groups_json,expires_at,revoked FROM sami_credentials WHERE token_hash=?")).bind(hash).fetch_optional(&self.pool).await?.ok_or_else(||Error::forbidden("Invalid or revoked credential."))?;
        Self::decode_credential(r)
    }
    fn decode_credential(r: sqlx::any::AnyRow) -> Result<Credential> {
        Ok(Credential {
            id: r.get(0),
            tenant: r.get(1),
            principal: r.get(2),
            scopes: serde_json::from_str(&r.get::<String, _>(3))?,
            groups: serde_json::from_str(&r.get::<String, _>(4))?,
            expires_at: r.get(5),
            revoked: r.get::<i64, _>(6) != 0,
        })
    }
    pub async fn credentials(&self, t: &str) -> Result<Vec<Credential>> {
        let rows=sqlx::query(&self.sql("SELECT id,tenant,principal,scopes,groups_json,expires_at,revoked FROM sami_credentials WHERE tenant=? ORDER BY id")).bind(t).fetch_all(&self.pool).await?;
        rows.into_iter().map(Self::decode_credential).collect()
    }
    pub async fn revoke(&self, t: &str, id: &str) -> Result<()> {
        self.ensure_tenant(t).await?;
        let mut tx = self.pool.begin().await?;
        sqlx::query(&self.sql("UPDATE sami_epochs SET epoch=epoch WHERE tenant=?"))
            .bind(t)
            .execute(&mut *tx)
            .await?;
        let row = sqlx::query(&self.sql("SELECT epoch,last_hash FROM sami_epochs WHERE tenant=?"))
            .bind(t)
            .fetch_one(&mut *tx)
            .await?;
        let changed =
            sqlx::query(&self.sql("UPDATE sami_credentials SET revoked=1 WHERE tenant=? AND id=?"))
                .bind(t)
                .bind(id)
                .execute(&mut *tx)
                .await?;
        if changed.rows_affected() == 0 {
            return Err(Error::missing());
        }
        let epoch = row.get::<i64, _>(0) + 1;
        let envelope = json!({"tenant_ref":digest(t.as_bytes()),"epoch":epoch,"operation":"credential.revoked","credential_ref":digest(id.as_bytes()),"previous":row.get::<String,_>(1),"at":chrono::Utc::now().to_rfc3339()});
        let hash = digest(&canonical(&envelope));
        let event =
            json!({"envelope":envelope,"hash":hash,"signature":self.crypto.sign(&envelope)});
        sqlx::query(&self.sql("INSERT INTO sami_events(tenant,epoch,envelope) VALUES(?,?,?)"))
            .bind(t)
            .bind(epoch)
            .bind(serde_json::to_string(&event)?)
            .execute(&mut *tx)
            .await?;
        sqlx::query(&self.sql("UPDATE sami_epochs SET epoch=?,last_hash=? WHERE tenant=?"))
            .bind(epoch)
            .bind(hash)
            .bind(t)
            .execute(&mut *tx)
            .await?;
        tx.commit().await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn isolation_cas_and_atomic_epoch() {
        let c = Crypto::new(&"11".repeat(32), &"22".repeat(32)).unwrap();
        let s = Store::connect("sqlite::memory:", c).await.unwrap();
        let w = Write {
            kind: "claim".into(),
            id: "a".into(),
            payload: json!({"secret":"one"}),
            expected: Some(0),
            immutable: false,
        };
        assert_eq!(
            s.commit("t1", "actor", "create", vec![w.clone()], Some(0))
                .await
                .unwrap(),
            1
        );
        assert!(s.get("t2", "claim", "a").await.is_err());
        assert!(s
            .commit("t1", "actor", "wrong", vec![w], Some(0))
            .await
            .is_err());
        assert_eq!(s.history("t1", "claim", "a").await.unwrap().len(), 1);
        assert_eq!(s.events("t1", 0, 20).await.unwrap().len(), 1);
    }
}
