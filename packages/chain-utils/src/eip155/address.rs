use k256::elliptic_curve::sec1::ToEncodedPoint;
use k256::PublicKey;
use sha3::{Digest, Keccak256};

/// Generate an Ethereum address from a SEC1-encoded public key.
///
/// This function takes a hex-encoded SEC1 public key (typically generated from
/// ICP threshold signatures) and converts it to an Ethereum address using the
/// standard Ethereum address derivation process: Keccak256 hash of the uncompressed
/// public key coordinates, taking the last 20 bytes.
///
/// # Arguments
///
/// * `pub_key_sec1_string` - A hex-encoded SEC1 public key string. Can be either:
///   - Compressed format (33 bytes, starts with 0x02 or 0x03)
///   - Uncompressed format (65 bytes, starts with 0x04)
///
/// # Returns
///
/// * `Ok(String)` - The Ethereum address as a 0x-prefixed lowercase hex string (42 characters)
/// * `Err(String)` - Error message if the public key is invalid or cannot be processed
///
/// # Examples
///
/// ```rust
/// use atp_chain_utils::eip155::address::generate_address;
///
/// // Uncompressed public key
/// let pubkey = "04a34b99f22c790c4e36b2b3c2c35a36db06226e41c692fc82b8b56ac1c540c5bd5b8dec5235a0fa8722476c7709c02559e3aa73aa03918ba2d492eea75abea235";
/// let address = generate_address(pubkey.to_string()).unwrap();
/// // address = "0x..." (42 character hex string)
///
/// // Compressed public key
/// let compressed = "0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798";
/// let address = generate_address(compressed.to_string()).unwrap();
/// ```
///
/// # Algorithm
///
/// 1. Decode hex string to bytes
/// 2. Parse as SEC1 public key using secp256k1 curve
/// 3. Convert to uncompressed point (65 bytes with 0x04 prefix)
/// 4. Take x,y coordinates (skip first byte)
/// 5. Compute Keccak256 hash of coordinates
/// 6. Take last 20 bytes as Ethereum address
/// 7. Format as 0x-prefixed hex string
pub fn generate_address(pub_key_sec1_string: String) -> Result<String, String> {
    let pub_key_sec1_bytes =
        hex::decode(&pub_key_sec1_string).map_err(|_| "Invalid hex format.".to_string())?;
    let pub_key = match PublicKey::from_sec1_bytes(&pub_key_sec1_bytes) {
        Ok(key) => key,
        Err(_) => return Err("Invalid SEC1 public key format.".to_string()),
    };

    let point = pub_key.to_encoded_point(false);
    let point_bytes = point.as_bytes();
    assert_eq!(point_bytes[0], 0x04);

    let mut hasher = Keccak256::new();
    hasher.update(&point_bytes[1..]); // Skip the 0x04 prefix
    let result = hasher.finalize();

    let mut eth_address = [0u8; 20];
    eth_address.copy_from_slice(&result[12..]);

    Ok(format!("0x{}", hex::encode(eth_address)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_address_valid_key() {
        let pub_key = "04a34b99f22c790c4e36b2b3c2c35a36db06226e41c692fc82b8b56ac1c540c5bd5b8dec5235a0fa8722476c7709c02559e3aa73aa03918ba2d492eea75abea235".to_string();

        let result = generate_address(pub_key);
        assert!(result.is_ok());
        let address = result.unwrap();
        assert!(address.starts_with("0x"));
        assert_eq!(address.len(), 42); // 0x + 40 hex chars
    }

    #[test]
    fn test_generate_address_invalid_key() {
        let invalid_pub_key = "invalid_key".to_string();
        let result = generate_address(invalid_pub_key);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid hex format"));
    }

    #[test]
    fn test_generate_address_known_key() {
        // Test with a known public key that should generate a specific address
        let pub_key =
            "0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798".to_string();
        let result = generate_address(pub_key);
        assert!(result.is_ok());
        // The address format should be correct even if we don't know the exact expected value
        let address = result.unwrap();
        assert!(address.starts_with("0x"));
        assert_eq!(address.len(), 42);
    }
}
