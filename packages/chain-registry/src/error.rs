use caip::CaipError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ChainRegistryError {
    #[error("CAIP error: {0}")]
    CaipError(#[from] CaipError),

    #[error("Chain not found: {0}")]
    ChainNotFound(String),

    #[error("Asset not found: {0}")]
    AssetNotFound(String),

    #[error("Token pair not found: {0}")]
    TokenPairNotFound(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("I/O error: {0}")]
    IoError(String),

    #[error("Token pair already exists: {0}")]
    DuplicateTokenPair(String),

    #[error("Chain already exists: {0}")]
    DuplicateChain(String),

    #[error("Asset already exists: {0}")]
    DuplicateAsset(String),

    #[error("Invalid trading route: {0}")]
    InvalidTradingRoute(String),

    #[error("Circular dependency detected: {0}")]
    CircularDependency(String),
}

pub type Result<T> = std::result::Result<T, ChainRegistryError>;
