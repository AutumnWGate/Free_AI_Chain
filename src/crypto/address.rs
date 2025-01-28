use serde::{Deserialize, Serialize};
use bip39::{Mnemonic, Language};
use bip32::{DerivationPath, XPrv, Seed};
use std::str::FromStr;
use argon2::{
    password_hash::{
        rand_core::OsRng,
        SaltString
    },
    Argon2,
};
use bech32::{
    self, 
    segwit::{self,}, // 用于 SegWit 地址编码
    Hrp, // 用于人类可读前缀
};
use bech32::primitives::gf32::Fe32;   // 用于字节到 Fe32 的转换
use bitcoin_hashes::{sha256, ripemd160};
use hex;

// FAIC 的币种 ID 1010101010 ，硬派生标识 0xBC34EB12
const FAIC_COIN_TYPE: u32 = 0xBC34EB12;

// BIP47 通知代码的版本号
const BIP47_VERSION: u8 = 0x01;

// BIP47 通知的代码类型
const BIP47_NOTIFICATION_TYPE: u8 = 0x01;

// SegWit 地址的版本号
const SEGWIT_VERSION: u8 = 0x00;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Address(Vec<u8>);

impl Address {
    // 使用助记词生成地址 (BIP44)
    pub fn new_from_mnemonic(mnemonic: &str) -> Result<Self, &'static str> {
        // 解析助记词
        let mnemonic = Mnemonic::parse_in(Language::English, mnemonic)
            .map_err(|_| "Invalid mnemonic")?;
            
        // 生成种子，使用空密码
        let seed = Seed::new(mnemonic.to_seed(""));
        
        // 从种子生成主私钥
        let root_key = XPrv::new(seed)
            .map_err(|_| "Failed to derive root key")?;

        // BIP44 路径: m/44'/FAIC_COIN_TYPE'/0'/0/0
        let path = DerivationPath::from_str(&format!("m/44'/{}'/0'/0/0", FAIC_COIN_TYPE))
            .map_err(|_| "Invalid derivation path")?;
            
        let child_key = path.into_iter().try_fold(root_key, |key, child_number| {
            key.derive_child(child_number)
        }).map_err(|_| "Failed to derive child key")?;

        let public_key = child_key.public_key();
        
        // 计算 RIPEMD160(SHA256(public_key))
        let public_key_bytes = public_key.to_bytes();
        let sha256_result = sha256::Hash::hash(&public_key_bytes);
        let ripemd160_result = ripemd160::Hash::hash(&sha256_result.as_ref());

        Ok(Address(ripemd160_result.to_byte_array().to_vec()))
    }

    // 使用密码生成助记词 (使用 Argon2 作为 KDF)
    pub fn generate_mnemonic_from_password(password: &str) -> Result<String, &'static str> {
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
                &mut output_key_material
            )
            .map_err(|_| "Failed to hash password")?;
    
        // 使用哈希值作为熵生成助记词
        let mnemonic = Mnemonic::from_entropy(&output_key_material)
            .map_err(|_| "Failed to generate mnemonic")?;
    
        Ok(mnemonic.to_string())
    }

    // 生成 BIP47 支付代码
    pub fn generate_payment_code(mnemonic: &str) -> Result<String, &'static str> {
        let mnemonic = Mnemonic::parse_in(Language::English, mnemonic)
            .map_err(|_| "Invalid mnemonic")?;
            
        let seed = Seed::new(mnemonic.to_seed(""));
        let root_xprv = XPrv::new(seed)
            .map_err(|_| "Failed to derive root key")?;
    
        let path = DerivationPath::from_str(&format!("m/47'/{}'/0'", FAIC_COIN_TYPE))
            .map_err(|_| "Invalid derivation path")?;
            
        let payment_code_xprv = path.into_iter().try_fold(root_xprv, |key, child_number| {
            key.derive_child(child_number)
        }).map_err(|_| "Failed to derive payment code key")?;
    
        let payment_code_xpub = payment_code_xprv.public_key();
        
        // 创建一个可变的 Vec 来存储支付码
        let mut payment_code_bytes = Vec::with_capacity(35);  // 33 字节公钥 + 2 字节版本和类型
        payment_code_bytes.push(BIP47_VERSION);
        payment_code_bytes.push(BIP47_NOTIFICATION_TYPE);
        payment_code_bytes.extend_from_slice(&payment_code_xpub.to_bytes());
    
        let hrp = Hrp::parse("pc").map_err(|_| "Invalid HRP")?;
        // 直接传入 Vec<u8> 的引用
        bech32::encode::<bech32::Bech32>(hrp, &payment_code_bytes)
            .map_err(|_| "Failed to encode payment code")
    }

    // 生成 SegWit 地址 (BIP84)
    // 修改 NewSegwitAddress 函数
    pub fn new_segwit_address(mnemonic: &str) -> Result<Self, &'static str> {
        // 验证助记词不为空
        if mnemonic.trim().is_empty() {
            return Err("Mnemonic cannot be empty");
        }
        
        // 验证助记词有效性
        let mnemonic = Mnemonic::parse_in(Language::English, mnemonic)
            .map_err(|_| "Invalid mnemonic")?;

        // 生成种子
        let seed = Seed::new(mnemonic.to_seed(""));
        let root_xprv = XPrv::new(seed)
            .map_err(|_| "Failed to derive root key")?;
    
        // BIP84 路径: m/84'/FAIC_COIN_TYPE'/0'/0/0
        let path = DerivationPath::from_str(&format!("m/84'/{}'/0'/0/0", FAIC_COIN_TYPE))
            .map_err(|_| "Invalid derivation path")?;
            
        // 使用 try_fold 遍历派生路径
        let child_xprv = path.into_iter().try_fold(root_xprv, |key, child_number| {
            key.derive_child(child_number)
        }).map_err(|_| "Failed to derive child key")?;
    
        let public_key = child_xprv.public_key();
    
        // 计算 RIPEMD160(SHA256(public_key))
        let public_key_bytes = public_key.to_bytes();
        let sha256_hash = sha256::Hash::hash(&public_key_bytes);
        let hash = ripemd160::Hash::hash(sha256_hash.as_ref());
    
        // 使用 segwit 模块的函数进行编码
        let hrp = Hrp::parse("faic").map_err(|_| "Invalid HRP")?;
        let address = segwit::encode(hrp, Fe32::Q, hash.as_ref()) 
            .map_err(|_| "Failed to encode address")?;
    
        Ok(Address(address.as_bytes().to_vec()))
    }

    // 将地址转换为十六进制字符串
    pub fn to_string(&self) -> String {
        hex::encode(&self.0)
    }

    // 从十六进制字符串创建地址
    pub fn from_string(encoded: &str) -> Result<Self, &'static str> {
        let decoded = hex::decode(encoded).map_err(|_| "Invalid hex string")?;
        Ok(Address(decoded))
    }
}

impl AsRef<[u8]> for Address {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}