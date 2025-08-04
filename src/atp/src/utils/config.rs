use atp_chain_registry::ChainRegistry;

/*
* dfx_test_key: Only available on the local replica started by dfx.
* test_key_1: Test key available on the ICP mainnet.
* key_1: Production key available on the ICP mainnet.
*/
pub const KEY_ID: &str = "dfx_test_key";
// pub const KEY_ID: &str = "test_key_1";
// pub const KEY_ID: &str = "key_1";
//

pub fn get_chain_registry() -> Result<ChainRegistry, String> {
    ChainRegistry::from_file("config.toml")
        .map_err(|e| format!("Failed to load chain registry: {}", e))
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
