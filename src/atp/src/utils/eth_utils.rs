use k256::elliptic_curve::sec1::ToEncodedPoint;
use k256::PublicKey;
use sha3::{Digest, Keccak256};

pub fn generate_eth_address_from_sec1(pub_key_sec1_bytes: Vec<u8>) -> Result<String, String> {
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
