use crate::types::errors::Result;

#[derive(Clone, Debug)]
pub struct Signature(pub Vec<u8>);

pub trait Attestor: Send + Sync {
    fn sign(&self, bundle: &[u8]) -> Result<Signature>;
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
