use atp_caip::chain_id::ChainId;

pub fn generate_address(pub_key: String, chain_id: ChainId) -> Result<String, String> {
    let address = match chain_id.namespace() {
        "eip155" => crate::eip155::address::generate_address(pub_key),
        _ => return Err(format!("Unsupported namespace: {}", chain_id.namespace())),
    };
    return address;
}
