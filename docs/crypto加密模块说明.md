src/crypto/
├── mod.rs        // 模块声明文件
├── address.rs    // 地址生成及相关操作
├── hash.rs       // 哈希计算相关函数，SHA-256
└── signature.rs  // 签名和验证相关功能

mod.rs:
    声明 crypto 模块。
    使用 pub mod 导出子模块，例如 pub mod address;、pub mod hash; 和 pub mod signature; (如果实现了签名模块)。
address.rs:
    定义 Address 结构体，用于表示钱包地址。
    实现 Address 的相关方法，例如：
        new_address(): 生成一个新的地址。
        to_string(): 将地址转换为字符串表示。
        from_string(): 从字符串表示创建地址。
    地址应当符合BIP32、BIP39、BIP44、BIP47、BIP84标准。
hash.rs:
    提供哈希计算相关的函数。
    例如 sha256(data: &[u8]) -> Vec<u8> 函数，用于计算给定数据的 SHA-256 哈希值。
    可以根据需要添加其他哈希算法的函数。
signature.rs :
    定义签名和公钥的数据结构。
    实现签名生成和验证的相关函数。
例如 sign(data: &[u8], private_key: &PrivateKey) -> Signature 和 verify(data: &[u8], signature: &Signature, public_key: &PublicKey) -> bool。