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

pub fn verify_payload(signature: &str, payload: &[u8]) -> Result<(), PayloadError> {
    let signature = match signature.get(7..) {
        Some(sig) => sig,
        None => return Err(PayloadError),
    };
    let signature = match hex::decode(&signature) {
        Ok(e) => e,
        Err(_) => {
            return Err(PayloadError);
        }
    };

    let secret = std::env::var("GITHUB_WEBHOOK_SECRET").expect("GITHUB_WEBHOOK_SECRET not found");
    let mut secret = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
    secret.update(payload);

    if let Err(_) = secret.verify_slice(&signature) {
        return Err(PayloadError);
    }

    Ok(())
}
