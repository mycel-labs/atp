/// Generate a Solana address from a hex-encoded public key.
///
/// This function takes a 32-byte hex-encoded public key (typically generated from
/// ICP threshold signatures) and converts it directly to a Solana address using
/// base58 encoding. In Solana, public keys are used directly as addresses.
///
/// # Arguments
///
/// * `pub_key_hex_string` - A 32-byte public key encoded as a 64-character hex string
///   (without 0x prefix). Must be exactly 32 bytes when decoded.
///
/// # Returns
///
/// * `Ok(String)` - The Solana address as a base58-encoded string (typically 32-44 characters)
/// * `Err(String)` - Error message if the hex string is invalid or not exactly 32 bytes
///
/// # Examples
///
/// ```rust
/// use atp_chain_utils::solana::address::generate_address;
///
/// let pubkey = "e258d6e13adfb7b6eb771e0c9e8b1e3d4e3f1a2b3c4d5e6f7a8b9c0d1e2f3a4b";
/// let address = generate_address(pubkey.to_string()).unwrap();
/// // address = "Fe3d...7z" (base58 string)
///
/// // Edge case: all zeros
/// let zero_key = "0000000000000000000000000000000000000000000000000000000000000000";
/// let zero_address = generate_address(zero_key.to_string()).unwrap();
/// // zero_address = "11111111111111111111111111111111"
/// ```
///
/// # Algorithm
///
/// 1. Decode hex string to 32-byte array
/// 2. Validate length is exactly 32 bytes
/// 3. Encode bytes using base58 alphabet
/// 4. Return base58 string as Solana address
///
/// # Notes
///
/// - Solana uses Ed25519 curve, but this function works with any 32-byte key
/// - The public key bytes are used directly as the address (no hashing)
/// - Base58 encoding uses Bitcoin's alphabet (no 0, O, I, l characters)
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
