use lazy_static::lazy_static;
use num_bigint::BigUint;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::str::FromStr;
use thiserror::Error;

lazy_static! {
    /// 1 FAIC = 10^8
    pub static ref ONE_FAIC: BigUint = BigUint::from(100_000_000u64);
    /// 最大数量: 2^128 - 1
    pub static ref MAX_AMOUNT: BigUint = BigUint::parse_bytes(b"340282366920938463463374607431768211455", 10)
        .expect("MAX_AMOUNT 常量初始化失败：无效的数字字符串");
}

/// Amount 数据类型

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Amount {
    value: BigUint,
}

impl Amount {
    /// 最小单位: 1 (0.00000001 FAIC), 实际精度: 8位小数。参考来源doge
    pub const DECIMALS: u64 = 8;

    /// 从 BigUint 创建 Amount
    pub fn from_biguint(value: BigUint) -> Result<Self, AmountError> {
        if value > *MAX_AMOUNT {
            return Err(AmountError::InvalidAmount);
        }
        Ok(Amount { value })
    }

    /// 从字符串创建 Amount
    pub fn from_str(value: &str) -> Result<Self, AmountError> {
        let parsed_value = BigUint::from_str(value).map_err(|_| AmountError::InvalidAmount)?;
        Self::from_biguint(parsed_value)
    }

    /// 获取 Amount 的值
    pub fn value(&self) -> &BigUint {
        &self.value
    }

    /// 获取 Amount 的值的字节表示（大端序）
    pub fn to_bytes_be(&self) -> Vec<u8> {
        self.value.to_bytes_be()
    }

    /// 将 Amount 转换为字符串，包含八位小数
    pub fn to_string(&self) -> String {
        let value_str = self.value.to_string();
        let len = value_str.len();

        if len <= Self::DECIMALS as usize {
            // 如果数值小于 1 FAIC，需要在前面补零
            format!("0.{:0>8}", value_str)
        } else {
            // 插入小数点
            let (integer_part, decimal_part) = value_str.split_at(len - Self::DECIMALS as usize);
            format!("{}.{}", integer_part, decimal_part)
        }
    }
}

// 为 Amount 实现加法
impl std::ops::Add for Amount {
    type Output = Result<Amount, AmountError>;

    fn add(self, other: Amount) -> Self::Output {
        let result = self.value + other.value;
        if result > *MAX_AMOUNT {
            return Err(AmountError::Overflow);
        }
        Ok(Amount { value: result })
    }
}

// 为 Amount 实现减法
impl std::ops::Sub for Amount {
    type Output = Result<Amount, AmountError>;

    fn sub(self, other: Amount) -> Self::Output {
        if self.value < other.value {
            return Err(AmountError::Underflow);
        }
        let result = self.value - other.value;
        Ok(Amount { value: result })
    }
}

// 为 Amount 实现乘法
impl std::ops::Mul for Amount {
    type Output = Result<Amount, AmountError>;

    fn mul(self, other: Amount) -> Self::Output {
        let result = self.value * other.value;
        if result > *MAX_AMOUNT {
            return Err(AmountError::Overflow);
        }
        Ok(Amount { value: result })
    }
}

// 为 Amount 实现除法
impl std::ops::Div for Amount {
    type Output = Result<Amount, AmountError>;

    fn div(self, other: Amount) -> Self::Output {
        if other.value == BigUint::from(0u64) {
            return Err(AmountError::DivisionByZero);
        }
        let result = self.value / other.value;
        Ok(Amount { value: result })
    }
}

impl sqlx::encode::Encode<'_, sqlx::Sqlite> for Amount {
    fn encode_by_ref(
        &self,
        args: &mut Vec<sqlx::sqlite::SqliteArgumentValue<'_>>,
    ) -> Result<sqlx::encode::IsNull, Box<dyn Error + Send + Sync>> {
        let str_value = self.to_string();
        <String as sqlx::encode::Encode<sqlx::Sqlite>>::encode(str_value, args)
    }
}

impl sqlx::decode::Decode<'_, sqlx::Sqlite> for Amount {
    fn decode(value: sqlx::sqlite::SqliteValueRef<'_>) -> Result<Self, sqlx::error::BoxDynError> {
        let value_str = <String as sqlx::decode::Decode<sqlx::Sqlite>>::decode(value)?;
        Amount::from_str(&value_str).map_err(|e| {
            Box::new(sqlx::Error::Decode(e.into())) as Box<dyn std::error::Error + Send + Sync>
        })
    }
}

impl sqlx::Type<sqlx::Sqlite> for Amount {
    fn type_info() -> sqlx::sqlite::SqliteTypeInfo {
        <String as sqlx::Type<sqlx::Sqlite>>::type_info()
    }
}

// 定义 AmountError 枚举
#[derive(Error, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AmountError {
    #[error("无效金额")]
    InvalidAmount,
    #[error("余额不足")]
    InsufficientBalance,
    #[error("金额溢出")]
    Overflow,
    #[error("金额下溢")]
    Underflow,
    #[error("除数为零")]
    DivisionByZero,
}

// 为 Amount 实现 Default trait
impl Default for Amount {
    fn default() -> Self {
        Amount {
            value: BigUint::from(0u64),
        }
    }
}

impl Ord for Amount {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.value.cmp(&other.value)
    }
}

impl PartialOrd for Amount {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
