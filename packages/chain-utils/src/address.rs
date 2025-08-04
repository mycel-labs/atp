use atp_caip::chain_id::ChainId;

pub fn generate_address(pub_key: String, chain_id: ChainId) -> Result<String, String> {
    let address = match chain_id.namespace() {
        "eip155" => crate::eip155::address::generate_address(pub_key),
        "solana" => crate::solana::address::generate_address(pub_key),
        _ => return Err(format!("Unsupported namespace: {}", chain_id.namespace())),
    };
    return address;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_address_eip155() {
        let pub_key = "04a34b99f22c790c4e36b2b3c2c35a36db06226e41c692fc82b8b56ac1c540c5bd5b8dec5235a0fa8722476c7709c02559e3aa73aa03918ba2d492eea75abea235".to_string();
        let chain_id = ChainId::new("eip155", "1").unwrap();

        let result = generate_address(pub_key, chain_id);
        assert!(result.is_ok());
        let address = result.unwrap();
        assert!(address.starts_with("0x"));
        assert_eq!(address.len(), 42);
    }

    #[test]
    fn test_generate_address_solana() {
        let pub_key =
            "e258d6e13adfb7b6eb771e0c9e8b1e3d4e3f1a2b3c4d5e6f7a8b9c0d1e2f3a4b".to_string();
        let chain_id = ChainId::new("solana", "5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp").unwrap();

        let result = generate_address(pub_key, chain_id);
        assert!(result.is_ok());
        let address = result.unwrap();
        assert!(address.len() >= 32 && address.len() <= 44);
    }

    #[test]
    fn test_generate_address_unsupported_namespace() {
        let pub_key = "test_key".to_string();
        let chain_id = ChainId::new("test", "000000000019d6689c085ae165831e93").unwrap();

        let result = generate_address(pub_key, chain_id);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Unsupported namespace: test"));
    }
}
