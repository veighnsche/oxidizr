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
