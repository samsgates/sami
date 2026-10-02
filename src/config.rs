use crate::{
    crypto::{token, Crypto},
    error::{Error, Result},
};
use std::{env, path::PathBuf};

#[derive(Clone)]
pub struct Config {
    pub database_url: String,
    pub data_dir: PathBuf,
    pub crypto: Crypto,
    pub production: bool,
    pub bootstrap_key: Option<String>,
    pub console_dir: PathBuf,
    pub oidc_issuer: Option<String>,
    pub oidc_audience: Option<String>,
    pub oidc_jwks: Option<String>,
    pub allowed_hosts: Vec<String>,
    pub device_id: Option<String>,
    pub bundle_trust_keys: Vec<String>,
    pub bundle_decryption_key: Option<String>,
}
impl Config {
    pub fn from_env() -> Result<Self> {
        let data_dir = PathBuf::from(env::var("SAMI_DATA_DIR").unwrap_or_else(|_| "data".into()));
        std::fs::create_dir_all(&data_dir)?;
        let production = env::var("SAMI_ENV").is_ok_and(|v| v == "production");
        let local_key = |name: &str, file: &str| -> Result<String> {
            if let Ok(k) = env::var(name) {
                return Ok(k);
            }
            if production {
                return Err(Error::bad(format!("{name} is mandatory in production.")));
            }
            let p = data_dir.join(file);
            if p.exists() {
                return Ok(std::fs::read_to_string(p)?.trim().into());
            }
            let k = crate::crypto::digest(token().as_bytes());
            use std::io::Write;
            let mut options = std::fs::OpenOptions::new();
            options.write(true).create_new(true);
            #[cfg(unix)]
            {
                use std::os::unix::fs::OpenOptionsExt;
                options.mode(0o600);
            }
            match options.open(&p) {
                Ok(mut file) => {
                    file.write_all(k.as_bytes())?;
                    file.sync_all()?;
                }
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                    return Ok(std::fs::read_to_string(p)?.trim().into())
                }
                Err(e) => return Err(e.into()),
            }
            Ok(k)
        };
        let crypto = Crypto::new(
            &local_key("SAMI_ENCRYPTION_KEY", ".encryption.key")?,
            &local_key("SAMI_SIGNING_KEY", ".signing.key")?,
        )?;
        let bootstrap_key = env::var("SAMI_BOOTSTRAP_KEY").ok();
        if bootstrap_key.as_ref().is_some_and(|k| k.len() < 32) {
            return Err(Error::bad(
                "SAMI_BOOTSTRAP_KEY must contain at least 32 characters.",
            ));
        }
        Ok(Self {
            database_url: env::var("SAMI_DATABASE_URL").unwrap_or_else(|_| {
                format!("sqlite://{}?mode=rwc", data_dir.join("sami.db").display())
            }),
            data_dir,
            crypto,
            production,
            bootstrap_key,
            console_dir: PathBuf::from(
                env::var("SAMI_CONSOLE_DIR").unwrap_or_else(|_| "console/dist".into()),
            ),
            oidc_issuer: env::var("SAMI_OIDC_ISSUER").ok(),
            oidc_audience: env::var("SAMI_OIDC_AUDIENCE").ok(),
            oidc_jwks: env::var("SAMI_OIDC_JWKS_URL").ok(),
            device_id: env::var("SAMI_DEVICE_ID").ok(),
            bundle_trust_keys: env::var("SAMI_BUNDLE_TRUST_KEYS")
                .unwrap_or_default()
                .split(',')
                .filter(|s| !s.trim().is_empty())
                .map(|s| s.trim().into())
                .collect(),
            bundle_decryption_key: env::var("SAMI_BUNDLE_ENCRYPTION_KEY").ok(),
            allowed_hosts: env::var("SAMI_CONNECTOR_HOSTS")
                .unwrap_or_default()
                .split(',')
                .filter(|s| !s.trim().is_empty())
                .map(|s| s.trim().into())
                .collect(),
        })
    }
}
