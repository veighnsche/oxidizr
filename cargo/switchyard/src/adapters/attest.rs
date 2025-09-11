use crate::types::errors::Result;

#[derive(Clone, Debug)]
pub struct Signature(pub Vec<u8>);

pub trait Attestor {
    fn sign(&self, bundle: &[u8]) -> Result<Signature>;
}
