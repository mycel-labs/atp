use atp_caip::chain_id::ChainId;

/// Generate a blockchain address from a public key and CAIP chain identifier.
///
/// This function routes address generation to the appropriate blockchain-specific
/// implementation based on the chain namespace.
///
/// # Arguments
///
/// * `pub_key` - A string representation of the public key. Format depends on the chain:
///   - For EIP155 (Ethereum): Hex-encoded SEC1 public key (with or without 0x04 prefix)
///   - For Solana: 32-byte hex-encoded public key
///   - For BIP122 (Bitcoin): Hex-encoded SEC1 public key (compressed or uncompressed)
/// * `chain_id` - A CAIP-2 chain identifier specifying the target blockchain
///
/// # Returns
///
/// * `Ok(String)` - The generated blockchain address
/// * `Err(String)` - Error message if generation fails
///
/// # Supported Chains
///
/// - **eip155**: Ethereum and EVM-compatible chains (generates 0x-prefixed addresses)
/// - **solana**: Solana blockchain (generates base58-encoded addresses)
/// - **bip122**: Bitcoin and Bitcoin-compatible chains (generates P2PKH base58-encoded addresses)
///
/// # Examples
///
/// ```rust
/// use atp_caip::chain_id::ChainId;
/// use atp_chain_utils::address::generate_address;
///
/// // Generate Ethereum address
/// let eth_pubkey = "04a34b99f22c790c4e36b2b3c2c35a36db06226e41c692fc82b8b56ac1c540c5bd5b8dec5235a0fa8722476c7709c02559e3aa73aa03918ba2d492eea75abea235";
/// let eth_chain = ChainId::new("eip155", "1").unwrap();
/// let eth_address = generate_address(eth_pubkey.to_string(), eth_chain).unwrap();
///
/// // Generate Solana address
/// let sol_pubkey = "e258d6e13adfb7b6eb771e0c9e8b1e3d4e3f1a2b3c4d5e6f7a8b9c0d1e2f3a4b";
/// let sol_chain = ChainId::new("solana", "mainnet").unwrap();
/// let sol_address = generate_address(sol_pubkey.to_string(), sol_chain).unwrap();
///
/// // Generate Bitcoin address
/// let btc_pubkey = "04a34b99f22c790c4e36b2b3c2c35a36db06226e41c692fc82b8b56ac1c540c5bd5b8dec5235a0fa8722476c7709c02559e3aa73aa03918ba2d492eea75abea235";
/// let btc_chain = ChainId::new("bip122", "000000000019d6689c085ae165831e93").unwrap();
/// let btc_address = generate_address(btc_pubkey.to_string(), btc_chain).unwrap();
/// ```
pub fn generate_address(pub_key: String, chain_id: ChainId) -> Result<String, String> {
    let address = match chain_id.namespace() {
        "eip155" => crate::eip155::address::generate_address(pub_key),
        "solana" => crate::solana::address::generate_address(pub_key),
        "bip122" => crate::bip122::address::generate_p2pkh_address(
            pub_key,
            chain_id.reference().to_string(),
            true, // Default to compressed format
        ),
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
    fn test_generate_address_bitcoin() {
        let pub_key = "04a34b99f22c790c4e36b2b3c2c35a36db06226e41c692fc82b8b56ac1c540c5bd5b8dec5235a0fa8722476c7709c02559e3aa73aa03918ba2d492eea75abea235".to_string();
        let chain_id = ChainId::new("bip122", "000000000019d6689c085ae165831e93").unwrap();

        let result = generate_address(pub_key, chain_id);
        assert!(result.is_ok());
        let address = result.unwrap();
        assert!(address.starts_with('1')); // Bitcoin mainnet P2PKH
        assert!(address.len() >= 26 && address.len() <= 35);
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
