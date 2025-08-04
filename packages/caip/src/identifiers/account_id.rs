use candid::CandidType;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

use crate::error::{CaipError, Result};
use crate::validation::ACCOUNT_ID_REGEX;
use crate::ChainId;

/// CAIP-10 Account ID
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, CandidType)]
pub struct AccountId {
    chain_id: ChainId,
    account_address: String,
}

impl AccountId {
    /// let chain_id = ChainId::new("eip155", "1").unwrap();
    /// let account = AccountId::new(chain_id, "0xab16a96d359ec26a11e2c2b3d8f8b8942d5bfcdb").unwrap();
    pub fn new(chain_id: ChainId, account_address: impl Into<String>) -> Result<Self> {
        let account_id = Self {
            chain_id,
            account_address: account_address.into(),
        };
        account_id.validate()?;
        Ok(account_id)
    }

    pub fn chain_id(&self) -> &ChainId {
        &self.chain_id
    }

    pub fn account_address(&self) -> &str {
        &self.account_address
    }

    fn validate(&self) -> Result<()> {
        let formatted = self.to_string();
        if !ACCOUNT_ID_REGEX.is_match(&formatted) {
            return Err(CaipError::InvalidAccountId(formatted));
        }
        Ok(())
    }
}

impl fmt::Display for AccountId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.chain_id, self.account_address)
    }
}

impl FromStr for AccountId {
    type Err = CaipError;

    fn from_str(s: &str) -> Result<Self> {
        let captures = ACCOUNT_ID_REGEX
            .captures(s)
            .ok_or_else(|| CaipError::InvalidAccountId(s.to_string()))?;

        let chain_id = ChainId::new(&captures[1], &captures[2])?;

        Ok(AccountId {
            chain_id,
            account_address: captures[3].to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::CaipError;
    use std::str::FromStr;

    #[test]
    fn test_account_id_valid() {
        // Create valid account IDs
        let chain_id = ChainId::new("eip155", "1").unwrap();

        let ethereum_account = AccountId::new(
            chain_id.clone(),
            "0xab16a96d359ec26a11e2c2b3d8f8b8942d5bfcdb",
        );
        assert!(ethereum_account.is_ok());

        let solana_account = AccountId::new(
            ChainId::new("solana", "4sGjMW1sUnHzSxGspuhpqLDx6wiyjNtZ").unwrap(),
            "2q7pyhPwAwZ3QMfZrnAbDhnh9mDUqycszcpf8VDWZRQv",
        );
        assert!(solana_account.is_ok());

        // Test getters
        let account = ethereum_account.unwrap();
        assert_eq!(
            account.account_address(),
            "0xab16a96d359ec26a11e2c2b3d8f8b8942d5bfcdb"
        );
        assert_eq!(account.chain_id().to_string(), "eip155:1");

        // Test to_string
        assert_eq!(
            account.to_string(),
            "eip155:1:0xab16a96d359ec26a11e2c2b3d8f8b8942d5bfcdb"
        );
    }

    #[test]
    fn test_account_id_invalid() {
        let chain_id = ChainId::new("eip155", "1").unwrap();

        // Invalid account address (empty)
        assert!(AccountId::new(chain_id.clone(), "").is_err());

        // Invalid account address (invalid characters for the protocol)
        assert!(AccountId::new(chain_id.clone(), "not-a-valid-eth-address!").is_err());

        // Invalid account address (too long)
        assert!(AccountId::new(chain_id, "a".repeat(129)).is_err());
    }

    #[test]
    fn test_account_id_fromstr() {
        // Valid parsing
        assert_eq!(
            AccountId::from_str("eip155:1:0xab16a96d359ec26a11e2c2b3d8f8b8942d5bfcdb").unwrap(),
            AccountId::new(
                ChainId::new("eip155", "1").unwrap(),
                "0xab16a96d359ec26a11e2c2b3d8f8b8942d5bfcdb"
            )
            .unwrap()
        );

        // Invalid format
        assert!(matches!(
            AccountId::from_str("eip155:1"),
            Err(CaipError::InvalidAccountId(_))
        ));

        assert!(matches!(
            AccountId::from_str("eip155:"),
            Err(CaipError::InvalidAccountId(_))
        ));

        assert!(matches!(
            AccountId::from_str("eip155:1:"),
            Err(CaipError::InvalidAccountId(_))
        ));

        assert!(matches!(
            AccountId::from_str(":1:0xab16a96d359ec26a11e2c2b3d8f8b8942d5bfcdb"),
            Err(CaipError::InvalidAccountId(_))
        ));
    }
}
