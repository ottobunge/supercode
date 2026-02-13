//! Key generation for peer authentication

use anyhow::{Context, Result};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};

/// Generate a new key pair for this node (x25519 for key exchange)
pub fn generate_keypair() -> Result<(String, String)> {
    use rand::rngs::OsRng;
    use x25519_dalek::{PublicKey, StaticSecret};

    let mut rng = OsRng;
    let static_secret = StaticSecret::random_from_rng(&mut rng);
    let static_public = PublicKey::from(&static_secret);

    Ok((
        BASE64.encode(static_secret.to_bytes()),
        BASE64.encode(static_public.to_bytes()),
    ))
}

/// Get public key from private (for x25519)
pub fn get_public_key(private_key_base64: &str) -> Result<String> {
    use x25519_dalek::{PublicKey, StaticSecret};

    let bytes = BASE64.decode(private_key_base64)?;
    if bytes.len() != 32 {
        anyhow::bail!("Invalid key length, expected 32 bytes, got {}", bytes.len());
    }
    let mut arr = [0u8; 32];
    arr.copy_from_slice(&bytes);
    let secret = StaticSecret::from(arr);
    let public = PublicKey::from(&secret);
    Ok(BASE64.encode(public.to_bytes()))
}
