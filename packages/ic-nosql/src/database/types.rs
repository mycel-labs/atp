use candid::{CandidType, Decode, Encode};
use ic_stable_structures::storable::Bound;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

const MAX_VALUE_SIZE: u32 = 4096;

/// Composite key for the database, consisting of a partition key and optional sort key
#[derive(Clone, Debug, Serialize, Deserialize, CandidType, PartialEq, Eq, PartialOrd, Ord)]
pub struct CompositeKey {
    pub partition_key: String,
    pub sort_key: Option<String>,
}

impl ic_stable_structures::Storable for CompositeKey {
    fn to_bytes(&self) -> Cow<'_, [u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }

    const BOUND: Bound = Bound::Unbounded;
}

/// Collection of composite keys, used for secondary indexes
#[derive(Clone, Debug, Serialize, Deserialize, CandidType, PartialEq, Eq, PartialOrd, Ord)]
pub struct CompositeKeys(pub Vec<CompositeKey>);

impl ic_stable_structures::Storable for CompositeKeys {
    fn to_bytes(&self) -> Cow<'_, [u8]> {
        // Encode the vector of CompositeKeys as bytes
        Cow::Owned(Encode!(&self.0).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        // Decode bytes back into a vector of CompositeKeys
        CompositeKeys(Decode!(bytes.as_ref(), Vec<CompositeKey>).unwrap())
    }

    const BOUND: ic_stable_structures::storable::Bound =
        ic_stable_structures::storable::Bound::Unbounded;
}

/// Document structure containing the data and its keys
#[derive(Clone, Debug, Serialize, Deserialize, CandidType)]
pub struct Document<T> {
    pub partition_key: String,
    pub sort_key: Option<String>,
    pub data: T,
}

impl<T> ic_stable_structures::Storable for Document<T>
where
    T: Serialize + for<'de> Deserialize<'de> + CandidType,
{
    fn to_bytes(&self) -> Cow<'_, [u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }

    const BOUND: Bound = Bound::Bounded {
        max_size: MAX_VALUE_SIZE,
        is_fixed_size: false,
    };
}

/// Response structure for paginated queries
#[derive(Clone, Debug, Serialize, Deserialize, CandidType)]
pub struct QueryResponse<T> {
    pub page_number: usize,
    pub page_size: usize,
    pub total_pages: usize,
    pub results: Vec<Document<T>>,
}
