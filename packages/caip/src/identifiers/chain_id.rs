use candid::CandidType;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

use crate::error::{CaipError, Result};
use crate::validation::CHAIN_ID_REGEX;

/// CAIP-2 Chain ID
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, CandidType)]
pub struct ChainId {
    chain_namespace: String,
    chain_reference: String,
}

impl ChainId {
    /// * `Result<Self>` - A new ChainId instance if valid, or CaipError if validation fails
    /// let ethereum_mainnet = ChainId::new("eip155", "1").unwrap();
    /// let solana_mainnet = ChainId::new("solana", "mainnet").unwrap();
    pub fn new(namespace: impl Into<String>, reference: impl Into<String>) -> Result<Self> {
        let chain_id = Self {
            chain_namespace: namespace.into(),
            chain_reference: reference.into(),
        };
        chain_id.validate()?;
        Ok(chain_id)
    }

    pub fn namespace(&self) -> &str {
        &self.chain_namespace
    }

    pub fn reference(&self) -> &str {
        &self.chain_reference
    }

    /// Convert this ChainId to a wildcard version by replacing the reference with "*"
    /// 
    /// # Examples
    /// ```
    /// use atp_caip::ChainId;
    /// use std::str::FromStr;
    /// 
    /// let chain_id = ChainId::from_str("eip155:1").unwrap();
    /// let wildcard = chain_id.to_wildcard().unwrap();
    /// assert_eq!(wildcard.to_string(), "eip155:*");
    /// ```
    pub fn to_wildcard(&self) -> Result<Self> {
        Self::new(&self.chain_namespace, "*")
    }

    fn validate(&self) -> Result<()> {
        let formatted = self.to_string();
        if !CHAIN_ID_REGEX.is_match(&formatted) {
            return Err(CaipError::InvalidChainId(formatted));
        }
        Ok(())
    }
}

impl fmt::Display for ChainId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.chain_namespace, self.chain_reference)
    }
}

impl FromStr for ChainId {
    type Err = CaipError;

    /// * `Result<Self>` - A ChainId instance if parsing succeeds, or CaipError if invalid format
    /// use std::str::FromStr;
    /// let chain_id = ChainId::from_str("eip155:1").unwrap();
    /// assert_eq!(chain_id.namespace(), "eip155");
    /// assert_eq!(chain_id.reference(), "1");
    fn from_str(s: &str) -> Result<Self> {
        let captures = CHAIN_ID_REGEX
            .captures(s)
            .ok_or_else(|| CaipError::InvalidChainId(s.to_string()))?;

        Ok(ChainId {
            chain_namespace: captures[1].to_string(),
            chain_reference: captures[2].to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::CaipError;
    use std::str::FromStr;

    #[test]
    fn test_chain_id_valid() {
        // Valid chain IDs
        assert!(ChainId::new("eip155", "1").is_ok());
        assert!(ChainId::new("eip155", "*").is_ok());
        assert!(ChainId::new("solana", "mainnet").is_ok());
        assert!(ChainId::new("cosmos", "cosmoshub-4").is_ok());
        assert!(ChainId::new("polkadot", "91b171bb158e2d3848fa23a9f1c25182").is_ok());
        assert!(ChainId::new("bitcoin", "main").is_ok());
        assert!(ChainId::new("filecoin", "f").is_ok());

        // Test the getters
        let chain_id = ChainId::new("eip155", "1").unwrap();
        assert_eq!(chain_id.namespace(), "eip155");
        assert_eq!(chain_id.reference(), "1");

        // Test to_string
        assert_eq!(chain_id.to_string(), "eip155:1");
    }

    #[test]
    fn test_chain_id_invalid() {
        // Invalid namespace (too short)
        assert!(ChainId::new("e", "1").is_err());

        // Invalid namespace (too long)
        assert!(ChainId::new("toolongnamespace", "1").is_err());

        // Invalid namespace (invalid characters)
        assert!(ChainId::new("eip!155", "1").is_err());

        // Invalid reference (empty)
        assert!(ChainId::new("eip155", "").is_err());

        // Invalid reference (too long)
        assert!(ChainId::new("eip155", "a".repeat(33)).is_err());
    }

    #[test]
    fn test_chain_id_fromstr() {
        // Valid parsing
        assert_eq!(
            ChainId::from_str("eip155:1").unwrap(),
            ChainId::new("eip155", "1").unwrap()
        );
        assert_eq!(
            ChainId::from_str("eip155:*").unwrap(),
            ChainId::new("eip155", "*").unwrap()
        );

        assert_eq!(
            ChainId::from_str("solana:mainnet").unwrap(),
            ChainId::new("solana", "mainnet").unwrap()
        );

        // Invalid format
        assert!(matches!(
            ChainId::from_str("eip155:"),
            Err(CaipError::InvalidChainId(_))
        ));

        assert!(matches!(
            ChainId::from_str("eip155"),
            Err(CaipError::InvalidChainId(_))
        ));

        assert!(matches!(
            ChainId::from_str("eip155:1:extra"),
            Err(CaipError::InvalidChainId(_))
        ));
    }

    #[test]
    fn test_to_wildcard() {
        // Test converting specific chain IDs to wildcard
        let ethereum_mainnet = ChainId::from_str("eip155:1").unwrap();
        let ethereum_wildcard = ethereum_mainnet.to_wildcard().unwrap();
        assert_eq!(ethereum_wildcard.to_string(), "eip155:*");
        assert_eq!(ethereum_wildcard.namespace(), "eip155");
        assert_eq!(ethereum_wildcard.reference(), "*");

        let solana_mainnet = ChainId::from_str("solana:mainnet").unwrap();
        let solana_wildcard = solana_mainnet.to_wildcard().unwrap();
        assert_eq!(solana_wildcard.to_string(), "solana:*");
        assert_eq!(solana_wildcard.namespace(), "solana");
        assert_eq!(solana_wildcard.reference(), "*");

        // Test that already wildcard chains remain unchanged
        let already_wildcard = ChainId::from_str("eip155:*").unwrap();
        let still_wildcard = already_wildcard.to_wildcard().unwrap();
        assert_eq!(still_wildcard.to_string(), "eip155:*");
    }
}
