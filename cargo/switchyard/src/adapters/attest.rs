use thiserror::Error;
use crate::types::errors::{Error, ErrorKind};

#[derive(Debug, Error)]
pub enum AttestationError {
    #[error("attestation signing failed: {msg}")]
    Signing { msg: String },
    #[error("attestation verification failed: {msg}")]
    Verification { msg: String },
}

impl From<AttestationError> for Error {
    fn from(e: AttestationError) -> Self {
        Error {
            kind: ErrorKind::Io,  // Using Io for now, could be more specific
            msg: e.to_string(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Signature(pub Vec<u8>);

pub trait Attestor: Send + Sync {
    /// Sign the given bundle.
    /// # Errors
    /// Returns an error if signing fails.
    fn sign(&self, bundle: &[u8]) -> Result<Signature, AttestationError>;
    /// Return a stable identifier for the public key used to sign (e.g., fingerprint or KID).
    fn key_id(&self) -> String;
    /// Return the signature algorithm label for attestations. Defaults to "ed25519".
    fn algorithm(&self) -> &'static str {
        "ed25519"
    }
}

/// Build a JSON object with attestation fields for emission given an attestor and a bundle.
/// Returns None if signing fails.
pub fn build_attestation_fields(att: &dyn Attestor, bundle: &[u8]) -> Option<serde_json::Value> {
    use base64::Engine as _;
    use sha2::Digest as _;
    let sig = att.sign(bundle).ok()?;
    let sig_b64 = base64::engine::general_purpose::STANDARD.encode(sig.0.clone());
    let mut hasher = sha2::Sha256::new();
    hasher.update(bundle);
    let bundle_hash = hex::encode(hasher.finalize());
    Some(serde_json::json!({
        "sig_alg": att.algorithm(),
        "signature": sig_b64,
        "bundle_hash": bundle_hash,
        "public_key_id": att.key_id(),
    }))
}
