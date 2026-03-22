use aes::Aes128;
use ecb::cipher::block_padding::Pkcs7;
use ecb::cipher::{BlockDecryptMut, BlockEncryptMut, KeyInit};

use crate::error::{Result, WechatError};

type Aes128EcbEnc = ecb::Encryptor<Aes128>;
type Aes128EcbDec = ecb::Decryptor<Aes128>;

pub fn encrypt_aes_ecb(plaintext: &[u8], key: &[u8; 16]) -> Vec<u8> {
    Aes128EcbEnc::new(key.into()).encrypt_padded_vec_mut::<Pkcs7>(plaintext)
}

pub fn decrypt_aes_ecb(ciphertext: &[u8], key: &[u8; 16]) -> Result<Vec<u8>> {
    Aes128EcbDec::new(key.into())
        .decrypt_padded_vec_mut::<Pkcs7>(ciphertext)
        .map_err(|e| WechatError::InvalidResponse(format!("aes decrypt failed: {e}")))
}

pub fn aes_ecb_padded_size(size: usize) -> usize {
    ((size + 1).div_ceil(16)) * 16
}
