use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Curve {
    #[serde(rename = "secp256k1")]
    Secp256k1,
    #[serde(rename = "ed25519")]
    Ed25519,
}
