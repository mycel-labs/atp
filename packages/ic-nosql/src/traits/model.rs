use candid::CandidType;
use serde::{Deserialize, Serialize};

/// Trait that all database models must implement
pub trait Model: CandidType + Serialize + for<'de> Deserialize<'de> + Clone {
    /// The primary key type for this model
    type PrimaryKey: ToString + Clone;

    /// The secondary key type for this model (if any)
    type SecondaryKey: Clone + Ord + ic_stable_structures::Storable;

    /// Get the primary key for this model instance
    fn get_primary_key(&self) -> Self::PrimaryKey;

    /// Get the secondary key for this model instance (if any)
    fn get_secondary_key(&self) -> Option<Self::SecondaryKey> {
        None
    }

    /// Get the model name for database registration
    fn model_name() -> &'static str;
}
