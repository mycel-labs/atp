use candid::CandidType;
use ic_stable_structures::memory_manager::MemoryId;
use ic_stable_structures::Storable;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::HashMap;

use super::nosql_db::Database;
use super::types::{CompositeKey, CompositeKeys, Document, QueryResponse};
use crate::memory::stable_memory::create_stable_btree_map;

/// DatabaseManager provides a centralized way to manage multiple models and their storage
pub struct DatabaseManager {
    registered_models: RefCell<HashMap<String, ModelInfo>>,
    next_memory_id: RefCell<u8>,
}

/// Information about a registered model
#[derive(Clone, Debug)]
struct ModelInfo {
    _model_name: String,
    primary_memory_id: MemoryId,
    secondary_memory_id: Option<MemoryId>,
}

impl DatabaseManager {
    /// Create a new DatabaseManager
    pub fn new() -> Self {
        Self {
            registered_models: RefCell::new(HashMap::new()),
            next_memory_id: RefCell::new(0),
        }
    }

    /// Register a model with the database manager
    ///
    /// # Arguments
    /// * `model_name` - Unique name for the model
    /// * `primary_memory_id` - Memory ID for primary storage (None for auto-allocation)
    /// * `secondary_memory_id` - Memory ID for secondary index (None for no secondary index)
    ///
    /// # Returns
    /// * `Result<(), String>` - Ok if registration successful, Err with message if failed
    pub fn register_model(
        &self,
        model_name: &str,
        primary_memory_id: Option<u8>,
        secondary_memory_id: Option<u8>,
    ) -> Result<(), String> {
        let mut models = self.registered_models.borrow_mut();

        if models.contains_key(model_name) {
            return Err(format!("Model '{}' is already registered", model_name));
        }

        let primary_id = match primary_memory_id {
            Some(id) => {
                // Check if this memory ID is already in use
                let memory_id = MemoryId::new(id);
                for (_, info) in models.iter() {
                    if info.primary_memory_id == memory_id {
                        return Err(format!("Memory ID {} is already in use", id));
                    }
                    if let Some(secondary_id) = info.secondary_memory_id {
                        if secondary_id == memory_id {
                            return Err(format!("Memory ID {} is already in use", id));
                        }
                    }
                }
                MemoryId::new(id)
            }
            None => {
                // Auto-allocate memory ID
                let mut next_id = self.next_memory_id.borrow_mut();
                let id = MemoryId::new(*next_id);
                *next_id += 1;
                id
            }
        };

        let secondary_id = secondary_memory_id
            .map(|id| {
                // Check if this memory ID is already in use
                let memory_id = MemoryId::new(id);
                for (_, info) in models.iter() {
                    if info.primary_memory_id == memory_id {
                        return Err(format!("Memory ID {} is already in use", id));
                    }
                    if let Some(secondary_id) = info.secondary_memory_id {
                        if secondary_id == memory_id {
                            return Err(format!("Memory ID {} is already in use", id));
                        }
                    }
                }
                Ok(memory_id)
            })
            .transpose()?;

        let model_info = ModelInfo {
            _model_name: model_name.to_string(),
            primary_memory_id: primary_id,
            secondary_memory_id: secondary_id,
        };

        models.insert(model_name.to_string(), model_info);
        Ok(())
    }

    /// Create a database instance for a registered model
    pub fn get_database<T, SecondaryKey>(
        &self,
        model_name: &str,
        get_secondary_key: Option<Box<dyn Fn(&T) -> Option<SecondaryKey>>>,
    ) -> Result<Database<T, SecondaryKey>, String>
    where
        T: CandidType + Serialize + for<'de> Deserialize<'de> + Clone,
        SecondaryKey: Storable + Ord + Clone,
    {
        let models = self.registered_models.borrow();
        let model_info = models
            .get(model_name)
            .ok_or_else(|| format!("Model '{}' is not registered", model_name))?;

        // Create primary map
        let primary_map = RefCell::new(create_stable_btree_map::<CompositeKey, Document<T>>(
            model_info.primary_memory_id,
        ));

        // Create secondary index if requested
        let secondary_index = model_info.secondary_memory_id.map(|memory_id| {
            RefCell::new(create_stable_btree_map::<SecondaryKey, CompositeKeys>(
                memory_id,
            ))
        });

        Ok(Database::new(
            primary_map,
            secondary_index,
            get_secondary_key,
        ))
    }

    /// Create a database instance without secondary index
    pub fn get_simple_database<T>(&self, model_name: &str) -> Result<Database<T>, String>
    where
        T: CandidType + Serialize + for<'de> Deserialize<'de> + Clone,
    {
        self.get_database::<T, ()>(model_name, None)
    }

    /// Insert data into a registered model's database
    pub fn insert<T>(&self, model_name: &str, key: &str, data: &T) -> Result<Document<T>, String>
    where
        T: CandidType + Serialize + for<'de> Deserialize<'de> + Clone,
    {
        let db = self.get_simple_database::<T>(model_name)?;
        db.insert(model_name.to_string(), Some(key.to_string()), data.clone())
    }

    /// Get data from a registered model's database
    pub fn get<T>(&self, model_name: &str, key: &str) -> Result<T, String>
    where
        T: CandidType + Serialize + for<'de> Deserialize<'de> + Clone,
    {
        let db = self.get_simple_database::<T>(model_name)?;
        let document = db.get(model_name, Some(key.to_string()))?;
        Ok(document.data)
    }

    /// Query data from a registered model's database
    pub fn query<T>(
        &self,
        model_name: &str,
        page_size: usize,
        page_number: usize,
    ) -> Result<QueryResponse<T>, String>
    where
        T: CandidType + Serialize + for<'de> Deserialize<'de> + Clone,
    {
        let db = self.get_simple_database::<T>(model_name)?;
        db.query(Some(model_name), None, page_size, page_number)
    }

    /// List all registered models
    pub fn list_models(&self) -> Vec<String> {
        self.registered_models.borrow().keys().cloned().collect()
    }

    /// Check if a model is registered
    pub fn is_model_registered(&self, model_name: &str) -> bool {
        self.registered_models.borrow().contains_key(model_name)
    }

    /// Get model information
    pub fn get_model_info(&self, model_name: &str) -> Option<(MemoryId, Option<MemoryId>)> {
        self.registered_models
            .borrow()
            .get(model_name)
            .map(|info| (info.primary_memory_id, info.secondary_memory_id))
    }

    /// Reserve memory IDs for a range (useful for ATP which uses 0-9)
    pub fn reserve_memory_range(
        &self,
        start: u8,
        end: u8,
        description: &str,
    ) -> Result<(), String> {
        let mut models = self.registered_models.borrow_mut();

        for id in start..=end {
            // Check if this memory ID is already in use
            let memory_id = MemoryId::new(id);
            for (_, info) in models.iter() {
                if info.primary_memory_id == memory_id {
                    return Err(format!("Memory ID {} is already in use", id));
                }
                if let Some(secondary_id) = info.secondary_memory_id {
                    if secondary_id == memory_id {
                        return Err(format!("Memory ID {} is already in use", id));
                    }
                }
            }
        }

        // Reserve the range by adding a special entry
        models.insert(
            format!("__reserved_{}_{}_{}", start, end, description),
            ModelInfo {
                _model_name: description.to_string(),
                primary_memory_id: MemoryId::new(start),
                secondary_memory_id: Some(MemoryId::new(end)),
            },
        );

        // Update next_id to skip reserved range
        let mut next_id = self.next_memory_id.borrow_mut();
        if *next_id <= end {
            *next_id = end + 1;
        }

        Ok(())
    }
}

impl Default for DatabaseManager {
    fn default() -> Self {
        Self::new()
    }
}
