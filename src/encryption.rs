use chacha20::cipher::{KeyIvInit, StreamCipher};
use chacha20::ChaCha20;
use rand::rngs::OsRng;
use rand::RngCore;

use std::error::Error;

pub struct EncryptDecrypt {
    key: [u8; 32],
    iv: [u8; 12],
}

impl EncryptDecrypt {
    pub fn new() -> Self {
        let mut key = [0u8; 32];
        let mut nonce: [u8; 12] = [0u8; 12];
        OsRng.fill_bytes(&mut key);
        OsRng.fill_bytes(&mut nonce);
        Self {
            key: key.try_into().unwrap(),
            iv: nonce.try_into().unwrap(),
        }
    }

    pub fn get_key(&self) -> &[u8] {
        &self.key
    }

    pub fn get_iv(&self) -> &[u8] {
        &self.iv
    }

    pub fn encrypt_string_with_stream_cipher(&self, plaintext: &str) -> Result<Vec<u8>, Box<dyn Error>> {
        let mut cipher = ChaCha20::new_from_slices(&self.key, &self.iv).map_err(|e| e.to_string())?;

        let mut data = plaintext.as_bytes().to_vec();
        cipher.apply_keystream(&mut data);
        Ok(data)
    }

    pub fn decrypt_string_with_stream_cipher(data: &Vec<u8>, key: &[u8], iv: &[u8]) -> Result<String, Box<dyn Error>> {
        let mut cipher =
            ChaCha20::new_from_slices(key, iv).map_err(|e| e.to_string())?;

        let mut decrypted_data = data.clone();
        cipher.apply_keystream(&mut decrypted_data);

        Ok(String::from_utf8_lossy(&decrypted_data).into_owned())
    }
}

// pub fn encrypt_string_with_stream_cipher(plaintext: &str) -> Result<DataKeyPair, Box<dyn Error>> {
//     let mut key = [0u8; 32];
//     let mut nonce: [u8; 12] = [0u8; 12];
//     OsRng.fill_bytes(&mut key);
//     OsRng.fill_bytes(&mut nonce);
//
//     let mut cipher = ChaCha20::new_from_slices(&key, &nonce).map_err(|e| e.to_string())?;
//
//     let mut data = plaintext.as_bytes().to_vec();
//     cipher.apply_keystream(&mut data);
//     return Ok(DataKeyPair {
//         data,
//         key,
//         iv: nonce,
//     });
// }
//
// pub fn decrypt_string_with_stream_cipher(data: &DataKeyPair) -> Result<String, Box<dyn Error>> {
//     let mut cipher =
//         ChaCha20::new_from_slices(data.get_key(), data.get_iv()).map_err(|e| e.to_string())?;
//
//     let mut decrypted_data = data.get_data().to_vec();
//     cipher.apply_keystream(&mut decrypted_data);
//
//     Ok(String::from_utf8_lossy(&decrypted_data).into_owned())
// }
