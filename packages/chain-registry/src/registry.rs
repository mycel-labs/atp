use atp_caip::{AssetId, AssetIdBase, ChainId, TokenPair};
use std::fs;
use std::path::Path;
use std::str::FromStr;

use crate::error::{ChainRegistryError, Result};
use crate::types::{AssetConfig, ChainConfig, RegistryConfig};

// Include the default configuration file at compile time
pub const DEFAULT_CONFIG: &str = include_str!("../default_config.toml");

pub struct ChainRegistry {
    config: RegistryConfig,
}

impl ChainRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            config: RegistryConfig::new(),
        }
    }

    /// Load the default configuration that comes with the library
    pub fn default() -> Result<Self> {
        let config = Self::config_from_toml(DEFAULT_CONFIG)?;
        config.validate()?;
        Ok(Self { config })
    }

    /// Create from existing config
    pub fn from_config(config: RegistryConfig) -> Result<Self> {
        config.validate()?;
        Ok(Self { config })
    }

    /// Load registry from file
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self> {
        let path_ref = path.as_ref();
        let content = fs::read_to_string(path_ref).map_err(|e| {
            ChainRegistryError::ConfigError(format!("Failed to read config: {}", e))
        })?;

        let config = if path_ref.extension().and_then(|s| s.to_str()) == Some("toml") {
            Self::config_from_toml(&content)?
        } else if path_ref.extension().and_then(|s| s.to_str()) == Some("json") {
            Self::config_from_json(&content)?
        } else {
            // Try to parse as TOML first, then JSON
            Self::config_from_toml(&content)
                .or_else(|_| Self::config_from_json(&content))
                .map_err(|_| {
                    ChainRegistryError::ConfigError(
                        "Unable to parse config as TOML or JSON".to_string(),
                    )
                })?
        };

        config.validate()?;
        Ok(Self { config })
    }

    pub fn config_from_toml(content: &str) -> Result<RegistryConfig> {
        toml::from_str(content)
            .map_err(|e| ChainRegistryError::ConfigError(format!("Failed to parse TOML: {}", e)))
    }

    pub fn config_from_json(content: &str) -> Result<RegistryConfig> {
        serde_json::from_str(content)
            .map_err(|e| ChainRegistryError::ConfigError(format!("Failed to parse JSON: {}", e)))
    }

    /// * `Result<()>` - Success or ChainRegistryError if file writing fails
    /// use chain_registry::ChainRegistry;
    /// let registry = ChainRegistry::new();
    /// registry.save_to_file("my_config.toml").unwrap();
    pub fn save_to_file(&self, path: impl AsRef<Path>) -> Result<()> {
        let path_ref = path.as_ref();
        let content = if path_ref.extension().and_then(|s| s.to_str()) == Some("toml") {
            self.to_toml()?
        } else {
            self.to_json()?
        };

        fs::write(path_ref, content).map_err(|e| {
            ChainRegistryError::ConfigError(format!("Failed to write config: {}", e))
        })?;

        Ok(())
    }

    /// * `Result<String>` - TOML representation of the registry, or ChainRegistryError if serialization fails
    /// use chain_registry::ChainRegistry;
    /// let registry = ChainRegistry::new();
    /// let toml_string = registry.to_toml().unwrap();
    pub fn to_toml(&self) -> Result<String> {
        toml::to_string_pretty(&self.config).map_err(|e| {
            ChainRegistryError::ConfigError(format!("Failed to serialize to TOML: {}", e))
        })
    }

    /// * `Result<String>` - JSON representation of the registry, or ChainRegistryError if serialization fails
    /// use chain_registry::ChainRegistry;
    /// let registry = ChainRegistry::new();
    /// let json_string = registry.to_json().unwrap();
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string_pretty(&self.config).map_err(|e| {
            ChainRegistryError::ConfigError(format!("Failed to serialize to JSON: {}", e))
        })
    }

    /// let registry = ChainRegistry::new();
    /// assert!(config.chains.is_empty());
    pub fn config(&self) -> &RegistryConfig {
        &self.config
    }

    /// use atp_caip::{AssetIdBase, Curve};
    /// use std::collections::HashMap;
    /// let mut registry = ChainRegistry::new();
    /// let chain_config = ChainConfig {
    ///     chain_id: "eip155:1".to_string(),
    ///     name: "Ethereum Mainnet".to_string(),
    ///     native_asset: "slip44:60".to_string(),
    ///     rpc_endpoints: vec!["https://mainnet.infura.io/v3/YOUR-PROJECT-ID".to_string()],
    ///     explorer_url: Some("https://etherscan.io".to_string()),
    ///     cryptographic_curve: vec![Curve::Secp256k1],
    ///     is_testnet: false,
    ///     assets: vec![AssetIdBase::new("slip44", "60").unwrap()],
    ///     metadata: HashMap::new(),
    /// registry.add_chain(chain_config).unwrap();
    pub fn add_chain(&mut self, config: ChainConfig) -> Result<()> {
        let chain_id = ChainId::from_str(&config.chain_id)?;
        self.config.chains.insert(chain_id.to_string(), config);
        self.config.validate()?;
        Ok(())
    }

    /// use std::str::FromStr;
    /// let mut registry = ChainRegistry::default().unwrap();
    /// let chain_id = ChainId::from_str("eip155:1").unwrap();
    /// let removed_chain = registry.remove_chain(&chain_id).unwrap();
    /// assert_eq!(removed_chain.name, "Ethereum Mainnet");
    pub fn remove_chain(&mut self, chain_id: &ChainId) -> Result<ChainConfig> {
        self.config
            .chains
            .remove(&chain_id.to_string())
            .ok_or_else(|| ChainRegistryError::ChainNotFound(chain_id.to_string()))
    }

    /// use std::str::FromStr;
    /// let registry = ChainRegistry::default().unwrap();
    /// let chain_id = ChainId::from_str("eip155:1").unwrap();
    /// let chain_config = registry.get_chain(&chain_id).unwrap();
    /// assert_eq!(chain_config.name, "Ethereum Mainnet");
    pub fn get_chain(&self, chain_id: &ChainId) -> Result<&ChainConfig> {
        self.config
            .chains
            .get(&chain_id.to_string())
            .ok_or_else(|| ChainRegistryError::ChainNotFound(chain_id.to_string()))
    }

    /// use std::str::FromStr;
    /// let mut registry = ChainRegistry::default().unwrap();
    /// let chain_id = ChainId::from_str("eip155:1").unwrap();
    /// let chain_config = registry.get_chain_mut(&chain_id).unwrap();
    /// chain_config.name = "Ethereum Mainnet Updated".to_string();
    pub fn get_chain_mut(&mut self, chain_id: &ChainId) -> Result<&mut ChainConfig> {
        self.config
            .chains
            .get_mut(&chain_id.to_string())
            .ok_or_else(|| ChainRegistryError::ChainNotFound(chain_id.to_string()))
    }

    /// let registry = ChainRegistry::default().unwrap();
    /// assert!(!chains.is_empty());
    pub fn list_chains(&self) -> Vec<&ChainConfig> {
        self.config.chains.values().collect()
    }

    /// * `Result<()>` - Success or ChainRegistryError if the asset already exists
    /// use std::collections::HashMap;
    /// let mut registry = ChainRegistry::new();
    /// let asset_config = AssetConfig {
    ///     asset_id_base: AssetIdBase::new("slip44", "60").unwrap(),
    ///     symbol: "ETH".to_string(),
    ///     name: "Ethereum".to_string(),
    ///     is_native: true,
    ///     decimals: 18,
    ///     metadata: HashMap::new(),
    /// registry.add_asset(asset_config).unwrap();
    pub fn add_asset(&mut self, config: AssetConfig) -> Result<()> {
        let asset_id_str = config.asset_id_base.to_string();
        self.config.assets.insert(asset_id_str, config);
        self.config.validate()?;
        Ok(())
    }

    /// use std::str::FromStr;
    /// let mut registry = ChainRegistry::default().unwrap();
    /// let chain_id = ChainId::from_str("eip155:1").unwrap();
    /// let asset_id = AssetId::new(chain_id, "slip44", "60").unwrap();
    /// let removed_asset = registry.remove_asset(&asset_id).unwrap();
    /// assert_eq!(removed_asset.symbol, "ETH");
    pub fn remove_asset(&mut self, asset_id: &AssetId) -> Result<AssetConfig> {
        // Create the asset base from the AssetId for lookup
        let asset_base_key = format!(
            "{}:{}",
            asset_id.asset_namespace(),
            asset_id.asset_reference()
        );
        self.config
            .assets
            .remove(&asset_base_key)
            .ok_or_else(|| ChainRegistryError::AssetNotFound(asset_id.to_string()))
    }

    /// use std::str::FromStr;
    /// let registry = ChainRegistry::default().unwrap();
    /// let chain_id = ChainId::from_str("eip155:1").unwrap();
    /// let asset_id = AssetId::new(chain_id, "slip44", "60").unwrap();
    /// let asset_config = registry.get_asset_id_base(&asset_id).unwrap();
    /// assert_eq!(asset_config.symbol, "ETH");
    pub fn get_asset_id_base(&self, asset_id: &AssetId) -> Result<&AssetConfig> {
        // Create the asset base from the AssetId for lookup
        let asset_base_key = format!(
            "{}:{}",
            asset_id.asset_namespace(),
            asset_id.asset_reference()
        );
        self.config
            .assets
            .get(&asset_base_key)
            .ok_or_else(|| ChainRegistryError::AssetNotFound(asset_id.to_string()))
    }

    /// let registry = ChainRegistry::default().unwrap();
    /// let assets = registry.list_assets();
    /// assert!(!assets.is_empty());
    pub fn list_assets(&self) -> Vec<&AssetConfig> {
        self.config.assets.values().collect()
    }

    /// * `Result<()>` - Success or ChainRegistryError if assets don't exist or pair already exists
    /// use chain_registry::ChainRegistry;
    /// use atp_caip::{AssetId, ChainId, TokenPair};
    /// use std::str::FromStr;
    /// let mut registry = ChainRegistry::default().unwrap();
    /// let eth_chain = ChainId::from_str("eip155:1").unwrap();
    /// let sol_chain = ChainId::from_str("solana:mainnet").unwrap();
    /// let eth = AssetId::new(eth_chain, "slip44", "60").unwrap();
    /// let sol = AssetId::new(sol_chain, "slip44", "501").unwrap();
    /// registry.add_token_pair(pair).unwrap();
    pub fn add_token_pair(&mut self, pair: TokenPair) -> Result<()> {
        // Validate that both assets exist using asset base keys
        let from_asset_key = format!(
            "{}:{}",
            pair.from_asset.asset_namespace(),
            pair.from_asset.asset_reference()
        );
        let to_asset_key = format!(
            "{}:{}",
            pair.to_asset.asset_namespace(),
            pair.to_asset.asset_reference()
        );

        if !self.config.assets.contains_key(&from_asset_key) {
            return Err(ChainRegistryError::AssetNotFound(
                pair.from_asset.to_string(),
            ));
        }
        if !self.config.assets.contains_key(&to_asset_key) {
            return Err(ChainRegistryError::AssetNotFound(pair.to_asset.to_string()));
        }

        // Check if pair already exists
        if self
            .find_enabled_token_pair(&pair.from_asset, &pair.to_asset)
            .is_some()
        {
            return Err(ChainRegistryError::ConfigError(format!(
                "Token pair already exists: {}",
                pair.to_pair_string()
            )));
        }

        self.config.token_pairs.push(pair);
        self.config.validate()?;
        Ok(())
    }

    /// * `Result<TokenPair>` - The removed trading pair, or ChainRegistryError if not found
    /// use chain_registry::ChainRegistry;
    /// use atp_caip::{AssetId, ChainId};
    /// use std::str::FromStr;
    /// let mut registry = ChainRegistry::default().unwrap();
    /// let eth_chain = ChainId::from_str("eip155:1").unwrap();
    /// let sol_chain = ChainId::from_str("solana:mainnet").unwrap();
    /// let eth = AssetId::new(eth_chain, "slip44", "60").unwrap();
    /// let sol = AssetId::new(sol_chain, "slip44", "501").unwrap();
    /// let removed_pair = registry.remove_token_pair(&eth, &sol).unwrap();
    pub fn remove_token_pair(
        &mut self,
        from_asset: &AssetId,
        to_asset: &AssetId,
    ) -> Result<TokenPair> {
        let index = self
            .config
            .token_pairs
            .iter()
            .position(|pair| pair.from_asset == *from_asset && pair.to_asset == *to_asset)
            .ok_or_else(|| {
                ChainRegistryError::ConfigError(format!(
                    "Token pair not found: {}-{}",
                    from_asset, to_asset
                ))
            })?;

        Ok(self.config.token_pairs.remove(index))
    }

    /// use chain_registry::ChainRegistry;
    /// use atp_caip::{AssetId, ChainId};
    /// use std::str::FromStr;
    /// let registry = ChainRegistry::default().unwrap();
    /// let eth_chain = ChainId::from_str("eip155:1").unwrap();
    /// let sol_chain = ChainId::from_str("solana:mainnet").unwrap();
    /// let eth = AssetId::new(eth_chain, "slip44", "60").unwrap();
    /// let sol = AssetId::new(sol_chain, "slip44", "501").unwrap();
    pub fn find_enabled_token_pair(
        &self,
        from_asset: &AssetId,
        to_asset: &AssetId,
    ) -> Option<&TokenPair> {
        self.config.token_pairs.iter().find(|pair| {
            pair.enabled && pair.from_asset == *from_asset && pair.to_asset == *to_asset
        })
    }

    /// use std::str::FromStr;
    /// let mut registry = ChainRegistry::default().unwrap();
    /// let eth_chain = ChainId::from_str("eip155:1").unwrap();
    /// let sol_chain = ChainId::from_str("solana:mainnet").unwrap();
    /// let eth = AssetId::new(eth_chain, "slip44", "60").unwrap();
    /// let sol = AssetId::new(sol_chain, "slip44", "501").unwrap();
    /// use chain_registry::ChainRegistry;
    /// use atp_caip::{AssetId, ChainId};
    /// use std::str::FromStr;
    /// let mut registry = ChainRegistry::default().unwrap();
    /// let eth_chain = ChainId::from_str("eip155:1").unwrap();
    /// let sol_chain = ChainId::from_str("solana:mainnet").unwrap();
    /// let eth = AssetId::new(eth_chain, "slip44", "60").unwrap();
    /// let sol = AssetId::new(sol_chain, "slip44", "501").unwrap();
    pub fn find_token_pair_mut(
        &mut self,
        from_asset: &AssetId,
        to_asset: &AssetId,
    ) -> Option<&mut TokenPair> {
        self.config
            .token_pairs
            .iter_mut()
            .find(|pair| pair.from_asset == *from_asset && pair.to_asset == *to_asset)
    }

    /// use chain_registry::ChainRegistry;
    /// let registry = ChainRegistry::default().unwrap();
    /// assert!(!pairs.is_empty());
    pub fn list_token_pairs(&self) -> Vec<&TokenPair> {
        self.config.token_pairs.iter().collect()
    }

    /// use chain_registry::ChainRegistry;
    /// let registry = ChainRegistry::default().unwrap();
    /// assert!(!enabled_pairs.is_empty());
    pub fn list_enabled_token_pairs(&self) -> Vec<&TokenPair> {
        self.config
            .token_pairs
            .iter()
            .filter(|pair| pair.enabled)
            .collect()
    }

    /// Enables or disables a trading pair.
    /// * `Result<()>` - Success or ChainRegistryError if the pair is not found
    /// use chain_registry::ChainRegistry;
    /// use atp_caip::{AssetId, ChainId};
    /// use std::str::FromStr;
    /// let mut registry = ChainRegistry::default().unwrap();
    /// let eth_chain = ChainId::from_str("eip155:1").unwrap();
    /// let sol_chain = ChainId::from_str("solana:mainnet").unwrap();
    /// let eth = AssetId::new(eth_chain, "slip44", "60").unwrap();
    /// let sol = AssetId::new(sol_chain, "slip44", "501").unwrap();
    /// registry.set_pair_enabled(&eth, &sol, false).unwrap();
    pub fn set_pair_enabled(
        &mut self,
        from_asset: &AssetId,
        to_asset: &AssetId,
        enabled: bool,
    ) -> Result<()> {
        let pair = self
            .find_token_pair_mut(from_asset, to_asset)
            .ok_or_else(|| {
                ChainRegistryError::ConfigError(format!(
                    "Token pair not found: {}-{}",
                    from_asset, to_asset
                ))
            })?;

        pair.enabled = enabled;
        self.config.validate()?;
        Ok(())
    }
}

impl Default for ChainRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// Validation implementation
impl RegistryConfig {
    /// * `Result<()>` - Success or ChainRegistryError if validation fails
    /// use chain_registry::RegistryConfig;
    /// config.validate().unwrap();
    pub fn validate(&self) -> Result<()> {
        // Validate all chain and asset IDs
        for (id, chain) in &self.chains {
            let _chain_id = ChainId::from_str(&chain.chain_id)?;
            if id != &chain.chain_id {
                return Err(ChainRegistryError::ConfigError(format!(
                    "Chain ID mismatch: key '{}' != chain_id '{}'",
                    id, chain.chain_id
                )));
            }
        }

        for (id, asset) in &self.assets {
            // Validate asset ID base
            let _asset_id_base = AssetIdBase::from_str(&asset.asset_id_base.to_string())?;
            if id != &asset.asset_id_base.to_string() {
                return Err(ChainRegistryError::ConfigError(format!(
                    "Asset ID mismatch: key '{}' != asset_id_base '{}'",
                    id, asset.asset_id_base
                )));
            }
        }

        // Validate token pairs
        for pair in &self.token_pairs {
            // Check that both assets exist in the registry using asset base keys
            let from_asset_key = format!(
                "{}:{}",
                pair.from_asset.asset_namespace(),
                pair.from_asset.asset_reference()
            );
            let to_asset_key = format!(
                "{}:{}",
                pair.to_asset.asset_namespace(),
                pair.to_asset.asset_reference()
            );

            if !self.assets.contains_key(&from_asset_key) {
                return Err(ChainRegistryError::ConfigError(format!(
                    "Token pair references unknown from_asset: {}",
                    pair.from_asset
                )));
            }
            if !self.assets.contains_key(&to_asset_key) {
                return Err(ChainRegistryError::ConfigError(format!(
                    "Token pair references unknown to_asset: {}",
                    pair.to_asset
                )));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use atp_caip::{AssetIdBase, Curve};
    use std::collections::HashMap;

    #[test]
    fn test_registry_creation() {
        let registry = ChainRegistry::new();
        assert!(registry.list_chains().is_empty());
        assert!(registry.list_assets().is_empty());
        assert!(registry.list_token_pairs().is_empty());
    }

    #[test]
    fn test_chain_crud() {
        let mut registry = ChainRegistry::new();

        let chain_config = ChainConfig {
            chain_id: "eip155:1".to_string(),
            name: "Ethereum Mainnet".to_string(),
            native_asset: "slip44:60".to_string(),
            rpc_endpoints: vec!["https://mainnet.infura.io/v3/YOUR-PROJECT-ID".to_string()],
            explorer_url: Some("https://etherscan.io".to_string()),
            cryptographic_curve: vec![Curve::Secp256k1],
            is_testnet: false,
            assets: vec![],
            metadata: HashMap::new(),
        };

        // Add chain
        registry.add_chain(chain_config.clone()).unwrap();
        assert_eq!(registry.list_chains().len(), 1);

        // Get chain
        let chain_id = ChainId::from_str("eip155:1").unwrap();
        let retrieved = registry.get_chain(&chain_id).unwrap();
        assert_eq!(retrieved.name, "Ethereum Mainnet");

        // Remove chain
        let removed = registry.remove_chain(&chain_id).unwrap();
        assert_eq!(removed.name, "Ethereum Mainnet");
        assert!(registry.list_chains().is_empty());
    }

    #[test]
    fn test_asset_crud() {
        let mut registry = ChainRegistry::new();

        let asset_config = AssetConfig {
            asset_id_base: AssetIdBase::new("slip44", "60").unwrap(),
            symbol: "ETH".to_string(),
            name: "Ethereum".to_string(),
            is_native: true,
            decimals: 18,
            metadata: HashMap::new(),
        };

        // Add asset
        registry.add_asset(asset_config.clone()).unwrap();
        assert_eq!(registry.list_assets().len(), 1);

        // Test asset retrieval by listing (since get_asset needs full AssetId)
        let assets = registry.list_assets();
        assert_eq!(assets[0].symbol, "ETH");

        // Test get_asset with proper AssetId
        let asset_id =
            AssetId::new(ChainId::from_str("eip155:1").unwrap(), "slip44", "60").unwrap();

        let retrieved_asset = registry.get_asset_id_base(&asset_id).unwrap();
        assert_eq!(retrieved_asset.symbol, "ETH");
    }

    #[test]
    fn test_asset_chain_integration() {
        let mut registry = ChainRegistry::new();

        // First add asset
        let asset_config = AssetConfig {
            asset_id_base: AssetIdBase::new("slip44", "60").unwrap(),
            symbol: "ETH".to_string(),
            name: "Ethereum".to_string(),
            is_native: true,
            decimals: 18,
            metadata: HashMap::new(),
        };
        registry.add_asset(asset_config).unwrap();

        // Then add chain that references the asset
        let chain_config = ChainConfig {
            chain_id: "eip155:1".to_string(),
            name: "Ethereum Mainnet".to_string(),
            native_asset: "slip44:60".to_string(),
            rpc_endpoints: vec!["https://mainnet.infura.io/v3/YOUR-PROJECT-ID".to_string()],
            explorer_url: Some("https://etherscan.io".to_string()),
            cryptographic_curve: vec![Curve::Secp256k1],
            is_testnet: false,
            assets: vec![AssetIdBase::new("slip44", "60").unwrap()],
            metadata: HashMap::new(),
        };

        // This should not fail validation
        registry.add_chain(chain_config).unwrap();
        assert_eq!(registry.list_chains().len(), 1);
        assert_eq!(registry.list_assets().len(), 1);
    }
}
