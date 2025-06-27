use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Key, Nonce};
use rand::RngCore;
use std::io;
use std::path::Path;

const KEY_BYTES: &[u8; 32] = b"an example very very secret key.";

pub fn encrypt_file(src: &Path, dest: &Path) -> io::Result<()> {
    let data = std::fs::read(src)?;
    let key = Key::from_slice(KEY_BYTES);
    let cipher = Aes256Gcm::new(key);
    let mut nonce_bytes = [0u8; 12];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    let ciphertext = cipher
        .encrypt(nonce, data.as_ref())
        .map_err(|_| io::Error::new(io::ErrorKind::Other, "encrypt"))?;
    let mut out = Vec::new();
    out.extend_from_slice(&nonce_bytes);
    out.extend_from_slice(&ciphertext);
    std::fs::write(dest, out)?;
    Ok(())
}

pub fn decrypt_file(src: &Path, dest: &Path) -> io::Result<()> {
    let data = std::fs::read(src)?;
    if data.len() < 12 {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "too short"));
    }
    let (nonce_bytes, ciphertext) = data.split_at(12);
    let key = Key::from_slice(KEY_BYTES);
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(nonce_bytes);
    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| io::Error::new(io::ErrorKind::Other, "decrypt"))?;
    std::fs::write(dest, plaintext)?;
    Ok(())
}
