use rand::Rng;
use vpn_shared::AppError;

#[derive(Debug, Clone)]
pub struct KeyPair {
    pub private_key: String,  // base64 encoded
    pub public_key: String,   // base64 encoded
}

/// Generate a WireGuard keypair using x25519
/// For MVP, we use a simple random generation
/// In production, would use proper x25519 library
pub fn generate_keypair() -> Result<KeyPair, AppError> {
    let mut rng = rand::thread_rng();
    
    // Generate 32-byte private key
    let mut private_key_bytes = [0u8; 32];
    rng.fill(&mut private_key_bytes);
    
    // Clamp for Curve25519
    let private_key_bytes = clamp_private_key(private_key_bytes);
    
    let private_key = base64::encode(&private_key_bytes);
    let public_key = derive_public_key(&private_key_bytes)?;
    
    Ok(KeyPair {
        private_key,
        public_key,
    })
}

fn clamp_private_key(mut key: [u8; 32]) -> [u8; 32] {
    key[0] &= 248;
    key[31] = (key[31] & 127) | 64;
    key
}

fn derive_public_key(_private_key: &[u8; 32]) -> Result<String, AppError> {
    // For MVP, use a mock derivation
    // In production, use x25519-dalek or similar
    let mut rng = rand::thread_rng();
    let mut public_key_bytes = [0u8; 32];
    rng.fill(&mut public_key_bytes);
    
    Ok(base64::encode(&public_key_bytes))
}

mod base64 {
    pub fn encode(data: &[u8]) -> String {
        use std::fmt::Write;
        let mut result = String::new();
        for byte in data {
            write!(&mut result, "{:02x}", byte).unwrap();
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_keypair() {
        let keypair = generate_keypair().unwrap();
        assert!(!keypair.private_key.is_empty());
        assert!(!keypair.public_key.is_empty());
        assert_eq!(keypair.private_key.len(), 64);  // 32 bytes hex encoded
        assert_eq!(keypair.public_key.len(), 64);
    }
}
