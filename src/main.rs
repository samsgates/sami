use clap::{Parser, Subcommand};
use sami::{
    config::Config,
    crypto::{digest, token},
    error::{Error, Result},
    service::Service,
    storage::Credential,
};
use serde_json::{json, Value};
use std::{path::PathBuf, sync::Arc};
#[derive(Parser)]
#[command(
    version,
    about = "Correctable memory and controlled operational intelligence"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}
#[derive(Subcommand)]
enum Command {
    Serve {
        #[arg(long, default_value = "127.0.0.1")]
        host: String,
        #[arg(long, default_value_t = 8080)]
        port: u16,
    },
    Init {
        #[arg(long)]
        seed_demo: bool,
        #[arg(long, default_value = "default")]
        tenant: String,
    },
    Worker {
        #[arg(long, env = "SAMI_WORKER_KEY")]
        api_key: String,
        #[arg(long, default_value_t = 10)]
        interval_seconds: u64,
        #[arg(long)]
        once: bool,
    },
    Doctor,
    /// Output a per-device bundle decryption secret. Protect stdout as secret material.
    DeriveDeviceKey {
        #[arg(long)]
        tenant: String,
        #[arg(long)]
        device_id: String,
    },
    Evaluate {
        #[arg(long)]
        input: PathBuf,
        #[arg(long, default_value = "evaluation.metrics")]
        operation: String,
    },
    Ingest {
        #[arg(long)]
        input: PathBuf,
    },
    Replay {
        #[arg(long)]
        receipt_id: String,
        #[arg(long, default_value = "http://127.0.0.1:8080")]
        api_url: String,
        #[arg(long, env = "SAMI_API_KEY")]
        api_key: String,
    },
    ProvisionKey {
        #[arg(long, default_value = "default")]
        tenant: String,
        #[arg(long)]
        principal: String,
        #[arg(long, value_delimiter = ',')]
        scopes: Vec<String>,
        #[arg(long, value_delimiter = ',')]
        groups: Vec<String>,
        #[arg(long, default_value_t = 0)]
        expires_at: i64,
    },
    #[command(name = "validate", alias = "validate-pack")]
    ValidatePack {
        #[arg(long)]
        input: PathBuf,
    },
    Publish {
        #[arg(long)]
        id: String,
        #[arg(long, default_value = "sources")]
        kind: String,
        #[arg(long)]
        reason: String,
        #[arg(long)]
        claims: Option<PathBuf>,
        #[arg(long, default_value = "http://127.0.0.1:8080")]
        api_url: String,
        #[arg(long, env = "SAMI_API_KEY")]
        api_key: String,
    },
    Export {
        #[arg(long)]
        output: PathBuf,
        #[arg(long, default_value = "http://127.0.0.1:8080")]
        api_url: String,
        #[arg(long, env = "SAMI_API_KEY")]
        api_key: String,
    },
    Backup {
        #[arg(long)]
        output: PathBuf,
        #[arg(long)]
        tombstones_output: PathBuf,
    },
    Restore {
        #[arg(long)]
        input: PathBuf,
        #[arg(long)]
        tombstones_file: PathBuf,
    },
    Request {
        #[arg(long)]
        path: String,
        #[arg(long, default_value = "GET")]
        method: String,
        #[arg(long)]
        input: Option<PathBuf>,
        #[arg(long, default_value = "http://127.0.0.1:8080")]
        api_url: String,
        #[arg(long, env = "SAMI_API_KEY")]
        api_key: String,
    },
}
fn read(path: &PathBuf) -> Result<Value> {
    let bytes = std::fs::read(path)?;
    if bytes.len() > 4 * 1024 * 1024 {
        return Err(Error::bad("Input exceeds 4 MiB."));
    }
    Ok(serde_json::from_slice(&bytes)?)
}
fn print(value: &Value) -> Result<()> {
    println!("{}", serde_json::to_string_pretty(value)?);
    Ok(())
}
fn save_private(path: &PathBuf, value: &Value) -> Result<()> {
    use std::io::Write;
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options.open(path)?;
    file.write_all(&serde_json::to_vec_pretty(value)?)?;
    file.sync_all()?;
    Ok(())
}
async fn request(
    base: &str,
    path: &str,
    method: &str,
    key: &str,
    input: Option<Value>,
) -> Result<Value> {
    if !path.starts_with('/') || path.starts_with("//") {
        return Err(Error::bad("API path must start with one slash."));
    }
    let url = url::Url::parse(base).map_err(|_| Error::bad("Invalid API origin."))?;
    if url.scheme() != "https"
        && !(url.scheme() == "http"
            && ["localhost", "127.0.0.1", "[::1]"].contains(&url.host_str().unwrap_or("")))
    {
        return Err(Error::bad("Use HTTPS for remote API credentials."));
    }
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|_| Error::internal())?;
    let method = reqwest::Method::from_bytes(method.as_bytes())
        .map_err(|_| Error::bad("Invalid method."))?;
    let mut req = client
        .request(method, format!("{}{path}", base.trim_end_matches('/')))
        .bearer_auth(key);
    if let Some(v) = input {
        req = req.json(&v)
    }
    let response = req
        .send()
        .await
        .map_err(|_| Error::unavailable("API request failed."))?;
    let status = response.status();
    let body = response
        .json()
        .await
        .map_err(|_| Error::bad("API did not return JSON."))?;
    if !status.is_success() {
        return Err(Error::bad(format!("API status {status}: {body}")));
    }
    Ok(body)
}
async fn run() -> Result<()> {
    match Cli::parse().command {
        Command::Backup {
            output,
            tombstones_output,
        } => {
            let config = Config::from_env()?;
            let store = sami::storage::Store::connect(&config.database_url, config.crypto).await?;
            let backup = store.backup().await?;
            let decoded = store
                .crypto
                .open(backup["payload"].as_str().unwrap_or(""), "sami/backup/v1")?;
            let ledger: Vec<_> = decoded["tables"]["sami_records"]
                .as_array()
                .into_iter()
                .flatten()
                .filter(|r| r["kind"] == "tombstones" && r["deleted"] == 0)
                .map(|r| json!({"tenant":r["tenant"],"source_id":r["id"]}))
                .collect();
            save_private(&output, &backup)?;
            save_private(&tombstones_output, &json!(ledger))?;
            print(
                &json!({"status":"backed_up","encrypted":true,"notice":"Retain the newest tombstone ledger independently, and back up encryption/signing keys separately."}),
            )
        }
        Command::Restore {
            input,
            tombstones_file,
        } => {
            let config = Config::from_env()?;
            let store = sami::storage::Store::connect(&config.database_url, config.crypto).await?;
            let bytes = std::fs::read(input)?;
            if bytes.len() > 128 * 1024 * 1024 {
                return Err(Error::bad("Portable restore file exceeds 128 MiB."));
            }
            let backup = serde_json::from_slice(&bytes)?;
            print(&store.restore(&backup, &read(&tombstones_file)?).await?)
        }
        Command::Export {
            output,
            api_url,
            api_key,
        } => {
            save_private(
                &output,
                &request(&api_url, "/v1/export", "GET", &api_key, None).await?,
            )?;
            print(&json!({"status":"exported","audit":"Paginated separately through /v1/audit"}))
        }
        Command::Publish {
            id,
            kind,
            reason,
            claims,
            api_url,
            api_key,
        } => {
            if !["sources", "packs"].contains(&kind.as_str())
                || id.contains('/')
                || id.contains('\\')
            {
                return Err(Error::bad(
                    "Publish supports sources or packs with a valid ID.",
                ));
            }
            let mut body = json!({"reason":reason});
            if let Some(path) = claims {
                body["claims"] = read(&path)?;
            }
            print(
                &request(
                    &api_url,
                    &format!("/v1/{kind}/{id}/publish"),
                    "POST",
                    &api_key,
                    Some(body),
                )
                .await?,
            )
        }
        Command::Evaluate { input, operation } => print(
            &sami_research::dispatch(&operation, &read(&input)?)
                .map_err(|e| Error::bad(e.to_string()))?,
        ),
        Command::Ingest { input } => print(
            &sami_research::dispatch("ingestion.extract", &read(&input)?)
                .map_err(|e| Error::bad(e.to_string()))?,
        ),
        Command::ValidatePack { input } => print(
            &sami_research::dispatch("pack.validate", &read(&input)?)
                .map_err(|e| Error::bad(e.to_string()))?,
        ),
        Command::Replay {
            receipt_id,
            api_url,
            api_key,
        } => print(
            &request(
                &api_url,
                &format!("/v1/receipts/{receipt_id}/replay"),
                "POST",
                &api_key,
                Some(json!({})),
            )
            .await?,
        ),
        Command::Request {
            path,
            method,
            input,
            api_url,
            api_key,
        } => print(
            &request(
                &api_url,
                &path,
                &method,
                &api_key,
                input.map(|p| read(&p)).transpose()?,
            )
            .await?,
        ),
        command => {
            let config = Config::from_env()?;
            let service = Arc::new(Service::new(config).await?);
            match command {
                Command::Serve { host, port } => {
                    let listener = tokio::net::TcpListener::bind(format!("{host}:{port}")).await?;
                    tracing::info!(address=%listener.local_addr()?,"SAMI server listening");
                    axum::serve(listener, sami::api::router(service))
                        .with_graceful_shutdown(async {
                            let _ = tokio::signal::ctrl_c().await;
                        })
                        .await?;
                    Ok(())
                }
                Command::Init { seed_demo, tenant } => {
                    service.store.ensure_tenant(&tenant).await?;
                    if service.config.bootstrap_key.is_none() && tenant == "default" {
                        let key = token();
                        let c = Credential {
                            id: uuid::Uuid::new_v4().to_string(),
                            tenant: tenant.clone(),
                            principal: "local-owner".into(),
                            scopes: vec!["*".into()],
                            groups: vec![],
                            expires_at: 0,
                            revoked: false,
                        };
                        service
                            .store
                            .add_credential(&digest(key.as_bytes()), &c)
                            .await?;
                        print(
                            &json!({"bootstrap_key":key,"notice":"Secret returned once. Store privately; do not commit."}),
                        )?;
                    }
                    if seed_demo {
                        print(&service.seed_demo(&tenant).await?)?
                    } else {
                        print(&json!({"status":"initialized","tenant":tenant}))?
                    }
                    Ok(())
                }
                Command::ProvisionKey {
                    tenant,
                    principal,
                    scopes,
                    groups,
                    expires_at,
                } => {
                    if scopes.is_empty() {
                        return Err(Error::bad("At least one --scopes value is required."));
                    }
                    let key = token();
                    let c = Credential {
                        id: uuid::Uuid::new_v4().to_string(),
                        tenant,
                        principal,
                        scopes,
                        groups,
                        expires_at,
                        revoked: false,
                    };
                    service
                        .store
                        .add_credential(&digest(key.as_bytes()), &c)
                        .await?;
                    service
                        .store
                        .commit(
                            &c.tenant,
                            "local-operator",
                            "credential.provisioned",
                            vec![],
                            None,
                        )
                        .await?;
                    print(
                        &json!({"key":key,"credential":c,"notice":"CLI database access is operator authority; protect this host."}),
                    )
                }
                Command::DeriveDeviceKey { tenant, device_id } => {
                    if tenant.is_empty() || device_id.is_empty() {
                        return Err(Error::bad("Tenant and device identity are required."));
                    }
                    print(
                        &json!({"tenant":tenant,"device_id":device_id,"bundle_encryption_key":service.config.crypto.device_key(&tenant,&device_id),"issuer_public_key":service.config.crypto.public_key(),"notice":"Provision securely. Never commit this decryption secret."}),
                    )
                }
                Command::Doctor => {
                    service.store.pool.acquire().await?;
                    print(
                        &json!({"database":"reachable","production":service.config.production,"profile":"native_strict","version":env!("CARGO_PKG_VERSION"),"console_present":service.config.console_dir.join("index.html").exists(),"oidc_configured":service.config.oidc_issuer.is_some(),"device_id":service.config.device_id,"signing_public_key":service.config.crypto.public_key(),"external_deployment_certified":false}),
                    )
                }
                Command::Worker {
                    api_key,
                    interval_seconds,
                    once,
                } => loop {
                    let p = service.auth.authenticate(&api_key).await?;
                    let v = service.worker_once(&p).await?;
                    if once {
                        return print(&v);
                    }
                    tracing::info!(
                        count = v["reconciliations"].as_array().map_or(0, Vec::len),
                        "reconciliation pass"
                    );
                    tokio::select! {_=tokio::time::sleep(std::time::Duration::from_secs(interval_seconds.clamp(5,300)))=>{},_=tokio::signal::ctrl_c()=>return Ok(())}
                },
                _ => unreachable!(),
            }
        }
    }
}
#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "sami=info,tower_http=info".into()),
        )
        .json()
        .with_writer(std::io::stderr)
        .init();
    if let Err(e) = run().await {
        eprintln!("{}: {}", e.code, e.message);
        std::process::exit(1)
    }
}
