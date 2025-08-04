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
