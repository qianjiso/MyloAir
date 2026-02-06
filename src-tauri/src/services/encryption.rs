//! 加密服务
//!
//! 实现 AES-256-CBC 加密/解密，与 Electron 版本兼容

use aes::Aes256;
use cbc::{Decryptor, Encryptor};
use cbc::cipher::{BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use rand::Rng;

type Aes256CbcEnc = Encryptor<Aes256>;
type Aes256CbcDec = Decryptor<Aes256>;

/// 加密服务
pub struct EncryptionService {
    key: [u8; 32],
}

use sha2::{Sha256, Digest};

impl EncryptionService {
    /// 从字符串密钥创建服务实例
    /// 使用 SHA-256 哈希作为 AES 密钥
    pub fn new(key: &str) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(key.as_bytes());
        let result = hasher.finalize();
        
        // Copy into array
        let mut derived_key = [0u8; 32];
        derived_key.copy_from_slice(&result);
        
        Self { key: derived_key }
    }

    /// 加密文本
    /// 返回格式：Base64(IV + 加密数据)
    pub fn encrypt(&self, plaintext: &str) -> Result<String, String> {
        if plaintext.is_empty() {
            return Ok(String::new());
        }

        // 生成随机 IV
        let mut iv = [0u8; 16];
        rand::thread_rng().fill(&mut iv);

        // PKCS7 填充
        let plaintext_bytes = plaintext.as_bytes();
        let block_size = 16;
        let padding_len = block_size - (plaintext_bytes.len() % block_size);
        let mut padded = plaintext_bytes.to_vec();
        padded.extend(std::iter::repeat(padding_len as u8).take(padding_len));

        // 加密
        let cipher = Aes256CbcEnc::new(&self.key.into(), &iv.into());
        let mut buffer = padded;
        let buffer_len = buffer.len();
        cipher.encrypt_padded_mut::<aes::cipher::block_padding::NoPadding>(&mut buffer, buffer_len)
            .map_err(|e| format!("加密失败: {:?}", e))?;

        // IV + 密文
        let mut result = iv.to_vec();
        result.extend(&buffer);

        Ok(BASE64.encode(&result))
    }

    /// 解密文本
    pub fn decrypt(&self, ciphertext: &str) -> Result<String, String> {
        if ciphertext.is_empty() {
            return Ok(String::new());
        }

        // Base64 解码
        let data = BASE64.decode(ciphertext)
            .map_err(|e| format!("Base64 解码失败: {}", e))?;

        if data.len() < 17 {
            return Err("密文太短".to_string());
        }

        // 提取 IV 和密文
        let iv: [u8; 16] = data[..16].try_into()
            .map_err(|_| "IV 长度错误")?;
        let encrypted = &data[16..];

        // 解密
        let cipher = Aes256CbcDec::new(&self.key.into(), &iv.into());
        let mut buffer = encrypted.to_vec();
        cipher.decrypt_padded_mut::<aes::cipher::block_padding::NoPadding>(&mut buffer)
            .map_err(|e| format!("解密失败: {:?}", e))?;

        // 移除 PKCS7 填充
        let padding_len = *buffer.last().ok_or("空数据")? as usize;
        if padding_len > 16 || padding_len > buffer.len() {
            return Err("填充无效".to_string());
        }
        buffer.truncate(buffer.len() - padding_len);

        String::from_utf8(buffer)
            .map_err(|e| format!("UTF-8 解码失败: {}", e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt() {
        let service = EncryptionService::new("test_key_12345");
        let plaintext = "Hello, 世界!";
        
        let encrypted = service.encrypt(plaintext).unwrap();
        assert!(!encrypted.is_empty());
        assert_ne!(encrypted, plaintext);
        
        let decrypted = service.decrypt(&encrypted).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_empty_string() {
        let service = EncryptionService::new("test_key");
        
        let encrypted = service.encrypt("").unwrap();
        assert_eq!(encrypted, "");
        
        let decrypted = service.decrypt("").unwrap();
        assert_eq!(decrypted, "");
    }
}
