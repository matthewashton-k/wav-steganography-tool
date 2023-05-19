use std::error::Error;
use chacha20::{ChaCha20};
use chacha20::cipher::{ KeyIvInit, StreamCipher};
use rand::rngs::OsRng;
use rand::RngCore;
use serde_json::error::Category::Data;

pub struct DataKeyPair {
    data: Vec<u8>,
    key: [u8; 32],
    iv: [u8; 12],
}

impl DataKeyPair {
    pub fn get_data(&self) -> &[u8] {
        &self.data
    }
    pub fn new(data: &[u8], key: &[u8], iv: &[u8]) -> DataKeyPair {
        DataKeyPair {
            data: data.to_vec(),
            key: key.try_into().unwrap(),
            iv: iv.try_into().unwrap(),
        }
    }

    pub fn get_key(&self) -> &[u8] {
        &self.key
    }

    pub fn get_iv(&self) -> &[u8] {
        &self.iv
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }
}

pub fn encrypt_string_with_stream_cipher(plaintext: &str) -> Result<DataKeyPair, Box<dyn Error>> {
    let mut key = [0u8; 32];
    let mut nonce: [u8; 12] = [0u8; 12];
    OsRng.fill_bytes(&mut key);
    OsRng.fill_bytes(&mut nonce);

    let mut cipher = ChaCha20::new_from_slices(&key, &nonce).map_err(|e| e.to_string())?;

    let mut data = plaintext.as_bytes().to_vec();
    cipher.apply_keystream(&mut data);
    return Ok(DataKeyPair{
        data,
        key,
        iv: nonce,
    });
}

pub fn decrypt_string_with_stream_cipher(data: &DataKeyPair) -> Result<String, Box<dyn Error>> {
    let mut cipher = ChaCha20::new_from_slices(data.get_key(), data.get_iv()).map_err(|e| e.to_string())?;

    let mut decrypted_data = data.get_data().to_vec();
    cipher.apply_keystream(&mut decrypted_data);

    Ok(String::from_utf8_lossy(&decrypted_data).into_owned())
}


