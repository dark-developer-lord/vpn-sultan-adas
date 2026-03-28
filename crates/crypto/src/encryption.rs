use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use rand::Rng;
use vpn_shared::AppError;

/// Encrypt data using AES-256-GCM
/// Returns (nonce, ciphertext)
pub fn encrypt_key(master_key: &[u8; 32], plaintext: &[u8]) -> Result<(Vec<u8>, Vec<u8>), AppError> {
    let cipher = Aes256Gcm::new(master_key.into());
    
    // Generate random nonce
    let mut rng = rand::thread_rng();
    let mut nonce_bytes = [0u8; 12];
    rng.fill(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    
    // Encrypt
    let ciphertext = cipher
        .encrypt(nonce, plaintext)
        .map_err(|_| AppError::EncryptionError)?;
    
    Ok((nonce_bytes.to_vec(), ciphertext))
}

/// Decrypt data using AES-256-GCM
pub fn decrypt_key(
    master_key: &[u8; 32],
    nonce: &[u8],
    ciphertext: &[u8],
) -> Result<Vec<u8>, AppError> {
    if nonce.len() != 12 {
        return Err(AppError::DecryptionError);
    }
    
    let cipher = Aes256Gcm::new(master_key.into());
    let nonce = Nonce::from_slice(nonce);
    
    cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| AppError::DecryptionError)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt() {
        let master_key = [42u8; 32];
        let plaintext = b"my-secret-wireguard-key";
        
        let (nonce, ciphertext) = encrypt_key(&master_key, plaintext).unwrap();
        let decrypted = decrypt_key(&master_key, &nonce, &ciphertext).unwrap();
        
        assert_eq!(plaintext, &decrypted[..]);
    }
}
