use candid::{CandidType, Decode, Encode};
use ic_stable_structures::{storable::Bound, Storable};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

#[derive(CandidType, Clone, Serialize, Deserialize, Debug, PartialEq)]
pub enum SignatureAlgorithm {
    #[serde(rename = "ecdsa")]
    Ecdsa,
    #[serde(rename = "schnorr")]
    Schnorr,
}

impl Storable for SignatureAlgorithm {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), SignatureAlgorithm).unwrap()
    }

    const BOUND: Bound = Bound::Unbounded;
}

#[derive(CandidType, Clone, Serialize, Deserialize, Debug, PartialEq)]
pub enum Curve {
    #[serde(rename = "secp256k1")]
    Secp256k1,
    #[serde(rename = "ed25519")]
    Ed25519,
}
impl Storable for Curve {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Curve).unwrap()
    }

    const BOUND: Bound = Bound::Unbounded;
}
