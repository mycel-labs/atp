use candid::CandidType;
use ic_stable_structures::{StableBTreeMap, Storable};
use serde::{Deserialize, Serialize};
use std::cell::RefCell;

use super::types::{CompositeKey, CompositeKeys, Document, QueryResponse};
use crate::memory::stable_memory::Memory;

/// Database implementation with support for primary and secondary indexes
pub struct Database<T, SecondaryKey = ()>
where
    T: CandidType + Serialize + for<'de> Deserialize<'de> + Clone,
    SecondaryKey: Clone + Ord + Storable,
{
    map: RefCell<StableBTreeMap<CompositeKey, Document<T>, Memory>>, // Primary map
    secondary_index: Option<RefCell<StableBTreeMap<SecondaryKey, CompositeKeys, Memory>>>, // Optional secondary index
    get_secondary_key: Option<Box<dyn Fn(&T) -> Option<SecondaryKey>>>, // Function to derive secondary index key
}

impl<T, SecondaryKey> Database<T, SecondaryKey>
where
    T: CandidType + Serialize + for<'de> Deserialize<'de> + Clone,
    SecondaryKey: Storable + Ord + Clone,
{
    /// Constructor to initialize the NoSQL database with an optional secondary index
    pub fn new(
        map: RefCell<StableBTreeMap<CompositeKey, Document<T>, Memory>>,
        secondary_index: Option<RefCell<StableBTreeMap<SecondaryKey, CompositeKeys, Memory>>>,
        get_secondary_key: Option<Box<dyn Fn(&T) -> Option<SecondaryKey>>>,
    ) -> Self {
        Database {
            map,
            secondary_index,
            get_secondary_key,
        }
    }

    /// Insert a document and update secondary index if applicable
    pub fn insert(
        &self,
        partition_key: String,
        sort_key: Option<String>,
        data: T,
    ) -> Result<Document<T>, String> {
        let document = Document {
            partition_key: partition_key.clone(),
            sort_key: sort_key.clone(),
            data: data.clone(),
        };

        let key = CompositeKey {
            partition_key: partition_key.clone(),
            sort_key: sort_key.clone(),
        };

        // To avoid borrowing conflicts, check and remove existing documents in separate scopes
        let old_secondary_key = {
            // Check if the key already exists in the primary map
            if let Some(existing_document) = self.map.borrow().get(&key) {
                // If the document exists, check if there is an old secondary key
                if let (Some(_), Some(get_secondary_key)) =
                    (&self.secondary_index, &self.get_secondary_key)
                {
                    get_secondary_key(&existing_document.data)
                } else {
                    None
                }
            } else {
                None
            }
        };

        // If an old secondary key exists, update the secondary index
        if let Some(old_key) = old_secondary_key {
            if let (Some(secondary_index), _) = (&self.secondary_index, &self.get_secondary_key) {
                let mut index_map = secondary_index.borrow_mut();
                if let Some(mut composite_keys) = index_map.get(&old_key) {
                    // Remove the stale key from the secondary index
                    composite_keys.0.retain(|k| k != &key);

                    // If no keys remain, remove the secondary key entry
                    if composite_keys.0.is_empty() {
                        index_map.remove(&old_key);
                    } else {
                        index_map.insert(old_key, composite_keys);
                    }
                }
            }
        }

        // Remove the old document
        {
            let mut map = self.map.borrow_mut();
            // Explicitly remove the old document from the primary map to free memory
            map.remove(&key);
            // Insert into the primary map
            map.insert(key.clone(), document.clone());
        }

        // Update the secondary index if a key function is provided
        if let (Some(secondary_index), Some(get_secondary_key)) =
            (&self.secondary_index, &self.get_secondary_key)
        {
            if let Some(new_secondary_key) = get_secondary_key(&data) {
                let mut index_map = secondary_index.borrow_mut();

                // Check if the secondary key already exists
                if let Some(mut composite_keys) = index_map.get(&new_secondary_key) {
                    // If it exists, append the new key
                    composite_keys.0.push(key.clone());
                    index_map.insert(new_secondary_key.clone(), composite_keys);
                } else {
                    // If it doesn't exist, create a new CompositeKeys entry
                    index_map.insert(new_secondary_key, CompositeKeys(vec![key.clone()]));
                }
            }
        }

        Ok(document)
    }

    /// Get a single document by partition key and optional sort key
    pub fn get(
        &self,
        partition_key: &str,
        sort_key: Option<String>,
    ) -> Result<Document<T>, String> {
        let key = CompositeKey {
            partition_key: partition_key.to_string(),
            sort_key,
        };

        // Attempt to retrieve the document from the primary map
        self.map
            .borrow()
            .get(&key)
            .ok_or("Document not found.".to_string())
    }

    /// Query by either partition key or secondary index with pagination
    pub fn query(
        &self,
        partition_key: Option<&str>,
        secondary_key: Option<SecondaryKey>,
        page_size: usize,
        page_number: usize,
    ) -> Result<QueryResponse<T>, String> {
        // Validate page params
        if page_size == 0 {
            return Err("Page size must be greater than 0.".to_string());
        }
        if page_number == 0 {
            return Err("Page number must be greater than 0.".to_string());
        }

        match (partition_key, secondary_key) {
            (Some(partition_key), None) => {
                self.query_by_partition_key(partition_key, page_size, page_number)
            }
            (None, Some(secondary_key)) => {
                self.query_by_secondary_key(secondary_key, page_size, page_number)
            }
            (Some(partition_key), Some(secondary_key)) => self
                .query_by_partition_and_secondary_key(
                    partition_key,
                    secondary_key,
                    page_size,
                    page_number,
                ),
            (None, None) => {
                Err("At least one of partition key or secondary key must be provided.".to_string())
            }
        }
    }

    // Helper method for querying by partition key
    fn query_by_partition_key(
        &self,
        partition_key: &str,
        page_size: usize,
        page_number: usize,
    ) -> Result<QueryResponse<T>, String> {
        // Get all entries from the primary map
        let map = self.map.borrow();

        // Create a range for the given partition key to find all matching entries
        let range_start = CompositeKey {
            partition_key: partition_key.to_string(),
            sort_key: None,
        };
        let range_end = CompositeKey {
            partition_key: partition_key.to_string(),
            sort_key: Some(String::from("\u{10FFFF}")), // Maximum Unicode value as range end
        };

        // Collect matching documents within the range
        let matching_documents: Vec<Document<T>> = map
            .range(range_start..=range_end)
            .map(|(_, doc)| doc.clone())
            .collect();

        // Check if any documents were found
        if matching_documents.is_empty() {
            return Err(format!(
                "No documents found for partition key '{}'",
                partition_key
            ));
        }

        // Apply pagination
        let start_index = (page_number - 1) * page_size;

        // Check if the requested page exists
        if start_index >= matching_documents.len() {
            return Err(format!(
                "Page {} does not exist. Total documents: {}, Page size: {}",
                page_number,
                matching_documents.len(),
                page_size
            ));
        }

        // Calculate total pages
        let total_pages = (matching_documents.len() + page_size - 1) / page_size;

        // Return the paginated subset
        Ok(QueryResponse {
            page_number,
            page_size,
            total_pages,
            results: matching_documents
                .into_iter()
                .skip(start_index)
                .take(page_size)
                .collect(),
        })
    }

    // Helper method for querying by secondary key
    fn query_by_secondary_key(
        &self,
        secondary_key: SecondaryKey,
        page_size: usize,
        page_number: usize,
    ) -> Result<QueryResponse<T>, String> {
        // Query by secondary key
        let secondary_index = match &self.secondary_index {
            Some(index) => index,
            None => return Err("Secondary index not configured.".to_string()),
        };

        // Retrieve keys matching the secondary index
        let keys = secondary_index
            .borrow()
            .get(&secondary_key)
            .ok_or("No entries found for the given secondary key.".to_string())?;

        // Get all matching documents
        let matching_documents: Vec<Document<T>> = keys
            .0
            .iter()
            .filter_map(|key| self.map.borrow().get(key))
            .collect();

        // Calculate pagination indices
        let start_index = (page_number - 1) * page_size;

        // Check if the requested page exists
        if start_index >= matching_documents.len() {
            return Err(format!(
                "Page {} does not exist. Total documents: {}, Page size: {}",
                page_number,
                matching_documents.len(),
                page_size
            ));
        }

        // Calculate total pages
        let total_pages = (matching_documents.len() + page_size - 1) / page_size;

        // Return the paginated subset
        Ok(QueryResponse {
            page_number,
            page_size,
            total_pages,
            results: matching_documents
                .into_iter()
                .skip(start_index)
                .take(page_size)
                .collect(),
        })
    }

    // Helper method for querying by both partition key and secondary key
    fn query_by_partition_and_secondary_key(
        &self,
        partition_key: &str,
        secondary_key: SecondaryKey,
        page_size: usize,
        page_number: usize,
    ) -> Result<QueryResponse<T>, String> {
        // Query by both partition key and secondary key
        let secondary_index = match &self.secondary_index {
            Some(index) => index,
            None => return Err("Secondary index not configured.".to_string()),
        };

        let keys = secondary_index
            .borrow()
            .get(&secondary_key)
            .ok_or_else(|| {
                format!(
                    "No entries found for partition key '{}' and secondary key.",
                    partition_key
                )
            })?;

        // Get all matching documents filtered by partition key
        let matching_documents: Vec<Document<T>> = keys
            .0
            .iter()
            .filter(|key| key.partition_key == partition_key)
            .filter_map(|key| self.map.borrow().get(key))
            .collect();

        if matching_documents.is_empty() {
            return Err(format!(
                "No entries found for partition key '{}' and secondary key.",
                partition_key
            ));
        }

        // Calculate pagination indices
        let start_index = (page_number - 1) * page_size;

        // Check if the requested page exists
        if start_index >= matching_documents.len() {
            return Err(format!(
                "Page {} does not exist. Total documents: {}, Page size: {}",
                page_number,
                matching_documents.len(),
                page_size
            ));
        }

        // Calculate total pages
        let total_pages = (matching_documents.len() + page_size - 1) / page_size;

        // Return the paginated subset
        Ok(QueryResponse {
            page_number,
            page_size,
            total_pages,
            results: matching_documents
                .into_iter()
                .skip(start_index)
                .take(page_size)
                .collect(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use candid::Principal;
    use candid::{CandidType, Decode, Encode};
    use ic_stable_structures::memory_manager::{MemoryId, MemoryManager};
    use ic_stable_structures::{storable::Bound, DefaultMemoryImpl, StableBTreeMap, Storable};
    use std::borrow::Cow;
    use std::cell::RefCell;

    // Define a sample Account struct for testing
    #[derive(Clone, Debug, Serialize, Deserialize, CandidType, PartialEq, Eq, PartialOrd, Ord)]
    pub enum AccountStatus {
        Active,
        Inactive,
        Suspended,
    }

    impl Storable for AccountStatus {
        fn to_bytes(&self) -> Cow<[u8]> {
            Cow::Owned(Encode!(self).unwrap())
        }

        fn from_bytes(bytes: Cow<[u8]>) -> Self {
            Decode!(bytes.as_ref(), AccountStatus).unwrap()
        }

        const BOUND: Bound = Bound::Unbounded;
    }

    #[derive(Clone, Debug, Serialize, Deserialize, CandidType, PartialEq)]
    pub struct TestAccountStruct {
        pub id: String,
        pub owner: Principal,
        pub balance: u64,
        pub status: AccountStatus,
    }

    thread_local! {
        static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> =
            RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));
    }

    // Helper function to create a Database instance without a secondary index
    fn create_test_db() -> Database<TestAccountStruct> {
        let map = RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0))),
        ));
        Database::new(map, None, None)
    }

    // Helper function to create a Database instance with a secondary index on the `status` field
    fn create_test_db_with_secondary_index() -> Database<TestAccountStruct, AccountStatus> {
        let map = RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0))),
        ));
        let secondary_index = RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1))),
        ));

        let get_secondary_key =
            Box::new(|account: &TestAccountStruct| Some(account.status.clone()));

        Database::new(map, Some(secondary_index), Some(get_secondary_key))
    }

    #[test]
    fn test_insert_and_get_document() {
        let db = create_test_db();

        let account = TestAccountStruct {
            id: "1".to_string(),
            owner: Principal::anonymous(),
            balance: 1000,
            status: AccountStatus::Active,
        };

        let result = db.insert("user_1".to_string(), Some("1".to_string()), account.clone());
        assert!(result.is_ok());

        let retrieved = db.get("user_1", Some("1".to_string()));
        assert!(retrieved.is_ok());

        let retrieved_account = retrieved.unwrap();
        assert_eq!(retrieved_account.data, account);
    }

    #[test]
    fn test_query_by_secondary_key() {
        let db = create_test_db_with_secondary_index();

        let accounts = vec![
            TestAccountStruct {
                id: "1".to_string(),
                owner: Principal::anonymous(),
                balance: 1000,
                status: AccountStatus::Active,
            },
            TestAccountStruct {
                id: "2".to_string(),
                owner: Principal::anonymous(),
                balance: 2000,
                status: AccountStatus::Inactive,
            },
        ];

        for account in &accounts {
            db.insert(
                "user".to_string(),
                Some(account.id.clone()),
                account.clone(),
            )
            .unwrap();
        }

        let results = db
            .query(None, Some(AccountStatus::Active), 10, 1)
            .unwrap()
            .results;
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].data.id, "1");
    }
}
