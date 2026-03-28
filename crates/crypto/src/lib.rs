pub mod encryption;
pub mod wireguard;

pub use encryption::{encrypt_key, decrypt_key};
pub use wireguard::{generate_keypair, KeyPair};

pub struct KeyGenerator;

impl KeyGenerator {
    pub fn generate_keypair() -> (String, String) {
        // For MVP, generate mock keypairs
        // In production, use x25519-dalek
        use rand::Rng;
        let mut rng = rand::thread_rng();
        
        let mut private_key_bytes = [0u8; 32];
        rng.fill(&mut private_key_bytes);
        
        let mut public_key_bytes = [0u8; 32];
        rng.fill(&mut public_key_bytes);
        
        let private_key = base64_encode(&private_key_bytes);
        let public_key = base64_encode(&public_key_bytes);
        
        (public_key, private_key)
    }
}

fn base64_encode(data: &[u8]) -> String {
    // use std::fmt::Write;
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::new();
    
    for chunk in data.chunks(3) {
        let mut buf = [0u8; 3];
        for (i, &byte) in chunk.iter().enumerate() {
            buf[i] = byte;
        }
        
        let b1 = (buf[0] >> 2) as usize;
        let b2 = (((buf[0] & 0x03) << 4) | (buf[1] >> 4)) as usize;
        let b3 = (((buf[1] & 0x0f) << 2) | (buf[2] >> 6)) as usize;
        let b4 = (buf[2] & 0x3f) as usize;
        
        result.push(CHARSET[b1] as char);
        result.push(CHARSET[b2] as char);
        
        if chunk.len() > 1 {
            result.push(CHARSET[b3] as char);
        } else {
            result.push('=');
        }
        
        if chunk.len() > 2 {
            result.push(CHARSET[b4] as char);
        } else {
            result.push('=');
        }
    }
    
    result
}
