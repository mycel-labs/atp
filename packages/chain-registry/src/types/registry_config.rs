use atp_caip::TokenPair;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::types::{AssetConfig, ChainConfig};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryConfig {
    #[serde(default)]
    pub chains: HashMap<String, ChainConfig>,
    #[serde(default)]
    pub assets: HashMap<String, AssetConfig>,
    #[serde(default)]
    pub token_pairs: Vec<TokenPair>,
}

impl RegistryConfig {
    pub fn new() -> Self {
        Self {
            chains: HashMap::new(),
            assets: HashMap::new(),
            token_pairs: Vec::new(),
        }
    }
}

impl Default for RegistryConfig {
    fn default() -> Self {
        Self::new()
    }
}
