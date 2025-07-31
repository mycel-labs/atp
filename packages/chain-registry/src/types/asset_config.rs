use caip::AssetIdBase;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetConfig {
    pub asset_id_base: AssetIdBase,
    pub symbol: String,
    pub name: String,
    pub is_native: bool,
    pub decimals: u8,
    #[serde(default)]
    pub metadata: HashMap<String, toml::Value>,
}
