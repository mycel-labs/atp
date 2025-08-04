pub fn generate_address(pub_key_hex_string: String) -> Result<String, String> {
    let pub_key_bytes = hex::decode(&pub_key_hex_string)
        .map_err(|e| format!("Failed to decode public key hex: {}", e))?;
    // Ensure the public key is 32 bytes long
    if pub_key_bytes.len() != 32 {
        return Err("Public key must be 32 bytes long".to_string());
    }
    // Encode the public key to a base58 string
    let pub_key_base58_string = bs58::encode(pub_key_bytes).into_string();
    Ok(pub_key_base58_string)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_address_valid_key() {
        // Test with a valid 32-byte hex public key
        let pub_key =
            "e258d6e13adfb7b6eb771e0c9e8b1e3d4e3f1a2b3c4d5e6f7a8b9c0d1e2f3a4b".to_string();

        let result = generate_address(pub_key);
        assert!(result.is_ok());
        let address = result.unwrap();
        // Solana addresses are base58 encoded and typically 32-44 characters
        assert!(address.len() >= 32 && address.len() <= 44);
        // Should be valid base58
        assert!(bs58::decode(&address).into_vec().is_ok());
    }

    #[test]
    fn test_generate_address_invalid_hex() {
        let invalid_pub_key = "invalid_hex".to_string();
        let result = generate_address(invalid_pub_key);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("Failed to decode public key hex"));
    }

    #[test]
    fn test_generate_address_wrong_length() {
        // Test with hex that's too short (only 2 bytes when decoded)
        let short_pub_key = "abcd".to_string();
        let result = generate_address(short_pub_key);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("Public key must be 32 bytes long"));
    }

    #[test]
    fn test_generate_address_too_long() {
        // Test with hex that's too long (33 bytes when decoded)
        let long_pub_key =
            "e258d6e13adfb7b6eb771e0c9e8b1e3d4e3f1a2b3c4d5e6f7a8b9c0d1e2f3a4bff".to_string();
        let result = generate_address(long_pub_key);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("Public key must be 32 bytes long"));
    }

    #[test]
    fn test_generate_address_known_key() {
        // Test with all zeros (valid but edge case)
        let pub_key =
            "0000000000000000000000000000000000000000000000000000000000000000".to_string();
        let result = generate_address(pub_key);
        assert!(result.is_ok());
        let address = result.unwrap();
        // Should generate a valid base58 address
        assert_eq!(address, "11111111111111111111111111111111");
    }
}
