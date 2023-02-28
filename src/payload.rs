use hmac::{Hmac, Mac};
use sha2::Sha256;

#[derive(Debug)]
pub struct PayloadError;

impl std::fmt::Display for PayloadError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Error validating payload")
    }
}

impl std::error::Error for PayloadError {}

type HmacSha256 = Hmac<Sha256>;

pub fn verify_payload(secret: &str, signature: &str, payload: &[u8]) -> Result<(), PayloadError> {
    let signature = signature.strip_prefix("sha256=").ok_or(PayloadError)?;
    let signature = hex::decode(signature).map_err(|_| PayloadError)?;

    let mut secret = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
    secret.update(payload);
    secret.verify_slice(&signature).map_err(|_| PayloadError)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verifies_payload() {
        let secret = "secret key";
        let signature = "sha256=5648bac13cba31903089f25b0613926e36fd61bc26d68d336b4386fd4a8ab3f6";
        let payload = "This is a test.";
        let payload_is_valid = verify_payload(secret, signature, payload.as_bytes());

        assert!(payload_is_valid.is_ok());
    }
}
