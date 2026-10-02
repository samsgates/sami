use crate::error::{Error, Result};
use aes_gcm::{
    aead::{Aead, KeyInit, Payload},
    Aes256Gcm, Nonce,
};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use ed25519_dalek::{Signature, Signer, SigningKey, VerifyingKey};
use hmac::{Hmac, Mac};
use rand::RngCore;
use serde_json::Value;
use sha2::{Digest, Sha256};

#[derive(Clone)]
pub struct Crypto {
    cipher: Aes256Gcm,
    encryption_seed: [u8; 32],
    signing: SigningKey,
}
pub fn canonical(value: &Value) -> Vec<u8> {
    fn order(v: &Value) -> Value {
        match v {
            Value::Object(o) => {
                let mut keys: Vec<_> = o.keys().collect();
                keys.sort();
                Value::Object(
                    keys.into_iter()
                        .map(|k| (k.clone(), order(&o[k])))
                        .collect(),
                )
            }
            Value::Array(a) => Value::Array(a.iter().map(order).collect()),
            _ => v.clone(),
        }
    }
    serde_json::to_vec(&order(value)).unwrap_or_default()
}
pub fn digest(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}
pub fn token() -> String {
    let mut b = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut b);
    URL_SAFE_NO_PAD.encode(b)
}
impl Crypto {
    pub fn new(encryption: &str, signing: &str) -> Result<Self> {
        let parse = |s: &str| -> Result<[u8; 32]> {
            let v = hex::decode(s)
                .map_err(|_| Error::bad("Keys must contain 64 hexadecimal characters."))?;
            v.try_into()
                .map_err(|_| Error::bad("Keys must be exactly 32 bytes."))
        };
        let enc = parse(encryption)?;
        let signing = parse(signing)?;
        Ok(Self {
            cipher: Aes256Gcm::new_from_slice(&enc).map_err(|_| Error::internal())?,
            encryption_seed: enc,
            signing: SigningKey::from_bytes(&signing),
        })
    }
    /// A device-specific key is provisioned through a separate secure channel.
    /// No database master key or issuer signing seed needs to leave the issuer.
    pub fn device_key(&self, tenant: &str, device: &str) -> String {
        let mut mac = <Hmac<Sha256> as Mac>::new_from_slice(&self.encryption_seed)
            .expect("HMAC accepts a fixed-size key");
        mac.update(&canonical(&serde_json::json!([
            "sami.bundle.device.v1",
            tenant,
            device
        ])));
        hex::encode(mac.finalize().into_bytes())
    }
    fn device_cipher(key: &str) -> Result<Aes256Gcm> {
        let bytes = hex::decode(key).map_err(|_| Error::bad("Invalid device key."))?;
        Aes256Gcm::new_from_slice(&bytes).map_err(|_| Error::bad("Device key must be 32 bytes."))
    }
    pub fn seal_device(value: &Value, aad: &str, key: &str) -> Result<String> {
        Self::seal_cipher(&Self::device_cipher(key)?, value, aad)
    }
    pub fn open_device(value: &str, aad: &str, key: &str) -> Result<Value> {
        Self::open_cipher(&Self::device_cipher(key)?, value, aad)
    }
    pub fn seal(&self, value: &Value, aad: &str) -> Result<String> {
        Self::seal_cipher(&self.cipher, value, aad)
    }
    fn seal_cipher(cipher: &Aes256Gcm, value: &Value, aad: &str) -> Result<String> {
        let mut n = [0u8; 12];
        rand::thread_rng().fill_bytes(&mut n);
        let ct = cipher
            .encrypt(
                Nonce::from_slice(&n),
                Payload {
                    msg: &canonical(value),
                    aad: aad.as_bytes(),
                },
            )
            .map_err(|_| Error::internal())?;
        Ok(URL_SAFE_NO_PAD.encode([n.to_vec(), ct].concat()))
    }
    pub fn open(&self, value: &str, aad: &str) -> Result<Value> {
        Self::open_cipher(&self.cipher, value, aad)
    }
    fn open_cipher(cipher: &Aes256Gcm, value: &str, aad: &str) -> Result<Value> {
        let b = URL_SAFE_NO_PAD
            .decode(value)
            .map_err(|_| Error::internal())?;
        if b.len() < 28 {
            return Err(Error::internal());
        }
        let clear = cipher
            .decrypt(
                Nonce::from_slice(&b[..12]),
                Payload {
                    msg: &b[12..],
                    aad: aad.as_bytes(),
                },
            )
            .map_err(|_| Error::internal())?;
        Ok(serde_json::from_slice(&clear)?)
    }
    pub fn sign(&self, value: &Value) -> String {
        hex::encode(self.signing.sign(&canonical(value)).to_bytes())
    }
    pub fn verify(&self, value: &Value, signature: &str) -> bool {
        verify_public(value, signature, &self.public_key())
    }
    pub fn public_key(&self) -> String {
        hex::encode(self.signing.verifying_key().as_bytes())
    }
}
pub fn verify_public(value: &Value, signature: &str, public_key: &str) -> bool {
    let Ok(b) = hex::decode(public_key) else {
        return false;
    };
    let Ok(k): std::result::Result<[u8; 32], _> = b.try_into() else {
        return false;
    };
    let Ok(k) = VerifyingKey::from_bytes(&k) else {
        return false;
    };
    let Ok(s) = hex::decode(signature) else {
        return false;
    };
    let Ok(s) = Signature::from_slice(&s) else {
        return false;
    };
    k.verify_strict(&canonical(value), &s).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn device_keys_separate_tenants_devices_and_master_key() {
        let c = Crypto::new(&"01".repeat(32), &"02".repeat(32)).unwrap();
        let k = c.device_key("a", "one");
        assert_ne!(k, c.device_key("b", "one"));
        assert_ne!(k, c.device_key("a", "two"));
        let v = serde_json::json!({"x":1});
        let ct = Crypto::seal_device(&v, "bundle/a/one", &k).unwrap();
        assert_eq!(Crypto::open_device(&ct, "bundle/a/one", &k).unwrap(), v);
        assert!(c.open(&ct, "bundle/a/one").is_err());
        assert!(Crypto::open_device(&ct, "bundle/a/two", &k).is_err());
    }
    #[test]
    fn authenticated_scope_and_signatures() {
        let c = Crypto::new(&"01".repeat(32), &"02".repeat(32)).unwrap();
        let v = serde_json::json!({"x":1});
        let ct = c.seal(&v, "tenant:a").unwrap();
        assert_eq!(c.open(&ct, "tenant:a").unwrap(), v);
        assert!(c.open(&ct, "tenant:b").is_err());
        let s = c.sign(&v);
        assert!(c.verify(&v, &s));
        assert!(!c.verify(&serde_json::json!({"x":2}), &s));
    }
}
