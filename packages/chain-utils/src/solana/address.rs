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
