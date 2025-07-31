use serde::{Deserialize, Serialize};
use std::str::FromStr;

use crate::AssetId;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TokenPair {
    pub from_asset: AssetId,
    pub to_asset: AssetId,
    pub enabled: bool,
    #[serde(default)]
    pub min_trade_amount: Option<String>,
    #[serde(default)]
    pub max_trade_amount: Option<String>,
    #[serde(default)]
    pub fee_percentage: Option<f64>,
}

impl TokenPair {
    /// let eth_chain = ChainId::new("eip155", "1").unwrap();
    /// let sol_chain = ChainId::new("solana", "mainnet").unwrap();
    /// let eth = AssetId::new(eth_chain, "slip44", "60").unwrap();
    /// let sol = AssetId::new(sol_chain, "slip44", "501").unwrap();
    /// let pair = TokenPair::new(eth, sol);
    pub fn new(from_asset: AssetId, to_asset: AssetId) -> Self {
        Self {
            from_asset,
            to_asset,
            enabled: true,
            min_trade_amount: None,
            max_trade_amount: None,
            fee_percentage: None,
        }
    }

    /// let eth_chain = ChainId::new("eip155", "1").unwrap();
    /// let sol_chain = ChainId::new("solana", "mainnet").unwrap();
    /// let eth = AssetId::new(eth_chain, "slip44", "60").unwrap();
    /// let sol = AssetId::new(sol_chain, "slip44", "501").unwrap();
    /// let pair = TokenPair::new(eth.clone(), sol.clone());
    /// let reverse = pair.reverse();
    /// assert_eq!(reverse.from_asset, sol);
    /// assert_eq!(reverse.to_asset, eth);
    pub fn reverse(&self) -> Self {
        Self {
            from_asset: self.to_asset.clone(),
            to_asset: self.from_asset.clone(),
            enabled: self.enabled,
            min_trade_amount: self.min_trade_amount.clone(),
            max_trade_amount: self.max_trade_amount.clone(),
            fee_percentage: self.fee_percentage,
        }
    }

    /// Generates a string representation of the trading pair.
    /// * `String` - Formatted pair string (e.g., "eip155:1/slip44:60-solana:mainnet/slip44:501")
    /// let eth_chain = ChainId::new("eip155", "1").unwrap();
    /// let sol_chain = ChainId::new("solana", "mainnet").unwrap();
    /// let eth = AssetId::new(eth_chain, "slip44", "60").unwrap();
    /// let sol = AssetId::new(sol_chain, "slip44", "501").unwrap();
    /// let pair = TokenPair::new(eth, sol);
    /// assert_eq!(pair.to_pair_string(), "eip155:1/slip44:60-solana:mainnet/slip44:501");
    pub fn to_pair_string(&self) -> String {
        format!("{}-{}", self.from_asset, self.to_asset)
    }

    /// let pair = TokenPair::from_pair_string("eip155:1/slip44:60-solana:mainnet/slip44:501").unwrap();
    pub fn from_pair_string(pair_str: &str) -> Result<Self, crate::error::CaipError> {
        let parts: Vec<&str> = pair_str.split('-').collect();
        if parts.len() != 2 {
            return Err(crate::error::CaipError::InvalidTokenPariString(format!(
                "Invalid pair string format: {}",
                pair_str
            )));
        }

        let from_asset = AssetId::from_str(parts[0]).map_err(|e| {
            crate::error::CaipError::InvalidAssetId(format!("Invalid from_asset: {}", e))
        })?;

        let to_asset = AssetId::from_str(parts[1]).map_err(|e| {
            crate::error::CaipError::InvalidAssetId(format!("Invalid to_asset: {}", e))
        })?;

        Ok(Self::new(from_asset, to_asset))
    }

    pub fn involves_asset(&self, asset: &AssetId) -> bool {
        self.from_asset == *asset || self.to_asset == *asset
    }

    /// Checks if this is a cross-chain trading pair.
    pub fn is_cross_chain(&self) -> bool {
        self.from_asset.chain_id() != self.to_asset.chain_id()
    }

    /// * `asset` - One of the assets in the pair
    pub fn get_other_asset(&self, asset: &AssetId) -> Option<&AssetId> {
        if self.from_asset == *asset {
            Some(&self.to_asset)
        } else if self.to_asset == *asset {
            Some(&self.from_asset)
        } else {
            None
        }
    }
}

impl std::str::FromStr for TokenPair {
    type Err = crate::error::CaipError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_pair_string(s)
    }
}

impl std::fmt::Display for TokenPair {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_pair_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ChainId;
    use std::str::FromStr;

    #[test]
    fn test_token_pair_creation() {
        let from_asset =
            AssetId::new(ChainId::from_str("eip155:1").unwrap(), "slip44", "60").unwrap();
        let to_asset = AssetId::new(
            ChainId::from_str("solana:mainnet").unwrap(),
            "slip44",
            "501",
        )
        .unwrap();

        let pair = TokenPair::new(from_asset.clone(), to_asset.clone());
        assert_eq!(
            pair.to_pair_string(),
            "eip155:1/slip44:60-solana:mainnet/slip44:501"
        );
        assert!(pair.is_cross_chain());
        assert!(pair.involves_asset(&from_asset));
        assert!(pair.involves_asset(&to_asset));
    }

    #[test]
    fn test_token_pair_reverse() {
        let from_asset =
            AssetId::new(ChainId::from_str("eip155:1").unwrap(), "slip44", "60").unwrap();
        let to_asset = AssetId::new(
            ChainId::from_str("solana:mainnet").unwrap(),
            "slip44",
            "501",
        )
        .unwrap();

        let pair = TokenPair::new(from_asset.clone(), to_asset.clone());
        let reverse_pair = pair.reverse();

        assert_eq!(reverse_pair.from_asset, to_asset);
        assert_eq!(reverse_pair.to_asset, from_asset);
    }

    #[test]
    fn test_token_pair_from_string() {
        let pair_str = "eip155:1/slip44:60-solana:mainnet/slip44:501";
        let pair = TokenPair::from_str(pair_str).unwrap();

        assert_eq!(pair.from_asset.to_string(), "eip155:1/slip44:60");
        assert_eq!(pair.to_asset.to_string(), "solana:mainnet/slip44:501");
        assert_eq!(pair.to_string(), pair_str);
    }

    #[test]
    fn test_get_other_asset() {
        let from_asset =
            AssetId::new(ChainId::from_str("eip155:1").unwrap(), "slip44", "60").unwrap();
        let to_asset = AssetId::new(
            ChainId::from_str("solana:mainnet").unwrap(),
            "slip44",
            "501",
        )
        .unwrap();

        let pair = TokenPair::new(from_asset.clone(), to_asset.clone());

        assert_eq!(pair.get_other_asset(&from_asset), Some(&to_asset));
        assert_eq!(pair.get_other_asset(&to_asset), Some(&from_asset));

        let other_asset = AssetId::new(
            ChainId::from_str("eip155:1").unwrap(),
            "erc20",
            "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
        )
        .unwrap();
        assert_eq!(pair.get_other_asset(&other_asset), None);
    }
}
