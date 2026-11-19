use crate::{Result, StorageError};
use aes::cipher::{BlockDecryptMut, BlockEncryptMut, KeyInit, KeyIvInit, block_padding::Pkcs7};
use aes_gcm::{Aes128Gcm, Aes256Gcm, Key, Nonce, aead::Aead};
use hmac::Hmac;
use pbkdf2::pbkdf2_hmac;
use rand::RngCore;
use sha2::{Digest, Sha256, Sha384, Sha512};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DigestAlgorithm {
    Sha256,
    Sha384,
    Sha512,
}

impl DigestAlgorithm {
    pub fn parse(name: &str) -> Result<Self> {
        match name.trim().to_ascii_uppercase().as_str() {
            "SHA-256" | "SHA256" => Ok(Self::Sha256),
            "SHA-384" | "SHA384" => Ok(Self::Sha384),
            "SHA-512" | "SHA512" => Ok(Self::Sha512),
            other => Err(StorageError::InvalidArgument(format!(
                "Unsupported digest algorithm: {other}"
            ))),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AesMode {
    Gcm,
    Cbc,
    Ecb,
}

impl AesMode {
    pub fn parse(mode: &str) -> Result<Self> {
        let up = mode.trim().to_ascii_uppercase();
        if up.contains("GCM") {
            Ok(Self::Gcm)
        } else if up.contains("CBC") {
            Ok(Self::Cbc)
        } else if up.contains("ECB") {
            Ok(Self::Ecb)
        } else {
            Err(StorageError::InvalidArgument(format!(
                "Unsupported AES mode: {mode}"
            )))
        }
    }
}

/// Equivalent to `ProfilePinCrypto.sha256Hex`.
pub fn sha256_hex(value: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(value.as_bytes());
    hex::encode(hasher.finalize())
}

/// Equivalent to `pluginGetRandomValues`.
pub fn get_random_values(len: usize) -> Vec<u8> {
    let mut buf = vec![0u8; len];
    rand::thread_rng().fill_bytes(&mut buf);
    buf
}

/// Equivalent to `pluginDigest`.
pub fn digest(algorithm: DigestAlgorithm, data: &[u8]) -> Vec<u8> {
    match algorithm {
        DigestAlgorithm::Sha256 => Sha256::digest(data).to_vec(),
        DigestAlgorithm::Sha384 => Sha384::digest(data).to_vec(),
        DigestAlgorithm::Sha512 => Sha512::digest(data).to_vec(),
    }
}

/// Equivalent to `pluginPbkdf2`.
pub fn pbkdf2_derive(
    password: &[u8],
    salt: &[u8],
    iterations: u32,
    key_size_bits: usize,
    algorithm: DigestAlgorithm,
) -> Result<Vec<u8>> {
    if iterations == 0 {
        return Err(StorageError::InvalidArgument(
            "PBKDF2 iterations must be positive".into(),
        ));
    }
    if key_size_bits == 0 || !key_size_bits.is_multiple_of(8) {
        return Err(StorageError::InvalidArgument(
            "PBKDF2 key size must be a positive byte-aligned bit length".into(),
        ));
    }
    let mut out = vec![0u8; key_size_bits / 8];
    match algorithm {
        DigestAlgorithm::Sha256 => pbkdf2_hmac::<Sha256>(password, salt, iterations, &mut out),
        DigestAlgorithm::Sha384 => pbkdf2_hmac::<Sha384>(password, salt, iterations, &mut out),
        DigestAlgorithm::Sha512 => pbkdf2_hmac::<Sha512>(password, salt, iterations, &mut out),
    }
    Ok(out)
}

pub fn hmac_sign(algorithm: DigestAlgorithm, key: &[u8], data: &[u8]) -> Result<Vec<u8>> {
    use hmac::Mac;
    match algorithm {
        DigestAlgorithm::Sha256 => {
            let mut mac = <Hmac<Sha256> as Mac>::new_from_slice(key)
                .map_err(|e| StorageError::Crypto(e.to_string()))?;
            mac.update(data);
            Ok(mac.finalize().into_bytes().to_vec())
        }
        DigestAlgorithm::Sha384 => {
            let mut mac = <Hmac<Sha384> as Mac>::new_from_slice(key)
                .map_err(|e| StorageError::Crypto(e.to_string()))?;
            mac.update(data);
            Ok(mac.finalize().into_bytes().to_vec())
        }
        DigestAlgorithm::Sha512 => {
            let mut mac = <Hmac<Sha512> as Mac>::new_from_slice(key)
                .map_err(|e| StorageError::Crypto(e.to_string()))?;
            mac.update(data);
            Ok(mac.finalize().into_bytes().to_vec())
        }
    }
}

/// Equivalent to `pluginAesEncrypt`.
pub fn aes_encrypt(mode: AesMode, key: &[u8], iv: &[u8], data: &[u8]) -> Result<Vec<u8>> {
    match mode {
        AesMode::Gcm => {
            if iv.len() != 12 {
                return Err(StorageError::InvalidArgument(
                    "AES-GCM requires a 12-byte nonce/IV".into(),
                ));
            }
            let nonce = Nonce::from_slice(iv);
            match key.len() {
                16 => {
                    let cipher = Aes128Gcm::new(Key::<Aes128Gcm>::from_slice(key));
                    cipher
                        .encrypt(nonce, data)
                        .map_err(|e| StorageError::Crypto(e.to_string()))
                }
                32 => {
                    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
                    cipher
                        .encrypt(nonce, data)
                        .map_err(|e| StorageError::Crypto(e.to_string()))
                }
                len => Err(StorageError::InvalidArgument(format!(
                    "Unsupported AES-GCM key length: {len} bytes"
                ))),
            }
        }
        AesMode::Cbc => {
            if iv.len() != 16 {
                return Err(StorageError::InvalidArgument(
                    "AES-CBC requires a 16-byte IV".into(),
                ));
            }
            match key.len() {
                16 => Ok(cbc::Encryptor::<aes::Aes128>::new_from_slices(key, iv)
                    .map_err(|e| StorageError::Crypto(e.to_string()))?
                    .encrypt_padded_vec_mut::<Pkcs7>(data)),
                24 => Ok(cbc::Encryptor::<aes::Aes192>::new_from_slices(key, iv)
                    .map_err(|e| StorageError::Crypto(e.to_string()))?
                    .encrypt_padded_vec_mut::<Pkcs7>(data)),
                32 => Ok(cbc::Encryptor::<aes::Aes256>::new_from_slices(key, iv)
                    .map_err(|e| StorageError::Crypto(e.to_string()))?
                    .encrypt_padded_vec_mut::<Pkcs7>(data)),
                len => Err(StorageError::InvalidArgument(format!(
                    "Unsupported AES-CBC key length: {len} bytes"
                ))),
            }
        }
        AesMode::Ecb => match key.len() {
            16 => Ok(ecb::Encryptor::<aes::Aes128>::new_from_slice(key)
                .map_err(|e| StorageError::Crypto(e.to_string()))?
                .encrypt_padded_vec_mut::<Pkcs7>(data)),
            24 => Ok(ecb::Encryptor::<aes::Aes192>::new_from_slice(key)
                .map_err(|e| StorageError::Crypto(e.to_string()))?
                .encrypt_padded_vec_mut::<Pkcs7>(data)),
            32 => Ok(ecb::Encryptor::<aes::Aes256>::new_from_slice(key)
                .map_err(|e| StorageError::Crypto(e.to_string()))?
                .encrypt_padded_vec_mut::<Pkcs7>(data)),
            len => Err(StorageError::InvalidArgument(format!(
                "Unsupported AES-ECB key length: {len} bytes"
            ))),
        },
    }
}

/// Equivalent to `pluginAesDecrypt`.
pub fn aes_decrypt(mode: AesMode, key: &[u8], iv: &[u8], data: &[u8]) -> Result<Vec<u8>> {
    match mode {
        AesMode::Gcm => {
            if iv.len() != 12 {
                return Err(StorageError::InvalidArgument(
                    "AES-GCM requires a 12-byte nonce/IV".into(),
                ));
            }
            let nonce = Nonce::from_slice(iv);
            match key.len() {
                16 => {
                    let cipher = Aes128Gcm::new(Key::<Aes128Gcm>::from_slice(key));
                    cipher
                        .decrypt(nonce, data)
                        .map_err(|e| StorageError::Crypto(e.to_string()))
                }
                32 => {
                    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
                    cipher
                        .decrypt(nonce, data)
                        .map_err(|e| StorageError::Crypto(e.to_string()))
                }
                len => Err(StorageError::InvalidArgument(format!(
                    "Unsupported AES-GCM key length: {len} bytes"
                ))),
            }
        }
        AesMode::Cbc => {
            if iv.len() != 16 {
                return Err(StorageError::InvalidArgument(
                    "AES-CBC requires a 16-byte IV".into(),
                ));
            }
            match key.len() {
                16 => cbc::Decryptor::<aes::Aes128>::new_from_slices(key, iv)
                    .map_err(|e| StorageError::Crypto(e.to_string()))?
                    .decrypt_padded_vec_mut::<Pkcs7>(data)
                    .map_err(|e| StorageError::Crypto(e.to_string())),
                24 => cbc::Decryptor::<aes::Aes192>::new_from_slices(key, iv)
                    .map_err(|e| StorageError::Crypto(e.to_string()))?
                    .decrypt_padded_vec_mut::<Pkcs7>(data)
                    .map_err(|e| StorageError::Crypto(e.to_string())),
                32 => cbc::Decryptor::<aes::Aes256>::new_from_slices(key, iv)
                    .map_err(|e| StorageError::Crypto(e.to_string()))?
                    .decrypt_padded_vec_mut::<Pkcs7>(data)
                    .map_err(|e| StorageError::Crypto(e.to_string())),
                len => Err(StorageError::InvalidArgument(format!(
                    "Unsupported AES-CBC key length: {len} bytes"
                ))),
            }
        }
        AesMode::Ecb => match key.len() {
            16 => ecb::Decryptor::<aes::Aes128>::new_from_slice(key)
                .map_err(|e| StorageError::Crypto(e.to_string()))?
                .decrypt_padded_vec_mut::<Pkcs7>(data)
                .map_err(|e| StorageError::Crypto(e.to_string())),
            24 => ecb::Decryptor::<aes::Aes192>::new_from_slice(key)
                .map_err(|e| StorageError::Crypto(e.to_string()))?
                .decrypt_padded_vec_mut::<Pkcs7>(data)
                .map_err(|e| StorageError::Crypto(e.to_string())),
            32 => ecb::Decryptor::<aes::Aes256>::new_from_slice(key)
                .map_err(|e| StorageError::Crypto(e.to_string()))?
                .decrypt_padded_vec_mut::<Pkcs7>(data)
                .map_err(|e| StorageError::Crypto(e.to_string())),
            len => Err(StorageError::InvalidArgument(format!(
                "Unsupported AES-ECB key length: {len} bytes"
            ))),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sha256_and_aes_roundtrip() {
        assert_eq!(
            sha256_hex("1234"),
            "03ac674216f3e15c761ee1a5e255f067953623c8b388b4459e13f978d7c846f4"
        );
        let key = [7u8; 32];
        let iv = [3u8; 12];
        let pt = b"nuvio-secret-payload";
        let ct = aes_encrypt(AesMode::Gcm, &key, &iv, pt).unwrap();
        let dec = aes_decrypt(AesMode::Gcm, &key, &iv, &ct).unwrap();
        assert_eq!(dec, pt);

        let iv_cbc = [5u8; 16];
        let ct_cbc = aes_encrypt(AesMode::Cbc, &key, &iv_cbc, pt).unwrap();
        let dec_cbc = aes_decrypt(AesMode::Cbc, &key, &iv_cbc, &ct_cbc).unwrap();
        assert_eq!(dec_cbc, pt);
    }

    #[test]
    fn pbkdf2_matches_expected_length() {
        let out = pbkdf2_derive(b"pass", b"salt", 1000, 256, DigestAlgorithm::Sha256).unwrap();
        assert_eq!(out.len(), 32);
    }
}
