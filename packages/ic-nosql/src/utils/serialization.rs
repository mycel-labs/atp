use candid::{CandidType, Decode, Encode};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

/// Serialize data to bytes using Candid encoding
pub fn serialize_to_bytes<T>(data: &T) -> Result<Vec<u8>, String>
where
    T: CandidType + Serialize,
{
    Encode!(data).map_err(|e| format!("Serialization error: {}", e))
}

/// Deserialize data from bytes using Candid decoding
pub fn deserialize_from_bytes<T>(bytes: &[u8]) -> Result<T, String>
where
    T: CandidType + for<'de> Deserialize<'de>,
{
    Decode!(bytes, T).map_err(|e| format!("Deserialization error: {}", e))
}

/// Convert data to storable bytes (Cow<[u8]>)
pub fn to_storable_bytes<T>(data: &T) -> Cow<'_, [u8]>
where
    T: CandidType + Serialize,
{
    Cow::Owned(Encode!(data).unwrap())
}

/// Convert storable bytes back to data
pub fn from_storable_bytes<T>(bytes: Cow<[u8]>) -> T
where
    T: CandidType + for<'de> Deserialize<'de>,
{
    Decode!(bytes.as_ref(), T).unwrap()
}
