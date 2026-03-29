use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use sha2::{Digest, Sha256};

fn get_encryption_key() -> [u8; 32] {
    let secret =
        std::env::var("ENCRYPTION_KEY").expect("ENCRYPTION_KEY environment variable must be set");
    let mut hasher = Sha256::new();
    hasher.update(secret.as_bytes());
    let result = hasher.finalize();
    let mut key = [0u8; 32];
    key.copy_from_slice(&result);
    key
}

pub fn encrypt(plaintext: &str) -> Result<String, String> {
    let key = get_encryption_key();
    let cipher = Aes256Gcm::new_from_slice(&key).map_err(|e| e.to_string())?;

    let nonce_bytes: [u8; 12] = rand::random();
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_bytes())
        .map_err(|e| e.to_string())?;

    let mut combined = nonce_bytes.to_vec();
    combined.extend(ciphertext);

    Ok(BASE64.encode(&combined))
}

pub fn decrypt(encrypted: &str) -> Result<String, String> {
    let key = get_encryption_key();
    let cipher = Aes256Gcm::new_from_slice(&key).map_err(|e| e.to_string())?;

    let combined = BASE64.decode(encrypted).map_err(|e| e.to_string())?;

    if combined.len() < 12 {
        return Err("Invalid encrypted data".to_string());
    }

    let (nonce_bytes, ciphertext) = combined.split_at(12);
    let nonce = Nonce::from_slice(nonce_bytes);

    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| e.to_string())?;

    String::from_utf8(plaintext).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Once;

    static INIT: Once = Once::new();

    fn setup_test_key() {
        INIT.call_once(|| {
            std::env::set_var("ENCRYPTION_KEY", "test-encryption-key-for-unit-tests-32b");
        });
    }

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        setup_test_key();
        let original = "my-secret-api-key-12345";
        let encrypted = encrypt(original).expect("Encryption should succeed");
        assert_ne!(
            encrypted, original,
            "Encrypted value should differ from original"
        );

        let decrypted = decrypt(&encrypted).expect("Decryption should succeed");
        assert_eq!(decrypted, original, "Decrypted value should match original");
    }

    #[test]
    fn test_encrypt_produces_different_ciphertext() {
        setup_test_key();
        let original = "same-input";
        let encrypted1 = encrypt(original).expect("Encryption should succeed");
        let encrypted2 = encrypt(original).expect("Encryption should succeed");

        assert_ne!(
            encrypted1, encrypted2,
            "Each encryption should produce different ciphertext due to random nonce"
        );

        assert_eq!(decrypt(&encrypted1).unwrap(), decrypt(&encrypted2).unwrap());
    }

    #[test]
    fn test_decrypt_invalid_base64() {
        setup_test_key();
        let result = decrypt("not-valid-base64!!!");
        assert!(result.is_err(), "Should fail on invalid base64");
    }

    #[test]
    fn test_decrypt_too_short() {
        setup_test_key();
        let short_input = BASE64.encode([0u8; 5]);
        let result = decrypt(&short_input);
        assert!(
            result.is_err(),
            "Should fail on data too short to contain nonce"
        );
    }
}

#[cfg(test)]
mod encrypt_util {
    use super::*;
    use std::sync::Once;

    static INIT: Once = Once::new();

    #[test]
    #[ignore]
    fn print_encrypted_deepseek_key() {
        INIT.call_once(|| {
            std::env::set_var("ENCRYPTION_KEY", "dev-encryption-key-change-in-production");
        });
        let api_key = "sk-21bb2908dcb7437bb0986e08b2a6fa48";
        let encrypted = encrypt(api_key).unwrap();
        eprintln!("ENCRYPTED_KEY: {}", encrypted);
        panic!("Encrypted key output");
    }

    #[test]
    #[ignore]
    fn decrypt_test3_key() {
        INIT.call_once(|| {
            std::env::set_var("ENCRYPTION_KEY", "dev-encryption-key-change-in-production");
        });
        let encrypted =
            "ISECJfpzW7t5b030jVMPxuv8f6wEuEIWdgUDSPg6OIEmZkwqX/Yl+T52FAEk96QvaHz27leN3Bymm84unebY";
        let decrypted = decrypt(encrypted).unwrap();
        eprintln!("DECRYPTED_KEY: {}", decrypted);
        panic!("Decrypted key output");
    }
}
