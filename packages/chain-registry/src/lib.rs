pub mod error;
pub mod query;
pub mod registry;
pub mod types;

// Re-export main types for convenience
pub use error::{ChainRegistryError, Result};
pub use query::{HealthIssue, RegistryStatistics};
pub use registry::{ChainRegistry, DEFAULT_CONFIG};
pub use types::*;

// Re-export CAIP types for convenience
pub use solveros_caip::{AssetId, AssetIdBase, ChainId, Curve, TokenPair};

#[cfg(test)]
mod integration_tests {
    use super::*;
    use std::collections::HashMap;
    use std::str::FromStr;

    #[test]
    fn test_full_workflow() {
        // Create a new registry
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
            assets: vec![
                AssetIdBase::new("slip44", "60").unwrap(), // ETH
                AssetIdBase::new("erc20", "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48").unwrap(), // USDC
            ],
            metadata: HashMap::new(),
        };
        registry.add_chain(eth_chain).unwrap();

        // Add Solana chain
        let sol_chain = ChainConfig {
            chain_id: "solana:mainnet".to_string(),
            name: "Solana Mainnet".to_string(),
            native_asset: "slip44:501".to_string(),
            rpc_endpoints: vec!["https://api.mainnet-beta.solana.com".to_string()],
            explorer_url: Some("https://solscan.io".to_string()),
            cryptographic_curve: vec![Curve::Ed25519],
            is_testnet: false,
            assets: vec![AssetIdBase::new("slip44", "501").unwrap()], // SOL
            metadata: HashMap::new(),
        };
        registry.add_chain(sol_chain).unwrap();

        // Add assets
        let eth_asset = AssetConfig {
            asset_id_base: AssetIdBase::new("slip44", "60").unwrap(),
            symbol: "ETH".to_string(),
            name: "Ethereum".to_string(),
            is_native: true,
            decimals: 18,
            metadata: HashMap::new(),
        };
        registry.add_asset(eth_asset).unwrap();

        let usdc_asset = AssetConfig {
            asset_id_base: AssetIdBase::new("erc20", "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48")
                .unwrap(),
            symbol: "USDC".to_string(),
            name: "USD Coin".to_string(),
            is_native: false,
            decimals: 6,
            metadata: HashMap::new(),
        };
        registry.add_asset(usdc_asset).unwrap();

        let sol_asset = AssetConfig {
            asset_id_base: AssetIdBase::new("slip44", "501").unwrap(),
            symbol: "SOL".to_string(),
            name: "Solana".to_string(),
            is_native: true,
            decimals: 9,
            metadata: HashMap::new(),
        };
        registry.add_asset(sol_asset).unwrap();

        // Create trading pairs
        let eth_chain_id = ChainId::from_str("eip155:1").unwrap();
        let sol_chain_id = ChainId::from_str("solana:mainnet").unwrap();

        let eth_asset_id = AssetId::new(eth_chain_id.clone(), "slip44", "60").unwrap();
        let usdc_asset_id = AssetId::new(
            eth_chain_id.clone(),
            "erc20",
            "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
        )
        .unwrap();
        let sol_asset_id = AssetId::new(sol_chain_id.clone(), "slip44", "501").unwrap();

        // ETH -> SOL (cross-chain)
        let eth_sol_pair = TokenPair::new(eth_asset_id.clone(), sol_asset_id.clone());
        registry.add_token_pair(eth_sol_pair).unwrap();

        // ETH -> USDC (same chain)
        let eth_usdc_pair = TokenPair::new(eth_asset_id.clone(), usdc_asset_id.clone());
        registry.add_token_pair(eth_usdc_pair).unwrap();

        // Test queries
        assert_eq!(registry.list_chains().len(), 2);
        assert_eq!(registry.list_assets().len(), 3);
        assert_eq!(registry.list_token_pairs().len(), 2);

        // Test chain queries
        let eth_chains = registry.get_chains_by_curve(&Curve::Secp256k1);
        assert_eq!(eth_chains.len(), 1);
        assert_eq!(eth_chains[0].name, "Ethereum Mainnet");

        let mainnet_chains = registry.get_mainnet_chains();
        assert_eq!(mainnet_chains.len(), 2);

        // Test asset queries
        let eth_assets = registry.get_chain_assets(&eth_chain_id).unwrap();
        assert_eq!(eth_assets.len(), 2);

        let native_eth = registry.get_native_asset(&eth_chain_id).unwrap();
        assert_eq!(native_eth.to_string(), "eip155:1/slip44:60");

        // Test pair queries
        let cross_chain_pairs = registry.get_cross_chain_pairs();
        assert_eq!(cross_chain_pairs.len(), 1);

        let same_chain_pairs = registry.get_same_chain_pairs();
        assert_eq!(same_chain_pairs.len(), 1);

        let eth_pairs = registry.get_pairs_for_asset(&eth_asset_id);
        assert_eq!(eth_pairs.len(), 2);

        // Test trading routes
        let routes = registry.find_trading_routes(&eth_asset_id, &sol_asset_id, 3);
        assert_eq!(routes.len(), 1); // Direct route

        // Test statistics
        let stats = registry.get_statistics();
        assert_eq!(stats.total_chains, 2);
        assert_eq!(stats.total_assets, 3);
        assert_eq!(stats.total_pairs, 2);
        assert_eq!(stats.cross_chain_pairs, 1);
        assert_eq!(stats.same_chain_pairs, 1);

        // Test health check
        let health_issues = registry.check_health();
        assert!(health_issues.is_empty());

        // Test requested features
        let chain_id_from_name = registry
            .get_chain_id_from_config("Ethereum Mainnet")
            .unwrap();
        assert_eq!(chain_id_from_name.to_string(), "eip155:1");

        let asset_id_from_config = registry
            .get_asset_id_from_config("ETH", "eip155:1")
            .unwrap();
        assert_eq!(asset_id_from_config.to_string(), "eip155:1/slip44:60");
    }

    #[test]
    fn test_serialization() {
        let mut registry = ChainRegistry::new();

        let asset_config = AssetConfig {
            asset_id_base: AssetIdBase::new("slip44", "60").unwrap(),
            symbol: "ETH".to_string(),
            name: "Ethereum".to_string(),
            is_native: true,
            decimals: 18,
            metadata: HashMap::new(),
        };

        registry.add_asset(asset_config.clone()).unwrap();
        assert_eq!(registry.list_assets().len(), 1);

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
        registry.add_chain(chain_config).unwrap();

        // Test TOML serialization
        let toml_str = registry.to_toml().unwrap();
        assert!(toml_str.contains("eip155:1"));
        assert!(toml_str.contains("Ethereum Mainnet"));

        // Test JSON serialization
        let json_str = registry.to_json().unwrap();
        assert!(json_str.contains("eip155:1"));
        assert!(json_str.contains("Ethereum Mainnet"));

        // Test round-trip
        let config_from_toml = ChainRegistry::config_from_toml(&toml_str).unwrap();
        println!("Config from TOML: {:?}", config_from_toml);
        let registry_from_config = ChainRegistry::from_config(config_from_toml).unwrap();
        assert_eq!(registry_from_config.list_chains().len(), 1);
    }
}
