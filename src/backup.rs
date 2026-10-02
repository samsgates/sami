//! Portable encrypted backup; restore requires an independently retained deletion ledger.
use crate::{
    crypto::{canonical, digest},
    error::{Error, Result},
    storage::Store,
};
use serde_json::{json, Value};
use sqlx::Row;
impl Store {
    pub async fn backup(&self) -> Result<Value> {
        let mut tx = self.pool.begin().await?;
        if self.postgres {
            sqlx::query("SET TRANSACTION ISOLATION LEVEL REPEATABLE READ READ ONLY")
                .execute(&mut *tx)
                .await?;
        }
        let mut data = serde_json::Map::new();
        for (table, columns, numeric) in [
            (
                "sami_epochs",
                vec!["tenant", "epoch", "last_hash"],
                vec!["epoch"],
            ),
            (
                "sami_records",
                vec![
                    "tenant", "kind", "id", "revision", "epoch", "payload", "deleted",
                ],
                vec!["revision", "epoch", "deleted"],
            ),
            (
                "sami_versions",
                vec![
                    "tenant",
                    "kind",
                    "id",
                    "revision",
                    "epoch",
                    "recorded_at",
                    "payload",
                ],
                vec!["revision", "epoch"],
            ),
            (
                "sami_events",
                vec!["tenant", "epoch", "envelope"],
                vec!["epoch"],
            ),
            (
                "sami_credentials",
                vec![
                    "token_hash",
                    "id",
                    "tenant",
                    "principal",
                    "scopes",
                    "groups_json",
                    "expires_at",
                    "revoked",
                ],
                vec!["expires_at", "revoked"],
            ),
        ] {
            let rows = sqlx::query(&format!(
                "SELECT {} FROM {table} LIMIT 100001",
                columns.join(",")
            ))
            .fetch_all(&mut *tx)
            .await?;
            if rows.len() > 100000 {
                return Err(Error::unavailable("Portable backup supports <=100000 rows per table; use database-native snapshot tooling for larger stores."));
            }
            let values: Vec<_> = rows
                .into_iter()
                .map(|r| {
                    let mut obj = serde_json::Map::new();
                    for c in &columns {
                        obj.insert(
                            (*c).into(),
                            if numeric.contains(c) {
                                json!(r.get::<i64, _>(*c))
                            } else {
                                json!(r.get::<String, _>(*c))
                            },
                        );
                    }
                    Value::Object(obj)
                })
                .collect();
            data.insert(table.into(), json!(values));
        }
        tx.commit().await?;
        let payload = json!({"format_version":1,"tables":data});
        let manifest = json!({"format_version":1,"created_at":chrono::Utc::now().to_rfc3339(),"runtime":env!("CARGO_PKG_VERSION"),"payload_digest":digest(&canonical(&payload)),"public_key":self.crypto.public_key(),"scope":"all tenants; operator-only; credentials are included as hashes"});
        Ok(
            json!({"manifest":manifest,"signature":self.crypto.sign(&manifest),"payload":self.crypto.seal(&payload,"sami/backup/v1")?}),
        )
    }
    pub async fn restore(&self, backup: &Value, ledger: &Value) -> Result<Value> {
        if !ledger.is_array() {
            return Err(Error::bad(
                "Provide latest independently retained tombstone ledger as a JSON array.",
            ));
        }
        let manifest = &backup["manifest"];
        if manifest["format_version"] != 1
            || !self
                .crypto
                .verify(manifest, backup["signature"].as_str().unwrap_or(""))
        {
            return Err(Error::forbidden(
                "Backup signature/version invalid or signing trust key differs.",
            ));
        }
        let payload = self
            .crypto
            .open(backup["payload"].as_str().unwrap_or(""), "sami/backup/v1")?;
        if digest(&canonical(&payload)) != manifest["payload_digest"] {
            return Err(Error::bad("Backup checksum mismatch."));
        }
        let mut tx = self.pool.begin().await?;
        for table in [
            "sami_epochs",
            "sami_records",
            "sami_versions",
            "sami_events",
            "sami_credentials",
        ] {
            let count: i64 = sqlx::query(&format!("SELECT COUNT(*) FROM {table}"))
                .fetch_one(&mut *tx)
                .await?
                .get(0);
            if count != 0 {
                return Err(Error::conflict("Restore requires an empty initialized database; use a new database and retain the existing one."));
            }
        }
        for (table, columns, numeric) in [
            (
                "sami_epochs",
                vec!["tenant", "epoch", "last_hash"],
                vec!["epoch"],
            ),
            (
                "sami_records",
                vec![
                    "tenant", "kind", "id", "revision", "epoch", "payload", "deleted",
                ],
                vec!["revision", "epoch", "deleted"],
            ),
            (
                "sami_versions",
                vec![
                    "tenant",
                    "kind",
                    "id",
                    "revision",
                    "epoch",
                    "recorded_at",
                    "payload",
                ],
                vec!["revision", "epoch"],
            ),
            (
                "sami_events",
                vec!["tenant", "epoch", "envelope"],
                vec!["epoch"],
            ),
            (
                "sami_credentials",
                vec![
                    "token_hash",
                    "id",
                    "tenant",
                    "principal",
                    "scopes",
                    "groups_json",
                    "expires_at",
                    "revoked",
                ],
                vec!["expires_at", "revoked"],
            ),
        ] {
            let rows = payload["tables"][table]
                .as_array()
                .ok_or_else(|| Error::bad("Incomplete backup tables."))?;
            if rows.len() > 100000 {
                return Err(Error::bad("Restore row budget exceeded."));
            }
            let query = self.sql(&format!(
                "INSERT INTO {table}({}) VALUES({})",
                columns.join(","),
                vec!["?"; columns.len()].join(",")
            ));
            for row in rows {
                let mut q = sqlx::query(&query);
                for c in &columns {
                    q = if numeric.contains(c) {
                        q.bind(
                            row[*c]
                                .as_i64()
                                .ok_or_else(|| Error::bad("Invalid backup numeric column."))?,
                        )
                    } else {
                        q.bind(
                            row[*c]
                                .as_str()
                                .ok_or_else(|| Error::bad("Invalid backup text column."))?,
                        )
                    };
                }
                q.execute(&mut *tx).await?;
            }
        }
        // Replay external deletion ledger inside the restore transaction before readiness.
        for item in ledger.as_array().ok_or_else(Error::internal)? {
            let tenant = item["tenant"]
                .as_str()
                .ok_or_else(|| Error::bad("Ledger tenant required."))?;
            let source = item["source_id"]
                .as_str()
                .ok_or_else(|| Error::bad("Ledger source_id required."))?;
            let rows = sqlx::query(&self.sql(
                "SELECT kind,id,revision,payload FROM sami_records WHERE tenant=? AND deleted=0",
            ))
            .bind(tenant)
            .fetch_all(&mut *tx)
            .await?;
            for row in rows {
                let kind: String = row.get(0);
                let id: String = row.get(1);
                let revision: i64 = row.get(2);
                let v = self.crypto.open(
                    &row.get::<String, _>(3),
                    &format!("{tenant}/{kind}/{id}/{revision}"),
                )?;
                let derivative = [
                    "receipt_payloads",
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
                ]
                .contains(&kind.as_str());
                if kind == "sources" && id == source
                    || kind == "claims"
                        && (v["source_id"] == source
                            || v["depends_on"].as_array().is_some_and(|a| !a.is_empty()))
                    || derivative
                {
                    sqlx::query(&self.sql("UPDATE sami_records SET payload='',deleted=1 WHERE tenant=? AND kind=? AND id=?")).bind(tenant).bind(&kind).bind(&id).execute(&mut *tx).await?;
                    sqlx::query(
                        &self.sql("DELETE FROM sami_versions WHERE tenant=? AND kind=? AND id=?"),
                    )
                    .bind(tenant)
                    .bind(&kind)
                    .bind(&id)
                    .execute(&mut *tx)
                    .await?;
                }
            }
            // Preserve a resurrection fence even if the deleted object was absent from this backup.
            let oldrev = sqlx::query(&self.sql(
                "SELECT revision FROM sami_records WHERE tenant=? AND kind='tombstones' AND id=?",
            ))
            .bind(tenant)
            .bind(source)
            .fetch_optional(&mut *tx)
            .await?
            .map_or(0, |r| r.get::<i64, _>(0));
            let rev = oldrev + 1;
            let encrypted = self.crypto.seal(
                &json!({"source_id":source,"restored_ledger":true}),
                &format!("{tenant}/tombstones/{source}/{rev}"),
            )?;
            sqlx::query(&self.sql("INSERT INTO sami_records(tenant,kind,id,revision,epoch,payload,deleted) VALUES(?,'tombstones',?,?,0,?,0) ON CONFLICT(tenant,kind,id) DO UPDATE SET revision=excluded.revision,payload=excluded.payload,deleted=0")).bind(tenant).bind(source).bind(rev).bind(encrypted).execute(&mut *tx).await?;
        }
        tx.commit().await?;
        Ok(
            json!({"status":"restored","tombstones_applied":ledger.as_array().map_or(0,Vec::len),"requires_doctor_and_deployment_validation":true}),
        )
    }
}
