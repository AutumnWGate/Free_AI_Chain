use crate::crypto::signature::{sign, verify, SignatureWrapper};
use crate::wallet::error::KeyManagerError;
use bip32::{DerivationPath, Seed, XPrv, XPub};
use bip39::{Language, Mnemonic};
use log::debug;
use secp256k1::{PublicKey, SecretKey};
use std::str::FromStr;

// FAIC 的币种 ID 1010101010 ，硬派生标识 0xBC34EB12
const FAIC_COIN_TYPE: u32 = 0xBC34EB12;
const MAX_DERIVATION_DEPTH: u8 = 5; // BIP44 标准的最大深度
const MAX_INDEX: u32 = 0x7fffffff; // 最大索引值 (非硬化)
const HARDENED_INDEX_START: u32 = 0x80000000; // 硬化索引起始值

#[derive(Debug)]
pub struct KeyManager {
    /// 扩展私钥 (xprv)，用于派生子私钥和签名交易
    ///
    /// **安全警告**:  私钥必须妥善保管，绝对不能泄露！
    master_key: XPrv,

    /// 扩展公钥 (xpub)，用于派生子公钥和生成地址 (无需私钥)
    ///
    /// 可以安全地公开扩展公钥，用于生成接收地址。
    pub xpub: XPub,
    /// 密钥派生路径 (BIP44)
    ///
    /// 记录了用于派生 `master_key` 和 `xpub` 的 BIP44 路径，方便追踪和管理密钥。
    derivation_path: DerivationPath,
}

impl KeyManager {
    /// 从助记词创建 `KeyManager` 实例
    ///
    /// # 参数
    ///
    /// * `mnemonic` - 助记词字符串
    ///
    /// # 返回值
    ///
    /// 返回 `Result`，成功时返回 `KeyManager` 实例，失败时返回错误信息
    ///
    /// # 错误类型
    ///
    /// * `&'static str` - 错误信息字符串
    pub fn from_mnemonic(mnemonic: &str, password: &str) -> Result<Self, KeyManagerError> {
        //  验证密码复杂度
        if password.len() < 8 {
            return Err(KeyManagerError::InvalidPassword(
                "密码长度必须至少为8个字符".into(),
            ));
        }

        // 检查是否包含至少一个大写字母
        if !password.chars().any(|c| c.is_ascii_uppercase()) {
            return Err(KeyManagerError::InvalidPassword(
                "密码必须包含至少一个大写字母".into(),
            ));
        }

        //  解析助记词，验证助记词的有效性
        let mnemonic_parsed = Mnemonic::parse_in(Language::English, mnemonic)
            .map_err(|_| KeyManagerError::InvalidMnemonic("无效的助记词".into()))?;

        //  直接使用原始密码。这样才符合 BIP39 规范。
        let seed = Seed::new(mnemonic_parsed.to_seed(password));

        //  从种子生成根私钥 (m)，这是 BIP32 树的根节点
        let root_key = XPrv::new(seed)
            .map_err(|_| KeyManagerError::InvalidPath("无效的派生路径格式".into()))?;

        //  定义 BIP44 派生路径: m/44'/FAIC_COIN_TYPE'/0'/0'/0
        //    - m: Master 根密钥
        //    - 44': BIP44 协议标识 (硬分叉，表示使用 BIP44 协议)
        //    - FAIC_COIN_TYPE': 币种类型 (硬分叉，每个币种有唯一的类型 ID)
        //    - 0': 账户 (硬分叉，用于隔离不同账户，通常从 0 开始)
        //    - 0: 外部链 (非硬分叉，用于接收地址，0 表示外部链，1 表示内部链/找零链)
        //    - 0: 地址索引 (非硬分叉，用于生成账户下的不同地址，从 0 开始递增)
        // 使用 create_bip44_path 创建标准路径
        let derivation_path = Self::create_bip44_path(0, false, 0)?;

        //  从根私钥派生子私钥 (也称为 master key 或 account key)
        //    -  根据 BIP44 路径，从 root_key 逐层派生
        let child_key = derivation_path
            .clone()
            .into_iter()
            .try_fold(root_key, |key, child_number| key.derive_child(child_number))
            .map_err(|_| KeyManagerError::DerivationError("派生子密钥失败".into()))?;

        //  获取子私钥对应的扩展公钥 (xpub)
        //    -  扩展公钥可以安全地用于派生子公钥和生成地址，而无需私钥
        let xpub = child_key.public_key();

        //  创建 KeyManager 实例并返回
        Ok(KeyManager {
            master_key: child_key, // 存储派生出的子私钥 (BIP44 路径的最后一层)
            xpub,
            derivation_path,
        })
    }

    /// 验证派生路径
    fn validate_derivation_path(path: &DerivationPath) -> Result<(), KeyManagerError> {
        let components: Vec<_> = path.clone().into_iter().collect();

        // 验证深度
        if components.len() != 5 {
            // BIP44 要求精确的5层
            return Err(KeyManagerError::InvalidPath("BIP44 路径必须是5层".into()));
        }

        // 验证每一层
        match components.as_slice() {
            [purpose, coin_type, account, change, index] => {
                // 44'
                if !purpose.is_hardened() || purpose.0 != 44 {
                    return Err(KeyManagerError::InvalidPath("第一层必须是 44'".into()));
                }
                // coin_type'
                if !coin_type.is_hardened() || coin_type.0 != FAIC_COIN_TYPE {
                    return Err(KeyManagerError::InvalidPath("第二层必须是币种标识'".into()));
                }
                // account'
                if !account.is_hardened() || account.0 >= HARDENED_INDEX_START {
                    return Err(KeyManagerError::InvalidPath(
                        "第三层必须是有效的硬化账户索引".into(),
                    ));
                }
                // change
                if change.is_hardened() || change.0 > 1 {
                    return Err(KeyManagerError::InvalidPath("第四层必须是 0 或 1".into()));
                }
                // address_index
                if index.is_hardened() || index.0 > MAX_INDEX {
                    return Err(KeyManagerError::InvalidPath(
                        "第五层必须是有效的地址索引".into(),
                    ));
                }
            }
            _ => unreachable!(),
        }

        Ok(())
    }

    /// 创建标准 BIP44 派生路径
    pub fn create_bip44_path(
        account: u32,
        change: bool,
        address_index: u32,
    ) -> Result<DerivationPath, KeyManagerError> {
        // 验证参数范围
        if account > MAX_INDEX {
            return Err(KeyManagerError::InvalidPath("账户索引超出范围".into()));
        }
        if address_index > MAX_INDEX {
            return Err(KeyManagerError::InvalidPath("地址索引超出范围".into()));
        }

        let path_str = format!(
            "m/44'/{}'/{}'/{}/{}",
            FAIC_COIN_TYPE,
            account,
            if change { 1 } else { 0 },
            address_index
        );

        let path = DerivationPath::from_str(&path_str)
            .map_err(|_| KeyManagerError::DerivationError("根密钥生成失败".into()))?;

        Self::validate_derivation_path(&path)?;
        Ok(path)
    }

    /// 派生指定路径的子密钥
    pub fn derive_key(&self, path: &str) -> Result<XPrv, KeyManagerError> {
        debug!("派生子密钥，路径: {}", path);

        let derivation_path = DerivationPath::from_str(path)
            .map_err(|_| KeyManagerError::InvalidPath("无效的派生路径格式".into()))?;

        // 添加路径验证
        Self::validate_derivation_path(&derivation_path)?;

        derivation_path
            .clone()
            .into_iter()
            .try_fold(self.master_key.clone(), |key, child_number| {
                key.derive_child(child_number)
            })
            .map_err(|_| KeyManagerError::DerivationError("派生子密钥失败".into()))
    }

    /// 获取指定路径的公钥
    pub fn get_public_key(&self, path: &str) -> Result<XPub, KeyManagerError> {
        debug!("获取公钥，路径: {}", path);
        let child_key = self.derive_key(path)?;
        Ok(child_key.public_key())
    }

    /// 签名消息
    pub fn sign_message(&self, message: &[u8]) -> Result<SignatureWrapper, KeyManagerError> {
        // 从 master_key 获取私钥
        let secret_key = SecretKey::from_slice(&self.master_key.to_bytes())
            .map_err(|e| KeyManagerError::SignatureError(e.to_string()))?;

        // 直接使用 signature.rs 中的 sign 函数
        Ok(sign(message, &secret_key))
    }

    /// 验证签名
    pub fn verify_signature(
        &self,
        message: &[u8],
        signature: &SignatureWrapper,
        public_key: &XPub,
    ) -> Result<bool, KeyManagerError> {
        // 转换公钥格式
        let pk = PublicKey::from_slice(&public_key.to_bytes())
            .map_err(|e| KeyManagerError::SignatureError(e.to_string()))?;

        // 直接使用 signature.rs 中的 verify 函数
        Ok(verify(message, signature, &pk))
    }
}
