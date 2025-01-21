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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignatureWrapper(RecoverableSignature);

// 手动实现 Serialize
impl Serialize for SignatureWrapper {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let bytes = self.to_bytes();
        serializer.serialize_bytes(&bytes)
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

impl SignatureWrapper {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, &'static str> {
        if bytes.len() != 65 {
            return Err("Invalid signature length");
        }
        // 使用 try_from 来从 i32 创建 RecoveryId
        let recovery_id = RecoveryId::try_from(bytes[64] as i32)
            .map_err(|_| "Invalid recovery ID")?;
        let signature = RecoverableSignature::from_compact(&bytes[..64], recovery_id)
            .map_err(|_| "Invalid signature")?;
        Ok(SignatureWrapper(signature))
    }

    pub fn to_bytes(&self) -> [u8; 65] {
        let (recovery_id, signature_bytes) = self.0.serialize_compact();
        let mut bytes = [0u8; 65];
        bytes[..64].copy_from_slice(&signature_bytes);
        // 使用 Into<i32> trait
        bytes[64] = i32::from(recovery_id) as u8;
        bytes
    }
}

// 签名函数
pub fn sign(data: &[u8], private_key: &SecretKey) -> SignatureWrapper {
    let secp = Secp256k1::new();
    // 使用 .into() 将 GenericArray 转换为 [u8; 32]
    let message = Message::from_digest(Sha256::digest(data).into());
    let signature = secp.sign_ecdsa_recoverable(&message, private_key);
    SignatureWrapper(signature)
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
    let standard_signature = signature.0.to_standard();
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