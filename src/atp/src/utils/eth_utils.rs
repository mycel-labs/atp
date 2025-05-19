use k256::elliptic_curve::sec1::ToEncodedPoint;
use k256::PublicKey;
use sha3::{Digest, Keccak256};

pub fn generate_eth_address_from_sec1(pub_key_hex: String) -> Result<String, String> {
    let pub_key_bytes = match hex::decode(&pub_key_hex) {
        Ok(bytes) => bytes,
        Err(_) => return Err("Invalid hex string".to_string()),
    };

    let pub_key = match PublicKey::from_sec1_bytes(&pub_key_bytes) {
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

pub fn generate_eth_address_from_xy(x_hex: String, y_hex: String) -> Result<String, String> {
    // Step 1: Decode the x and y coordinates from hex strings
    let x_bytes = match hex::decode(&x_hex) {
        Ok(bytes) => bytes,
        Err(_) => return Err("Invalid x coordinate hex string".to_string()),
    };

    let y_bytes = match hex::decode(&y_hex) {
        Ok(bytes) => bytes,
        Err(_) => return Err("Invalid y coordinate hex string".to_string()),
    };

    // Step 2: Ensure the coordinates are 32 bytes each (expected size for SECP256k1 x and y)
    if x_bytes.len() != 32 || y_bytes.len() != 32 {
        return Err("Invalid length for x or y coordinates. Expected 32 bytes each.".to_string());
    }

    // Step 3: Construct the uncompressed public key (0x04 + x + y)
    let mut pub_key_bytes = Vec::with_capacity(65);
    pub_key_bytes.push(0x04); // 0x04 prefix for uncompressed key
    pub_key_bytes.extend_from_slice(&x_bytes);
    pub_key_bytes.extend_from_slice(&y_bytes);

    // Step 4: Hash the public key (skip the 0x04 prefix)
    let mut hasher = Keccak256::new();
    hasher.update(&pub_key_bytes[1..]); // Skip the first byte (0x04)
    let result = hasher.finalize();

    // Step 5: Extract the last 20 bytes of the hash to form the Ethereum address
    let mut eth_address = [0u8; 20];
    eth_address.copy_from_slice(&result[12..]);

    // Return the Ethereum address in hex format
    Ok(format!("0x{}", hex::encode(eth_address)))
}

pub fn sha256(input: &String) -> [u8; 32] {
    let mut hasher = Keccak256::new();
    hasher.update(input.as_bytes());
    hasher.finalize().into()
}

pub fn verify_ecdsa_signature(
    public_key_sec1_hex: String,
    message_hash_hex: String,
    signature_hex: String,
) -> Result<bool, String> {
    // Decode message hash from hex
    let message_hash = hex::decode(&message_hash_hex)
        .map_err(|e| format!("Failed to hex-decode message hash: {}", e))?;

    // Decode signature from hex
    let signature_bytes = hex::decode(&signature_hex)
        .map_err(|e| format!("Failed to hex-decode signature: {}", e))?;

    // Decode public key from hex
    let pubkey_bytes = hex::decode(&public_key_sec1_hex)
        .map_err(|e| format!("Failed to hex-decode public key: {}", e))?;

    use k256::ecdsa::signature::Verifier;

    // Ensure signature is the correct length and take only r,s values
    let signature_bytes = if signature_bytes.len() >= 64 {
        &signature_bytes[..64]
    } else {
        return Err(format!(
            "Invalid signature length: {} bytes (expected at least 64)",
            signature_bytes.len()
        ));
    };

    // Parse signature
    let signature = k256::ecdsa::Signature::try_from(signature_bytes)
        .map_err(|e| format!("Failed to deserialize signature: {}", e))?;

    // Parse public key
    let verifying_key = k256::ecdsa::VerifyingKey::from_sec1_bytes(&pubkey_bytes)
        .map_err(|e| format!("Failed to deserialize sec1 encoding into public key: {}", e))?;

    // Verify signature against the provided message hash
    let verify_result = verifying_key.verify(&message_hash, &signature);
    Ok(verify_result.is_ok())
}
