use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use argon2::{
    password_hash::{
        rand_core::{OsRng, RngCore},
        SaltString,
    },
    Argon2,
};
use bip39::Mnemonic;

/// 加密助记词
pub fn encrypt_mnemonic(mnemonic: &str, password: &str) -> Result<Vec<u8>, &'static str> {
    // 1. 使用 Argon2 从密码派生加密密钥
    let salt = SaltString::generate(&mut OsRng);
    let mut encryption_key = [0u8; 32];

    Argon2::default()
        .hash_password_into(
            password.as_bytes(),
            salt.as_str().as_bytes(),
            &mut encryption_key,
        )
        .map_err(|_| "Failed to derive encryption key")?;

    // 2. 创建 AES-GCM 加密器
    let cipher =
        Aes256Gcm::new_from_slice(&encryption_key).map_err(|_| "Failed to create cipher")?;

    // 3. 生成随机 nonce
    let mut nonce_bytes = [0u8; 12];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    // 4. 加密助记词
    let ciphertext = cipher
        .encrypt(&nonce, mnemonic.as_bytes())
        .map_err(|_| "Encryption failed")?;

    // 5. 组合盐值、nonce 和密文
    let mut encrypted = Vec::with_capacity(salt.as_str().len() + nonce.len() + ciphertext.len());

    encrypted.extend_from_slice(salt.as_str().as_bytes());
    encrypted.extend_from_slice(&nonce);
    encrypted.extend_from_slice(&ciphertext);

    Ok(encrypted)
}

/// 解密助记词
pub fn decrypt_mnemonic(encrypted: &[u8], password: &str) -> Result<String, &'static str> {
    // 1. 提取盐值、nonce 和密文
    if encrypted.len() < 32 {
        return Err("Invalid encrypted data");
    }

    let salt = &encrypted[..16]; // Argon2 盐值长度
    let nonce = &encrypted[16..28]; // AES-GCM nonce 长度
    let ciphertext = &encrypted[28..];

    // 2. 使用 Argon2 重新派生加密密钥
    let mut encryption_key = [0u8; 32];
    Argon2::default()
        .hash_password_into(password.as_bytes(), salt, &mut encryption_key)
        .map_err(|_| "Failed to derive encryption key")?;

    // 3. 创建 AES-GCM 解密器
    let cipher =
        Aes256Gcm::new_from_slice(&encryption_key).map_err(|_| "Failed to create cipher")?;

    // 4. 解密助记词
    let plaintext = cipher
        .decrypt(nonce.into(), ciphertext)
        .map_err(|_| "Decryption failed")?;

    // 5. 转换为字符串
    String::from_utf8(plaintext).map_err(|_| "Invalid UTF-8 in decrypted data")
}

/// 使用密码生成助记词 (使用 Argon2 作为 KDF)
pub fn generate_from_password(password: &str) -> Result<String, &'static str> {
    if password.len() < 12 {
        return Err("Password must be at least 12 characters long");
    }

    // 生成随机盐值
    let salt = SaltString::generate(&mut OsRng);

    // 创建一个输出缓冲区
    let mut output_key_material = [0u8; 32];

    // 使用 Argon2id 生成哈希
    Argon2::default()
        .hash_password_into(
            password.as_bytes(),
            salt.as_str().as_bytes(),
            &mut output_key_material,
        )
        .map_err(|_| "Failed to hash password")?;

    // 使用哈希值作为熵生成助记词
    let mnemonic =
        Mnemonic::from_entropy(&output_key_material).map_err(|_| "Failed to generate mnemonic")?;

    Ok(mnemonic.to_string())
}
