use candid::CandidType;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

use crate::chain_id::ChainId;
use crate::error::{CaipError, Result};
use crate::validation::{ASSET_ID_BASE_REGEX, ASSET_ID_REGEX};

/// CAIP-19 Asset ID
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, CandidType)]
pub struct AssetId {
    chain_id: ChainId,
    asset_namespace: String,
    asset_reference: String,
}

impl AssetId {
    /// let chain_id = ChainId::new("eip155", "1").unwrap();
    /// let usdc = AssetId::new(chain_id, "erc20", "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48").unwrap();
    pub fn new(
        chain_id: ChainId,
        asset_namespace: impl Into<String>,
        asset_reference: impl Into<String>,
    ) -> Result<Self> {
        let asset_id = Self {
            chain_id,
            asset_namespace: asset_namespace.into(),
            asset_reference: asset_reference.into(),
        };
        asset_id.validate()?;
        Ok(asset_id)
    }

    pub fn chain_id(&self) -> &ChainId {
        &self.chain_id
    }

    pub fn asset_namespace(&self) -> &str {
        &self.asset_namespace
    }

    pub fn asset_reference(&self) -> &str {
        &self.asset_reference
    }

    fn validate(&self) -> Result<()> {
        let formatted = self.to_string();
        if !ASSET_ID_REGEX.is_match(&formatted) {
            return Err(CaipError::InvalidAssetId(formatted));
        }
        Ok(())
    }
}

impl fmt::Display for AssetId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}/{}:{}",
            self.chain_id, self.asset_namespace, self.asset_reference
        )
    }
}

impl FromStr for AssetId {
    type Err = CaipError;

    fn from_str(s: &str) -> Result<Self> {
        let captures = ASSET_ID_REGEX
            .captures(s)
            .ok_or_else(|| CaipError::InvalidAssetId(s.to_string()))?;

        let chain_id = ChainId::new(&captures[1], &captures[2])?;

        Ok(AssetId {
            chain_id,
            asset_namespace: captures[3].to_string(),
            asset_reference: captures[4].to_string(),
        })
    }
}

/// CAIP-19 Asset ID with base namespace
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AssetIdBase {
    asset_namespace: String,
    asset_reference: String,
}

impl AssetIdBase {
    /// let eth = AssetIdBase::new("slip44", "60").unwrap();
    /// let usdc = AssetIdBase::new("erc20", "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48").unwrap();
    pub fn new(
        asset_namespace: impl Into<String>,
        asset_reference: impl Into<String>,
    ) -> Result<Self> {
        let asset_id = Self {
            asset_namespace: asset_namespace.into(),
            asset_reference: asset_reference.into(),
        };
        asset_id.validate()?;
        Ok(asset_id)
    }

    pub fn asset_namespace(&self) -> &str {
        &self.asset_namespace
    }

    pub fn asset_reference(&self) -> &str {
        &self.asset_reference
    }

    fn validate(&self) -> Result<()> {
        let formatted = self.to_string();
        if !ASSET_ID_BASE_REGEX.is_match(&formatted) {
            return Err(CaipError::InvalidAssetId(formatted));
        }
        Ok(())
    }
}

impl fmt::Display for AssetIdBase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.asset_namespace, self.asset_reference)
    }
}

impl FromStr for AssetIdBase {
    type Err = CaipError;

    fn from_str(s: &str) -> Result<Self> {
        let captures = ASSET_ID_BASE_REGEX
            .captures(s)
            .ok_or_else(|| CaipError::InvalidAssetId(s.to_string()))?;

        Ok(AssetIdBase {
            asset_namespace: captures[1].to_string(),
            asset_reference: captures[2].to_string(),
        })
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::CaipError;
    use std::str::FromStr;

    #[test]
    fn test_asset_id_valid() {
        // Create valid asset IDs
        let chain_id = ChainId::new("eip155", "1").unwrap();

        let erc20_asset = AssetId::new(
            chain_id.clone(),
            "erc20",
            "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48", // USDC on Ethereum
        );
        assert!(erc20_asset.is_ok());

        let eth_asset = AssetId::new(
            chain_id, "slip44", "60", // ETH
        );
        assert!(eth_asset.is_ok());

        // Test getters
        let asset = eth_asset.unwrap();
        assert_eq!(asset.asset_namespace(), "slip44");
        assert_eq!(asset.asset_reference(), "60");
        assert_eq!(asset.chain_id().to_string(), "eip155:1");

        // Test to_string
        assert_eq!(asset.to_string(), "eip155:1/slip44:60");
    }

    #[test]
    fn test_asset_id_invalid() {
        let chain_id = ChainId::new("eip155", "1").unwrap();

        // Invalid asset namespace (too short)
        assert!(AssetId::new(chain_id.clone(), "e", "token").is_err());

        // Invalid asset namespace (too long)
        assert!(AssetId::new(chain_id.clone(), "toolongnamespace", "token").is_err());

        // Invalid asset namespace (invalid characters)
        assert!(AssetId::new(chain_id.clone(), "erc!20", "token").is_err());

        // Invalid asset reference (empty)
        assert!(AssetId::new(chain_id.clone(), "erc20", "").is_err());

        // Invalid asset reference (too long)
        assert!(AssetId::new(chain_id, "erc20", "a".repeat(65)).is_err());
    }

    #[test]
    fn test_asset_id_fromstr() {
        // Valid parsing
        assert_eq!(
            AssetId::from_str("eip155:1/erc20:0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48").unwrap(),
            AssetId::new(
                ChainId::new("eip155", "1").unwrap(),
                "erc20",
                "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"
            )
            .unwrap()
        );

        // Invalid format
        assert!(matches!(
            AssetId::from_str("eip155:1/erc20"),
            Err(CaipError::InvalidAssetId(_))
        ));

        assert!(matches!(
            AssetId::from_str("eip155:1/erc20:"),
            Err(CaipError::InvalidAssetId(_))
        ));

        assert!(matches!(
            AssetId::from_str("eip155:1erc20:token"),
            Err(CaipError::InvalidAssetId(_))
        ));
    }

    #[test]
    fn test_asset_id_base_valid() {
        // Create valid asset ID base instances
        let erc20_asset = AssetIdBase::new(
            "erc20",
            "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48", // USDC
        );
        assert!(erc20_asset.is_ok());

        let slip44_asset = AssetIdBase::new(
            "slip44", "60", // ETH
        );
        assert!(slip44_asset.is_ok());

        // Test getters
        let asset = slip44_asset.unwrap();
        assert_eq!(asset.asset_namespace(), "slip44");
        assert_eq!(asset.asset_reference(), "60");

        // Test to_string
        assert_eq!(asset.to_string(), "slip44:60");
    }

    #[test]
    fn test_asset_id_base_invalid() {
        // Invalid asset namespace (too short)
        assert!(AssetIdBase::new("e", "token").is_err());

        // Invalid asset namespace (too long)
        assert!(AssetIdBase::new("toolongnamespace", "token").is_err());

        // Invalid asset namespace (invalid characters)
        assert!(AssetIdBase::new("erc!20", "token").is_err());

        // Invalid asset reference (empty)
        assert!(AssetIdBase::new("erc20", "").is_err());

        // Invalid asset reference (too long)
        assert!(AssetIdBase::new("erc20", "a".repeat(65)).is_err());
    }

    #[test]
    fn test_asset_id_base_fromstr() {
        // Valid parsing
        assert_eq!(
            AssetIdBase::from_str("erc20:0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48").unwrap(),
            AssetIdBase::new("erc20", "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48").unwrap()
        );

        assert_eq!(
            AssetIdBase::from_str("slip44:60").unwrap(),
            AssetIdBase::new("slip44", "60").unwrap()
        );

        // Invalid format
        assert!(matches!(
            AssetIdBase::from_str("erc20"),
            Err(CaipError::InvalidAssetId(_))
        ));

        assert!(matches!(
            AssetIdBase::from_str("erc20:"),
            Err(CaipError::InvalidAssetId(_))
        ));

        assert!(matches!(
            AssetIdBase::from_str(":token"),
            Err(CaipError::InvalidAssetId(_))
        ));

        // Invalid with chain portion
        assert!(matches!(
            AssetIdBase::from_str("eip155:1/erc20:token"),
            Err(CaipError::InvalidAssetId(_))
        ));
    }
}
