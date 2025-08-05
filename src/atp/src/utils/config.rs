use atp_caip::Curve;
use atp_chain_registry::{ChainConfig, ChainRegistry};
use std::collections::HashMap;

/*
* dfx_test_key: Only available on the local replica started by dfx.
* test_key_1: Test key available on the ICP mainnet.
* key_1: Production key available on the ICP mainnet.
*/

#[cfg(feature = "local")]
pub const KEY_ID: &str = "dfx_test_key";

#[cfg(feature = "test")]
pub const KEY_ID: &str = "test_key_1";

#[cfg(feature = "production")]
pub const KEY_ID: &str = "key_1";

pub fn get_chain_registry() -> Result<ChainRegistry, String> {
    // Create hardcoded chain configurations for canister environment
    let mut registry = ChainRegistry::new();

    // EIP155 Wildcard Chain (Ethereum family)
    let eip155_chain = ChainConfig {
        chain_id: "eip155:*".to_string(),
        name: "EIP155 Wildcard Chain".to_string(),
        native_asset: "slip44:60".to_string(),
        rpc_endpoints: vec![],
        explorer_url: None,
        cryptographic_curve: vec![Curve::Secp256k1],
        is_testnet: false,
        assets: vec![],
        metadata: HashMap::new(),
    };

    registry
        .add_chain(eip155_chain)
        .map_err(|e| format!("Failed to add EIP155 chain: {}", e))?;

    // Solana Wildcard Chain
    let solana_chain = ChainConfig {
        chain_id: "solana:*".to_string(),
        name: "Solana Wildcard Chain".to_string(),
        native_asset: "slip44:501".to_string(),
        rpc_endpoints: vec![],
        explorer_url: None,
        cryptographic_curve: vec![Curve::Ed25519],
        is_testnet: false,
        assets: vec![],
        metadata: HashMap::new(),
    };

    registry
        .add_chain(solana_chain)
        .map_err(|e| format!("Failed to add Solana chain: {}", e))?;

    // BIP122 Wildcard Chain (Bitcoin family)
    let bip122_chain = ChainConfig {
        chain_id: "bip122:*".to_string(),
        name: "BIP122 Wildcard Chain".to_string(),
        native_asset: "".to_string(),
        rpc_endpoints: vec![],
        explorer_url: None,
        cryptographic_curve: vec![Curve::Secp256k1],
        is_testnet: false,
        assets: vec![],
        metadata: HashMap::new(),
    };

    registry
        .add_chain(bip122_chain)
        .map_err(|e| format!("Failed to add BIP122 chain: {}", e))?;

    Ok(registry)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_chain_registry() {
        let registry = get_chain_registry();
        assert!(
            registry.is_ok(),
            "Failed to get chain registry: {:?}",
            registry.err()
        );
    }
}
