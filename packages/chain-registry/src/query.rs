use atp_caip::{AssetId, AssetIdBase, ChainId, Curve, TokenPair};
use std::str::FromStr;

use crate::error::{ChainRegistryError, Result};
use crate::registry::ChainRegistry;
use crate::types::{AssetConfig, ChainConfig};

impl ChainRegistry {
    /// * `chain_name` - The human-readable name of the blockchain (e.g., "Ethereum Mainnet")
    /// use solveros_chain_registry::ChainRegistry;
    /// let registry = ChainRegistry::default().unwrap();
    /// let chain_id = registry.get_chain_id_from_config("Ethereum Mainnet").unwrap();
    /// assert_eq!(chain_id.to_string(), "eip155:1");
    pub fn get_chain_id_from_config(&self, chain_name: &str) -> Result<ChainId> {
        let chain_config = self
            .config()
            .chains
            .values()
            .find(|config| config.name == chain_name)
            .ok_or_else(|| ChainRegistryError::ChainNotFound(chain_name.to_string()))?;

        ChainId::from_str(&chain_config.chain_id)
            .map_err(|e| ChainRegistryError::ConfigError(format!("Invalid chain ID: {}", e)))
    }

    /// * `chain_id` - The CAIP-2 chain identifier string (e.g., "eip155:1")
    /// use solveros_chain_registry::ChainRegistry;
    /// let registry = ChainRegistry::default().unwrap();
    /// let asset_id = registry.get_asset_id_from_config("ETH", "eip155:1").unwrap();
    /// assert_eq!(asset_id.to_string(), "eip155:1/slip44:60");
    pub fn get_asset_id_from_config(&self, asset_symbol: &str, chain_id: &str) -> Result<AssetId> {
        // Find asset by symbol
        let asset_config = self
            .config()
            .assets
            .values()
            .find(|config| config.symbol == asset_symbol)
            .ok_or_else(|| ChainRegistryError::AssetNotFound(asset_symbol.to_string()))?;

        // Parse chain ID
        let chain_id_parsed = ChainId::from_str(chain_id)
            .map_err(|e| ChainRegistryError::ConfigError(format!("Invalid chain ID: {}", e)))?;

        // Construct full AssetId using the correct signature: new(chain_id, asset_namespace, asset_reference)
        AssetId::new(
            chain_id_parsed,
            asset_config.asset_id_base.asset_namespace(),
            asset_config.asset_id_base.asset_reference(),
        )
        .map_err(|e| ChainRegistryError::ConfigError(format!("Failed to create AssetId: {}", e)))
    }

    // ASSET DISCOVERY

    /// * `chain_id` - The CAIP-2 chain identifier
    /// use solveros_chain_registry::ChainRegistry;
    /// use solveros_caip::ChainId;
    /// use std::str::FromStr;
    /// let registry = ChainRegistry::default().unwrap();
    /// let chain_id = ChainId::from_str("eip155:1").unwrap();
    /// let assets = registry.get_chain_assets(&chain_id).unwrap();
    /// assert!(!assets.is_empty());
    pub fn get_chain_assets(&self, chain_id: &ChainId) -> Result<Vec<AssetId>> {
        let chain_config = self.get_chain(chain_id)?;
        let mut assets = Vec::new();

        for asset_base in &chain_config.assets {
            let asset_id = AssetId::new(
                chain_id.clone(),
                asset_base.asset_namespace(),
                asset_base.asset_reference(),
            )
            .map_err(|e| {
                ChainRegistryError::ConfigError(format!("Failed to create AssetId: {}", e))
            })?;
            assets.push(asset_id);
        }

        Ok(assets)
    }

    /// * `chain_id` - The CAIP-2 chain identifier
    /// use solveros_chain_registry::ChainRegistry;
    /// use solveros_caip::ChainId;
    /// use std::str::FromStr;
    /// let registry = ChainRegistry::default().unwrap();
    /// let chain_id = ChainId::from_str("eip155:1").unwrap();
    /// let native_asset = registry.get_native_asset(&chain_id).unwrap();
    /// assert_eq!(native_asset.to_string(), "eip155:1/slip44:60");
    pub fn get_native_asset(&self, chain_id: &ChainId) -> Result<AssetId> {
        let chain_config = self.get_chain(chain_id)?;
        let native_asset_base = AssetIdBase::from_str(&chain_config.native_asset)
            .map_err(|e| ChainRegistryError::ConfigError(format!("Invalid native asset: {}", e)))?;

        AssetId::new(
            chain_id.clone(),
            native_asset_base.asset_namespace(),
            native_asset_base.asset_reference(),
        )
        .map_err(|e| {
            ChainRegistryError::ConfigError(format!("Failed to create native AssetId: {}", e))
        })
    }

    /// * `chain_id` - The CAIP-2 chain identifier to search within
    /// use solveros_chain_registry::ChainRegistry;
    /// use solveros_caip::ChainId;
    /// use std::str::FromStr;
    /// let registry = ChainRegistry::default().unwrap();
    /// let chain_id = ChainId::from_str("eip155:1").unwrap();
    /// let asset_config = registry.get_asset_config_by_symbol_and_chain("ETH", &chain_id).unwrap();
    /// assert_eq!(asset_config.symbol, "ETH");
    pub fn get_asset_config_by_symbol_and_chain(
        &self,
        symbol: &str,
        chain_id: &ChainId,
    ) -> Result<&AssetConfig> {
        // First get all assets for the chain
        let chain_assets = self.get_chain_assets(chain_id)?;

        // Find the asset with matching symbol
        for asset_id in chain_assets {
            if let Ok(asset_config) = self.get_asset_id_base(&asset_id) {
                if asset_config.symbol == symbol {
                    return Ok(asset_config);
                }
            }
        }

        Err(ChainRegistryError::AssetNotFound(format!(
            "Asset '{}' not found on chain '{}'",
            symbol, chain_id
        )))
    }

    /// * `Vec<(ChainId, &AssetConfig)>` - Vector of tuples containing chain ID and asset configuration for each match
    /// use solveros_chain_registry::ChainRegistry;
    /// let registry = ChainRegistry::default().unwrap();
    /// let eth_assets = registry.find_asset_configs_by_symbol("ETH");
    /// assert!(!eth_assets.is_empty());
    pub fn find_asset_configs_by_symbol(&self, symbol: &str) -> Vec<(ChainId, &AssetConfig)> {
        let mut results = Vec::new();

        for asset_config in self.config().assets.values() {
            if asset_config.symbol == symbol {
                // Try to determine which chains have this asset
                for chain_config in self.config().chains.values() {
                    if chain_config.assets.contains(&asset_config.asset_id_base) {
                        if let Ok(chain_id) = ChainId::from_str(&chain_config.chain_id) {
                            results.push((chain_id, asset_config));
                        }
                    }
                }
            }
        }

        results
    }

    // TOKEN PAIR QUERIES

    /// * `Vec<&TokenPair>` - Vector of references to enabled trading pairs
    /// use solveros_chain_registry::ChainRegistry;
    /// let registry = ChainRegistry::default().unwrap();
    /// let supported_pairs = registry.get_supported_token_pairs();
    /// assert!(!supported_pairs.is_empty());
    pub fn get_supported_token_pairs(&self) -> Vec<&TokenPair> {
        self.config()
            .token_pairs
            .iter()
            .filter(|pair| pair.enabled)
            .collect()
    }

    /// * `Vec<&TokenPair>` - Vector of trading pairs where the asset is either source or target
    /// use solveros_chain_registry::ChainRegistry;
    /// use solveros_caip::{AssetId, ChainId};
    /// use std::str::FromStr;
    /// let registry = ChainRegistry::default().unwrap();
    /// let chain_id = ChainId::from_str("eip155:1").unwrap();
    /// let eth = AssetId::new(chain_id, "slip44", "60").unwrap();
    pub fn get_pairs_for_asset(&self, asset_id: &AssetId) -> Vec<&TokenPair> {
        self.config()
            .token_pairs
            .iter()
            .filter(|pair| pair.enabled && pair.involves_asset(asset_id))
            .collect()
    }

    /// * `Vec<&TokenPair>` - Vector of trading pairs between assets on different blockchains
    /// use solveros_chain_registry::ChainRegistry;
    /// let registry = ChainRegistry::default().unwrap();
    /// let cross_chain_pairs = registry.get_cross_chain_pairs();
    /// assert!(!cross_chain_pairs.is_empty());
    pub fn get_cross_chain_pairs(&self) -> Vec<&TokenPair> {
        self.config()
            .token_pairs
            .iter()
            .filter(|pair| pair.enabled && pair.is_cross_chain())
            .collect()
    }

    /// * `Vec<&TokenPair>` - Vector of trading pairs between assets on the same blockchain
    /// use solveros_chain_registry::ChainRegistry;
    /// let registry = ChainRegistry::default().unwrap();
    /// let same_chain_pairs = registry.get_same_chain_pairs();
    pub fn get_same_chain_pairs(&self) -> Vec<&TokenPair> {
        self.config()
            .token_pairs
            .iter()
            .filter(|pair| pair.enabled && !pair.is_cross_chain())
            .collect()
    }

    /// use solveros_chain_registry::ChainRegistry;
    /// use solveros_caip::{AssetId, ChainId};
    /// use std::str::FromStr;
    /// let registry = ChainRegistry::default().unwrap();
    /// let eth_chain = ChainId::from_str("eip155:1").unwrap();
    /// let sol_chain = ChainId::from_str("solana:mainnet").unwrap();
    /// let eth = AssetId::new(eth_chain, "slip44", "60").unwrap();
    /// let sol = AssetId::new(sol_chain, "slip44", "501").unwrap();
    pub fn is_pair_supported(&self, from_asset: &AssetId, to_asset: &AssetId) -> bool {
        self.find_token_pair(from_asset, to_asset).is_some()
    }

    /// * `Vec<Vec<&TokenPair>>` - Vector of trading routes, where each route is a sequence of token pairs
    /// use solveros_chain_registry::ChainRegistry;
    /// use std::str::FromStr;
    /// let registry = ChainRegistry::default().unwrap();
    /// let eth_chain = ChainId::from_str("eip155:1").unwrap();
    /// let sol_chain = ChainId::from_str("solana:mainnet").unwrap();
    /// let eth = AssetId::new(eth_chain, "slip44", "60").unwrap();
    /// let sol = AssetId::new(sol_chain, "slip44", "501").unwrap();
    /// * `Vec<Vec<&TokenPair>>` - Vector of trading routes, where each route is a sequence of token pairs
    /// use solveros_chain_registry::ChainRegistry;
    /// use solveros_caip::{AssetId, ChainId};
    /// use std::str::FromStr;
    /// let registry = ChainRegistry::default().unwrap();
    /// let eth_chain = ChainId::from_str("eip155:1").unwrap();
    /// let sol_chain = ChainId::from_str("solana:mainnet").unwrap();
    /// let eth = AssetId::new(eth_chain, "slip44", "60").unwrap();
    /// let sol = AssetId::new(sol_chain, "slip44", "501").unwrap();
    pub fn find_trading_routes(
        &self,
        from_asset: &AssetId,
        to_asset: &AssetId,
        max_hops: usize,
    ) -> Vec<Vec<&TokenPair>> {
        let mut routes = Vec::new();
        let mut visited = std::collections::HashSet::new();
        let enabled_pairs = self.get_supported_token_pairs();

        self.find_routes_recursive(
            from_asset,
            to_asset,
            &enabled_pairs,
            &mut Vec::new(),
            &mut routes,
            &mut visited,
            max_hops,
        );

        routes
    }

    fn find_routes_recursive<'a>(
        &self,
        current_asset: &AssetId,
        target_asset: &AssetId,
        available_pairs: &[&'a TokenPair],
        current_route: &mut Vec<&'a TokenPair>,
        routes: &mut Vec<Vec<&'a TokenPair>>,
        visited: &mut std::collections::HashSet<String>,
        remaining_hops: usize,
    ) {
        if remaining_hops == 0 {
            return;
        }

        if visited.contains(&current_asset.to_string()) {
            return;
        }

        visited.insert(current_asset.to_string());

        for pair in available_pairs {
            if let Some(next_asset) = pair.get_other_asset(current_asset) {
                current_route.push(pair);

                if next_asset == target_asset {
                    // Found a route!
                    routes.push(current_route.clone());
                } else {
                    // Continue searching
                    self.find_routes_recursive(
                        next_asset,
                        target_asset,
                        available_pairs,
                        current_route,
                        routes,
                        visited,
                        remaining_hops - 1,
                    );
                }

                current_route.pop();
            }
        }

        visited.remove(&current_asset.to_string());
    }

    // CHAIN FILTERING

    /// * `Vec<&ChainConfig>` - Vector of chain configurations that support the specified curve
    /// use solveros_chain_registry::ChainRegistry;
    /// use solveros_caip::Curve;
    /// let registry = ChainRegistry::default().unwrap();
    pub fn get_chains_by_curve(&self, curve: &Curve) -> Vec<&ChainConfig> {
        self.config()
            .chains
            .values()
            .filter(|config| config.cryptographic_curve.contains(curve))
            .collect()
    }

    /// * `Vec<&ChainConfig>` - Vector of testnet chain configurations
    /// use solveros_chain_registry::ChainRegistry;
    /// let registry = ChainRegistry::default().unwrap();
    pub fn get_testnet_chains(&self) -> Vec<&ChainConfig> {
        self.config()
            .chains
            .values()
            .filter(|config| config.is_testnet)
            .collect()
    }

    /// * `Vec<&ChainConfig>` - Vector of mainnet chain configurations
    /// use solveros_chain_registry::ChainRegistry;
    /// let registry = ChainRegistry::default().unwrap();
    pub fn get_mainnet_chains(&self) -> Vec<&ChainConfig> {
        self.config()
            .chains
            .values()
            .filter(|config| !config.is_testnet)
            .collect()
    }

    /// Find chains by name pattern
    pub fn find_chains_by_name(&self, pattern: &str) -> Vec<&ChainConfig> {
        let pattern_lower = pattern.to_lowercase();
        self.config()
            .chains
            .values()
            .filter(|config| config.name.to_lowercase().contains(&pattern_lower))
            .collect()
    }

    /// * `Vec<&ChainConfig>` - Vector of chain configurations that support the asset
    /// use solveros_chain_registry::ChainRegistry;
    /// use solveros_caip::AssetIdBase;
    /// let registry = ChainRegistry::default().unwrap();
    /// let eth_asset = AssetIdBase::new("slip44", "60").unwrap();
    /// let chains_with_eth = registry.get_chains_with_asset(&eth_asset);
    pub fn get_chains_with_asset(&self, asset_base: &AssetIdBase) -> Vec<&ChainConfig> {
        self.config()
            .chains
            .values()
            .filter(|config| config.assets.contains(asset_base))
            .collect()
    }

    // UTILITY QUERIES

    /// use solveros_chain_registry::ChainRegistry;
    /// let registry = ChainRegistry::default().unwrap();
    /// let stats = registry.get_statistics();
    pub fn get_statistics(&self) -> RegistryStatistics {
        let total_chains = self.config().chains.len();
        let total_assets = self.config().assets.len();
        let total_pairs = self.config().token_pairs.len();
        let enabled_pairs = self
            .config()
            .token_pairs
            .iter()
            .filter(|p| p.enabled)
            .count();
        let cross_chain_pairs = self
            .config()
            .token_pairs
            .iter()
            .filter(|p| p.is_cross_chain())
            .count();
        let testnet_chains = self
            .config()
            .chains
            .values()
            .filter(|c| c.is_testnet)
            .count();
        let mainnet_chains = total_chains - testnet_chains;

        RegistryStatistics {
            total_chains,
            mainnet_chains,
            testnet_chains,
            total_assets,
            total_pairs,
            enabled_pairs,
            cross_chain_pairs,
            same_chain_pairs: enabled_pairs - cross_chain_pairs,
        }
    }

    /// Generates all possible trading pairs from available assets.
    /// * `Vec<TokenPair>` - Vector of all possible trading pairs (not necessarily enabled)
    /// use solveros_chain_registry::ChainRegistry;
    /// let registry = ChainRegistry::default().unwrap();
    pub fn generate_all_possible_pairs(&self) -> Vec<TokenPair> {
        let mut all_assets = Vec::new();

        // Create AssetIds from the available assets in each chain
        for chain_config in self.config().chains.values() {
            if let Ok(chain_id) = ChainId::from_str(&chain_config.chain_id) {
                for asset_base in &chain_config.assets {
                    if let Ok(asset_id) = AssetId::new(
                        chain_id.clone(),
                        asset_base.asset_namespace(),
                        asset_base.asset_reference(),
                    ) {
                        all_assets.push(asset_id);
                    }
                }
            }
        }

        let mut pairs = Vec::new();
        for (i, from_asset) in all_assets.iter().enumerate() {
            for to_asset in all_assets.iter().skip(i + 1) {
                // Create both directions of the pair
                pairs.push(TokenPair::new(from_asset.clone(), to_asset.clone()));
                pairs.push(TokenPair::new(to_asset.clone(), from_asset.clone()));
            }
        }
        pairs
    }

    /// - Missing reverse trading pairs
    /// - Orphaned token pairs
    /// * `Vec<HealthIssue>` - Vector of health issues found in the registry
    /// use solveros_chain_registry::ChainRegistry;
    /// let registry = ChainRegistry::default().unwrap();
    /// let health_issues = registry.check_health();
    /// if health_issues.is_empty() {
    pub fn check_health(&self) -> Vec<HealthIssue> {
        let mut issues = Vec::new();

        // Check for chains without assets
        for (chain_id, chain_config) in &self.config().chains {
            if chain_config.assets.is_empty() {
                issues.push(HealthIssue::ChainWithoutAssets(chain_id.clone()));
            }

            // Check for invalid native asset
            if let Err(_) = AssetIdBase::from_str(&chain_config.native_asset) {
                issues.push(HealthIssue::InvalidNativeAsset {
                    chain_id: chain_id.clone(),
                    native_asset: chain_config.native_asset.clone(),
                });
            }
        }

        // Check for orphaned assets (not referenced by any chain)
        for (asset_id_str, _asset_config) in &self.config().assets {
            if let Ok(asset_id_base) = AssetIdBase::from_str(asset_id_str) {
                let is_referenced = self
                    .config()
                    .chains
                    .values()
                    .any(|chain| chain.assets.contains(&asset_id_base));

                if !is_referenced {
                    issues.push(HealthIssue::OrphanedAsset(asset_id_str.clone()));
                }
            }
        }

        // Check for disabled pairs that have no enabled reverse
        for pair in &self.config().token_pairs {
            if !pair.enabled {
                let reverse_enabled = self.config().token_pairs.iter().any(|p| {
                    p.enabled && p.from_asset == pair.to_asset && p.to_asset == pair.from_asset
                });

                if !reverse_enabled {
                    issues.push(HealthIssue::NoEnabledDirection {
                        from_asset: pair.from_asset.to_string(),
                        to_asset: pair.to_asset.to_string(),
                    });
                }
            }
        }

        issues
    }
}

#[derive(Debug, Clone)]
pub struct RegistryStatistics {
    pub total_chains: usize,
    pub mainnet_chains: usize,
    pub testnet_chains: usize,
    pub total_assets: usize,
    pub total_pairs: usize,
    pub enabled_pairs: usize,
    pub cross_chain_pairs: usize,
    pub same_chain_pairs: usize,
}

#[derive(Debug, Clone)]
pub enum HealthIssue {
    ChainWithoutAssets(String),
    OrphanedAsset(String),
    InvalidNativeAsset {
        chain_id: String,
        native_asset: String,
    },
    NoEnabledDirection {
        from_asset: String,
        to_asset: String,
    },
}

impl std::fmt::Display for HealthIssue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HealthIssue::ChainWithoutAssets(chain_id) => {
                write!(f, "Chain '{}' has no assets configured", chain_id)
            }
            HealthIssue::OrphanedAsset(asset_id) => {
                write!(f, "Asset '{}' is not referenced by any chain", asset_id)
            }
            HealthIssue::InvalidNativeAsset {
                chain_id,
                native_asset,
            } => {
                write!(
                    f,
                    "Chain '{}' has invalid native asset '{}'",
                    chain_id, native_asset
                )
            }
            HealthIssue::NoEnabledDirection {
                from_asset,
                to_asset,
            } => {
                write!(
                    f,
                    "No enabled trading direction between '{}' and '{}'",
                    from_asset, to_asset
                )
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::*;
    use std::collections::HashMap;

    fn create_test_registry() -> ChainRegistry {
        let mut registry = ChainRegistry::new();

        // Add Ethereum chain
        let eth_chain = ChainConfig {
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
        registry.add_chain(eth_chain).unwrap();

        // Add ETH asset
        let eth_asset = AssetConfig {
            asset_id_base: AssetIdBase::new("slip44", "60").unwrap(),
            symbol: "ETH".to_string(),
            name: "Ethereum".to_string(),
            is_native: true,
            decimals: 18,
            metadata: HashMap::new(),
        };
        registry.add_asset(eth_asset).unwrap();

        registry
    }

    #[test]
    fn test_get_chain_id_from_config() {
        let registry = create_test_registry();
        let chain_id = registry
            .get_chain_id_from_config("Ethereum Mainnet")
            .unwrap();
        assert_eq!(chain_id.to_string(), "eip155:1");
    }

    #[test]
    fn test_get_asset_id_from_config() {
        let registry = create_test_registry();
        let asset_id = registry
            .get_asset_id_from_config("ETH", "eip155:1")
            .unwrap();
        assert_eq!(asset_id.to_string(), "eip155:1/slip44:60");
    }

    #[test]
    fn test_registry_statistics() {
        let registry = create_test_registry();
        let stats = registry.get_statistics();

        assert_eq!(stats.total_chains, 1);
        assert_eq!(stats.mainnet_chains, 1);
        assert_eq!(stats.testnet_chains, 0);
        assert_eq!(stats.total_assets, 1);
    }

    #[test]
    fn test_health_check() {
        let registry = create_test_registry();
        let issues = registry.check_health();

        // Should be healthy
        assert!(issues.is_empty());
    }
}
