use thiserror::Error;

#[derive(Error, Debug)]
pub enum CaipError {
    #[error("Invalid CAIP-2 format: {0}")]
    InvalidChainId(String),

    #[error("Invalid CAIP-10 format: {0}")]
    InvalidAccountId(String),

    #[error("Invalid CAIP-19 format: {0}")]
    InvalidAssetId(String),

    #[error("Unknown chain namespace: {0}")]
    UnknownChainNamespace(String),

    #[error("Unknown asset namespace: {0}")]
    UnknownAssetNamespace(String),

    #[error("Invalid TokenPair string format: {0}")]
    InvalidTokenPairString(String),

    #[error("Decimal overflow: max {max}, got {got}")]
    DecimalOverflow { max: u8, got: u8 },

    #[error("Invalid amount: {0}")]
    InvalidAmount(String),
}

pub type Result<T> = std::result::Result<T, CaipError>;
