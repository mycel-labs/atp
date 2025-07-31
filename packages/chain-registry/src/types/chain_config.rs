use caip::{AssetIdBase, Curve};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainConfig {
    pub chain_id: String,
    pub name: String,
    pub native_asset: String,
    pub rpc_endpoints: Vec<String>,
    pub explorer_url: Option<String>,
    pub cryptographic_curve: Vec<Curve>,
    pub is_testnet: bool,
    pub assets: Vec<AssetIdBase>,
    #[serde(default)]
    pub metadata: HashMap<String, toml::Value>,
}
