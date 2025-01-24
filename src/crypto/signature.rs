use secp256k1::{
    ecdsa::{RecoverableSignature, RecoveryId},
    rand::{rngs::OsRng, thread_rng},
    Error as SecpError, Message, PublicKey, Secp256k1, SecretKey,
};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use sha2::{Digest, Sha256};

// 为外部类型实现序列化和反序列化
pub trait Keypair: Sized {
    fn from_bytes(bytes: &[u8]) -> Result<Self, SecpError>;
    fn to_bytes(&self) -> Vec<u8>;
}

impl Keypair for SecretKey {
    fn from_bytes(bytes: &[u8]) -> Result<Self, SecpError> {
        SecretKey::from_slice(bytes)
    }

    fn to_bytes(&self) -> Vec<u8> {
        self.secret_bytes().to_vec()
    }
}

impl Keypair for PublicKey {
    fn from_bytes(bytes: &[u8]) -> Result<Self, SecpError> {
        PublicKey::from_slice(bytes)
    }

    fn to_bytes(&self) -> Vec<u8> {
        self.serialize().to_vec()
    }
}



// 手动实现 Serialize
impl Serialize for SignatureWrapper {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let bytes: &[u8] = &self.bytes;  // 将 &[u8; 65] 转换为 &[u8]
        serializer.serialize_bytes(bytes)
    }
}

// 手动实现 Deserialize
impl<'de> Deserialize<'de> for SignatureWrapper {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let bytes: Vec<u8> = <Vec<u8> as Deserialize>::deserialize(deserializer)?;
        SignatureWrapper::from_bytes(&bytes).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignatureWrapper {
    signature: RecoverableSignature,
    bytes: [u8; 65],  // 缓存序列化后的字节
}

impl SignatureWrapper {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, &'static str> {
        if bytes.len() != 65 {
            return Err("Invalid signature length");
        }
        let recovery_id = RecoveryId::try_from(bytes[64] as i32)
            .map_err(|_| "Invalid recovery ID")?;
        let signature = RecoverableSignature::from_compact(&bytes[..64], recovery_id)
            .map_err(|_| "Invalid signature")?;
        
        let mut cached_bytes = [0u8; 65];
        cached_bytes.copy_from_slice(bytes);
        
        Ok(SignatureWrapper { 
            signature,
            bytes: cached_bytes,
        })
    }

    pub fn to_bytes(&self) -> &[u8; 65] {
        &self.bytes
    }

    // 添加用于数据库操作的辅助方法
    pub fn to_hex_string(&self) -> String {
        hex::encode(self.to_bytes())
    }

    pub fn from_hex_string(hex_str: &str) -> Result<Self, &'static str> {
        let bytes = hex::decode(hex_str).map_err(|_| "Invalid hex string")?;
        Self::from_bytes(&bytes)
    }

    /// 验证签名
    /// 
    /// # 参数
    /// * `message` - 待验证的消息字节
    /// * `address` - 发送方地址
    /// 
    /// # 返回值
    /// * `Result<bool, String>` - 验证结果，Ok(true) 表示验证通过
    pub fn verify(&self, message: &[u8], address: &str) -> Result<bool, String> {
        // 1. 计算消息哈希
        let message_hash = sha2::Sha256::digest(message);
        let message = secp256k1::Message::from_digest(message_hash.into());

        // 2. 从签名恢复公钥
        let secp = secp256k1::Secp256k1::new();
        let public_key = secp.recover_ecdsa(&message, &self.signature)
            .map_err(|e| format!("恢复公钥失败: {}", e))?;

        // 3. 从公钥生成地址
        let public_key_hash = sha2::Sha256::digest(&public_key.serialize());
        let recovered_address = hex::encode(&public_key_hash);

        // 4. 验证地址匹配
        if recovered_address != address {
            return Ok(false);
        }

        // 5. 验证签名
        let standard_signature = self.signature.to_standard();
        Ok(secp.verify_ecdsa(&message, &standard_signature, &public_key).is_ok())
    }


}

// 签名函数
pub fn sign(data: &[u8], private_key: &SecretKey) -> SignatureWrapper {
    let secp = Secp256k1::new();
    // 使用 .into() 将 GenericArray 转换为 [u8; 32]
    let message = Message::from_digest(Sha256::digest(data).into());
    let signature = secp.sign_ecdsa_recoverable(&message, private_key);
    SignatureWrapper { signature, bytes: [0u8; 65] }
}

// 验证函数
pub fn verify(
    data: &[u8],
    signature: &SignatureWrapper,
    public_key: &PublicKey,
) -> bool {
    let secp = Secp256k1::new();
    // 使用 .into() 将 GenericArray 转换为 [u8; 32]
    let message = Message::from_digest(Sha256::digest(data).into());
    // 使用 to_standard() 将 RecoverableSignature 转换为 Signature
    let standard_signature = signature.signature.to_standard();
    secp.verify_ecdsa(&message, &standard_signature, public_key)
        .is_ok()
}

// 使用 OsRng 生成密钥对
pub fn generate_keypair_with_osrng() -> (SecretKey, PublicKey) {
    let mut rng = OsRng;
    let secp = Secp256k1::new();
    secp.generate_keypair(&mut rng)
}

// 使用 thread_rng 生成密钥对
pub fn generate_keypair() -> (SecretKey, PublicKey) {
    let mut rng = thread_rng();
    let secp = Secp256k1::new();
    secp.generate_keypair(&mut rng)
}

impl AsRef<[u8]> for SignatureWrapper {
    fn as_ref(&self) -> &[u8] {
        self.to_bytes()
    }
}