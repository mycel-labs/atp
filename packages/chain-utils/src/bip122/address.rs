use bech32::{segwit, Hrp};
use k256::elliptic_curve::sec1::ToEncodedPoint;
use k256::PublicKey;
use ripemd::Ripemd160;
use sha2::{Digest, Sha256};

/// Generate a Bitcoin address from a SEC1-encoded public key using the chain reference.
///
/// This function takes a hex-encoded SEC1 public key (typically generated from
/// ICP threshold signatures) and converts it to a Bitcoin address. The address type
/// is determined by the chain_id reference:
/// - "000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce26f" (Bitcoin mainnet): P2PKH
/// - "000000000933ea01ad0ee984209779baaec3ced90fa3f408719526f8d77f4943" (Bitcoin testnet): P2PKH testnet
/// - "0f9188f13cb7b2c71f2a335e3a4fc328bf5beb436012afca590b1a11466e2206" (Bitcoin regtest): P2PKH regtest
///
/// # Arguments
///
/// * `pub_key_sec1_string` - A hex-encoded SEC1 public key string. Can be either:
///   - Compressed format (33 bytes, starts with 0x02 or 0x03)
///   - Uncompressed format (65 bytes, starts with 0x04)
/// * `chain_reference` - The chain reference from CAIP-2 chain identifier
/// * `use_compressed` - Whether to use compressed format for address generation:
///   - `true`: Use compressed public key (recommended, generates different addresses)
///   - `false`: Use uncompressed public key (legacy format)
///
/// # Returns
///
/// * `Ok(String)` - The Bitcoin address as a base58-encoded string
/// * `Err(String)` - Error message if the public key is invalid or chain is unsupported
///
/// # Examples
///
/// ```rust
/// use atp_chain_utils::bip122::address::generate_p2pkh_address;
///
/// // Bitcoin mainnet P2PKH address with compressed public key (recommended)
/// let pubkey = "04a34b99f22c790c4e36b2b3c2c35a36db06226e41c692fc82b8b56ac1c540c5bd5b8dec5235a0fa8722476c7709c02559e3aa73aa03918ba2d492eea75abea235";
/// let mainnet_ref = "000000000019d6689c085ae165831e93";
/// let address = generate_p2pkh_address(pubkey.to_string(), mainnet_ref.to_string(), true).unwrap();
///
/// // Bitcoin testnet P2PKH address with uncompressed public key (legacy)
/// let testnet_ref = "000000000933ea01ad0ee984209779ba";
/// let testnet_address = generate_p2pkh_address(pubkey.to_string(), testnet_ref.to_string(), false).unwrap();
/// ```
///
/// # Algorithm
///
/// 1. Decode hex string to bytes
/// 2. Parse as SEC1 public key using secp256k1 curve
/// 3. Convert to desired format based on use_compressed flag
/// 4. Compute SHA256 hash of the public key
/// 5. Compute RIPEMD160 hash of SHA256 result
/// 6. Add network version byte (0x00 for mainnet, 0x6f for testnet, 0x6f for regtest)
/// 7. Compute double SHA256 checksum
/// 8. Append first 4 bytes of checksum
/// 9. Encode result as base58
pub fn generate_p2pkh_address(
    pub_key_sec1_string: String,
    chain_reference: String,
    use_compressed: bool,
) -> Result<String, String> {
    // Decode the hex public key
    let pub_key_sec1_bytes =
        hex::decode(&pub_key_sec1_string).map_err(|_| "Invalid hex format.".to_string())?;

    // Parse as SEC1 public key
    let pub_key = match PublicKey::from_sec1_bytes(&pub_key_sec1_bytes) {
        Ok(key) => key,
        Err(_) => return Err("Invalid SEC1 public key format.".to_string()),
    };

    // Convert to desired format based on use_compressed flag
    let point = pub_key.to_encoded_point(use_compressed);
    let pubkey_bytes = point.as_bytes();

    // Compute SHA256 hash of the public key
    let mut sha256_hasher = Sha256::new();
    sha256_hasher.update(pubkey_bytes);
    let sha256_result = sha256_hasher.finalize();

    // Compute RIPEMD160 hash of SHA256 result
    let mut ripemd160_hasher = Ripemd160::new();
    ripemd160_hasher.update(&sha256_result);
    let ripemd160_result = ripemd160_hasher.finalize();

    // Determine network version byte based on chain reference
    let version_byte = match chain_reference.as_str() {
        // Full genesis block hashes
        "000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce26f" => 0x00, // Bitcoin mainnet
        "000000000933ea01ad0ee984209779baaec3ced90fa3f408719526f8d77f4943" => 0x6f, // Bitcoin testnet
        "0f9188f13cb7b2c71f2a335e3a4fc328bf5beb436012afca590b1a11466e2206" => 0x6f, // Bitcoin regtest (same as testnet)
        // Truncated versions (CAIP-2 compatible)
        "000000000019d6689c085ae165831e93" => 0x00, // Bitcoin mainnet (truncated)
        "000000000933ea01ad0ee984209779ba" => 0x6f, // Bitcoin testnet (truncated)
        "0f9188f13cb7b2c71f2a335e3a4fc328" => 0x6f, // Bitcoin regtest (truncated)
        _ => {
            return Err(format!(
                "Unsupported Bitcoin chain reference: {}",
                chain_reference
            ))
        }
    };

    // Create the versioned payload (version byte + ripemd160 hash)
    let mut versioned_payload = Vec::with_capacity(21);
    versioned_payload.push(version_byte);
    versioned_payload.extend_from_slice(&ripemd160_result);

    // Compute double SHA256 for checksum
    let mut checksum_hasher1 = Sha256::new();
    checksum_hasher1.update(&versioned_payload);
    let checksum_intermediate = checksum_hasher1.finalize();

    let mut checksum_hasher2 = Sha256::new();
    checksum_hasher2.update(&checksum_intermediate);
    let checksum_result = checksum_hasher2.finalize();

    // Take first 4 bytes as checksum
    let checksum = &checksum_result[0..4];

    // Create final payload (versioned payload + checksum)
    let mut final_payload = versioned_payload;
    final_payload.extend_from_slice(checksum);

    // Encode as base58
    let address = bs58::encode(final_payload).into_string();

    Ok(address)
}

/// Generate a Bitcoin P2WPKH (Pay-to-Witness-PubkeyHash) address from a SEC1-encoded public key.
///
/// P2WPKH addresses are native SegWit addresses that use bech32 encoding and always
/// require compressed public keys. They start with "bc1" for mainnet and "tb1" for testnet.
///
/// # Arguments
///
/// * `pub_key_sec1_string` - A hex-encoded SEC1 public key string (compressed or uncompressed)
/// * `chain_reference` - The chain reference from CAIP-2 chain identifier
///
/// # Returns
///
/// * `Ok(String)` - The Bitcoin P2WPKH address as a bech32-encoded string
/// * `Err(String)` - Error message if the public key is invalid or chain is unsupported
///
/// # Examples
///
/// ```rust
/// use atp_chain_utils::bip122::address::generate_p2wpkh_address;
///
/// let pubkey = "0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798";
/// let mainnet_ref = "000000000019d6689c085ae165831e93";
/// let address = generate_p2wpkh_address(pubkey.to_string(), mainnet_ref.to_string()).unwrap();
/// // Expected: "bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4"
/// ```
pub fn generate_p2wpkh_address(
    pub_key_sec1_string: String,
    chain_reference: String,
) -> Result<String, String> {
    // Decode the hex public keybc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4
    let pub_key_sec1_bytes =
        hex::decode(&pub_key_sec1_string).map_err(|_| "Invalid hex format.".to_string())?;

    // Parse as SEC1 public key
    let pub_key = match PublicKey::from_sec1_bytes(&pub_key_sec1_bytes) {
        Ok(key) => key,
        Err(_) => return Err("Invalid SEC1 public key format.".to_string()),
    };

    // P2WPKH always uses compressed public keys
    let point = pub_key.to_encoded_point(true);
    let pubkey_bytes = point.as_bytes();

    // Compute SHA256 hash of the compressed public key
    let mut sha256_hasher = Sha256::new();
    sha256_hasher.update(pubkey_bytes);
    let sha256_result = sha256_hasher.finalize();

    // Compute RIPEMD160 hash of SHA256 result
    let mut ripemd160_hasher = Ripemd160::new();
    ripemd160_hasher.update(&sha256_result);
    let ripemd160_result = ripemd160_hasher.finalize();

    // Determine human-readable part based on chain reference
    let hrp = match chain_reference.as_str() {
        // Full genesis block hashes
        "000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce26f" => "bc", // Bitcoin mainnet
        "000000000933ea01ad0ee984209779baaec3ced90fa3f408719526f8d77f4943" => "tb", // Bitcoin testnet
        "0f9188f13cb7b2c71f2a335e3a4fc328bf5beb436012afca590b1a11466e2206" => "tb", // Bitcoin regtest (same as testnet)
        // Truncated versions (CAIP-2 compatible)
        "000000000019d6689c085ae165831e93" => "bc", // Bitcoin mainnet (truncated)
        "000000000933ea01ad0ee984209779ba" => "tb", // Bitcoin testnet (truncated)
        "0f9188f13cb7b2c71f2a335e3a4fc328" => "tb", // Bitcoin regtest (truncated)
        _ => {
            return Err(format!(
                "Unsupported Bitcoin chain reference: {}",
                chain_reference
            ))
        }
    };

    // Create witness program data (20-byte pubkey hash)
    let witness_program = &ripemd160_result[..];

    // Create HRP
    let hrp = Hrp::parse(hrp).map_err(|e| format!("Invalid HRP: {}", e))?;

    // Encode as bech32 with witness version 0
    let address = segwit::encode(hrp, segwit::VERSION_0, witness_program)
        .map_err(|e| format!("Bech32 encoding failed: {}", e))?;

    Ok(address)
}

#[cfg(test)]
mod tests {
    use super::*;
    const PUB_KEY_UNCOMPRESSED: &str = "0479be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798483ada7726a3c4655da4fbfc0e1108a8fd17b448a68554199c47d08ffb10d4b8";
    const PUB_KEY_COMPRESSED: &str =
        "0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798";

    //Reference: https://secretscan.org/PrivateKeyHex.php?symbol=BTC&prefix=00
    const EXPECTED_UNCOMPRESSED_ADDRESS_MAINNET: &str = "1EHNa6Q4Jz2uvNExL497mE43ikXhwF6kZm";
    const EXPECTED_COMPRESSED_ADDRESS_MAINNET: &str = "1BgGZ9tcN4rm9KBzDn7KprQz87SZ26SAMH";

    //Reference: https://secretscan.org/PrivateKeyHex.php?symbol=BTC&prefix=6f
    const EXPECTED_UNCOMPRESSED_ADDRESS_TESTNET: &str = "mtoKs9V381UAhUia3d7Vb9GNak8Qvmcsme";
    const EXPECTED_COMPRESSED_ADDRESS_TESTNET: &str = "mrCDrCybB6J1vRfbwM5hemdJz73FwDBC8r";

    // P2WPKH test constants
    // Reference: https://secretscan.org/Bech32?prefix=bc
    const EXPECTED_P2WPKH_ADDRESS_MAINNET: &str = "bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4";
    // Reference: https://secretscan.org/Bech32?prefix=tb
    const EXPECTED_P2WPKH_ADDRESS_TESTNET: &str = "tb1qw508d6qejxtdg4y5r3zarvary0c5xw7kxpjzsx";

    #[test]
    fn test_generate_mainnet_address() {
        let mainnet_ref = "000000000019d6689c085ae165831e93";

        let result = generate_p2pkh_address(
            PUB_KEY_UNCOMPRESSED.to_string(),
            mainnet_ref.to_string(),
            false,
        );
        assert!(result.is_ok());
        let address = result.unwrap();
        assert_eq!(address, EXPECTED_UNCOMPRESSED_ADDRESS_MAINNET);
    }

    #[test]
    fn test_generate_compressed_mainnet_address() {
        let mainnet_ref = "000000000019d6689c085ae165831e93";

        let result = generate_p2pkh_address(
            PUB_KEY_UNCOMPRESSED.to_string(),
            mainnet_ref.to_string(),
            true,
        );
        assert!(result.is_ok());
        let address = result.unwrap();
        assert_eq!(address, EXPECTED_COMPRESSED_ADDRESS_MAINNET);
    }

    #[test]
    fn test_generate_testnet_address() {
        let testnet_ref = "000000000933ea01ad0ee984209779ba";

        let result = generate_p2pkh_address(
            PUB_KEY_UNCOMPRESSED.to_string(),
            testnet_ref.to_string(),
            false,
        );
        assert!(result.is_ok());
        let address = result.unwrap();
        assert_eq!(address, EXPECTED_UNCOMPRESSED_ADDRESS_TESTNET);
    }

    #[test]
    fn test_generate_compressed_testnet_address() {
        let testnet_ref = "000000000933ea01ad0ee984209779ba";

        let result = generate_p2pkh_address(
            PUB_KEY_UNCOMPRESSED.to_string(),
            testnet_ref.to_string(),
            true,
        );
        assert!(result.is_ok());
        let address = result.unwrap();
        assert_eq!(address, EXPECTED_COMPRESSED_ADDRESS_TESTNET);
    }

    #[test]
    fn test_compressed_public_key() {
        let mainnet_ref = "000000000019d6689c085ae165831e93";

        let result = generate_p2pkh_address(
            PUB_KEY_COMPRESSED.to_string(),
            mainnet_ref.to_string(),
            true,
        );
        assert!(result.is_ok());
        let address = result.unwrap();
        assert_eq!(address, EXPECTED_COMPRESSED_ADDRESS_MAINNET);
    }

    #[test]
    fn test_invalid_hex() {
        let invalid_pub_key = "invalid_hex".to_string();
        let mainnet_ref = "000000000019d6689c085ae165831e93";

        let result = generate_p2pkh_address(invalid_pub_key, mainnet_ref.to_string(), true);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid hex format"));
    }

    #[test]
    fn test_invalid_sec1_key() {
        let invalid_pub_key = "abcd1234".to_string(); // Valid hex but invalid SEC1
        let mainnet_ref = "000000000019d6689c085ae165831e93";

        let result = generate_p2pkh_address(invalid_pub_key, mainnet_ref.to_string(), true);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("Invalid SEC1 public key format"));
    }

    #[test]
    fn test_unsupported_chain_reference() {
        let pub_key = "04a34b99f22c790c4e36b2b3c2c35a36db06226e41c692fc82b8b56ac1c540c5bd5b8dec5235a0fa8722476c7709c02559e3aa73aa03918ba2d492eea75abea235";
        let unsupported_ref = "unsupported_chain_ref";

        let result = generate_p2pkh_address(pub_key.to_string(), unsupported_ref.to_string(), true);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("Unsupported Bitcoin chain reference"));
    }

    #[test]
    fn test_generate_p2wpkh_mainnet_address() {
        let mainnet_ref = "000000000019d6689c085ae165831e93";

        let result =
            generate_p2wpkh_address(PUB_KEY_COMPRESSED.to_string(), mainnet_ref.to_string());
        assert!(result.is_ok());
        let address = result.unwrap();
        assert_eq!(address, EXPECTED_P2WPKH_ADDRESS_MAINNET);
    }

    #[test]
    fn test_generate_p2wpkh_testnet_address() {
        let testnet_ref = "000000000933ea01ad0ee984209779ba";

        let result =
            generate_p2wpkh_address(PUB_KEY_COMPRESSED.to_string(), testnet_ref.to_string());
        assert!(result.is_ok());
        let address = result.unwrap();
        assert_eq!(address, EXPECTED_P2WPKH_ADDRESS_TESTNET);
    }

    #[test]
    fn test_generate_p2wpkh_with_uncompressed_key() {
        let mainnet_ref = "000000000019d6689c085ae165831e93";

        // P2WPKH should work with uncompressed input but convert to compressed
        let result =
            generate_p2wpkh_address(PUB_KEY_UNCOMPRESSED.to_string(), mainnet_ref.to_string());
        assert!(result.is_ok());
        let address = result.unwrap();
        assert_eq!(address, EXPECTED_P2WPKH_ADDRESS_MAINNET);
    }

    #[test]
    fn test_p2wpkh_invalid_chain_reference() {
        let unsupported_ref = "unsupported_chain_ref";

        let result =
            generate_p2wpkh_address(PUB_KEY_COMPRESSED.to_string(), unsupported_ref.to_string());
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("Unsupported Bitcoin chain reference"));
    }
}
