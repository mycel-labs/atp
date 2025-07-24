use candid::CandidType;
use serde::{Deserialize, Serialize};

use crate::database::types::{Document, QueryResponse};

/// Database trait providing core database operations
pub trait Database<T>
where
    T: CandidType + Serialize + for<'de> Deserialize<'de> + Clone,
{
    type Error;

    /// Insert or update a document
    fn insert(
        &self,
        partition_key: String,
        sort_key: Option<String>,
        data: T,
    ) -> Result<Document<T>, Self::Error>;

    /// Get a document by its keys
    fn get(
        &self,
        partition_key: &str,
        sort_key: Option<String>,
    ) -> Result<Option<Document<T>>, Self::Error>;

    /// Query documents with pagination
    fn query(
        &self,
        partition_key: Option<&str>,
        page_size: usize,
        page_number: usize,
    ) -> Result<QueryResponse<T>, Self::Error>;
}

/// Query trait for building database queries
pub trait Query<T> {
    type Error;

    /// Filter by partition key
    fn filter_by_partition_key(self, key: &str) -> Self;

    /// Set page size for pagination
    fn page_size(self, size: usize) -> Self;

    /// Set page number for pagination
    fn page_number(self, number: usize) -> Self;

    /// Execute the query
    fn execute(self) -> Result<QueryResponse<T>, Self::Error>;
}
