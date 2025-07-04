use ic_stable_structures::memory_manager::{MemoryId, MemoryManager as IcMemoryManager};
use ic_stable_structures::DefaultMemoryImpl;
use std::cell::RefCell;
use std::collections::HashMap;

use super::stable_memory::Memory;

/// Memory manager providing centralized memory allocation and management
pub struct MemoryManager {
    inner: RefCell<IcMemoryManager<DefaultMemoryImpl>>,
    allocations: RefCell<HashMap<String, (MemoryId, Option<MemoryId>)>>,
    next_id: RefCell<u8>,
}

impl MemoryManager {
    /// Create a new memory manager
    pub fn new() -> Self {
        Self {
            inner: RefCell::new(IcMemoryManager::init(DefaultMemoryImpl::default())),
            allocations: RefCell::new(HashMap::new()),
            next_id: RefCell::new(0),
        }
    }

    /// Allocate memory for a model
    ///
    /// # Arguments
    /// * `model_name` - Name of the model
    /// * `primary_id` - Optional specific memory ID for primary storage
    /// * `secondary_id` - Optional specific memory ID for secondary index
    ///
    /// # Returns
    /// * `Result<(MemoryId, Option<MemoryId>), String>` - Primary and optional secondary memory IDs
    pub fn allocate_memory(
        &self,
        model_name: &str,
        primary_id: Option<u8>,
        secondary_id: Option<u8>,
    ) -> Result<(MemoryId, Option<MemoryId>), String> {
        let mut allocations = self.allocations.borrow_mut();

        if allocations.contains_key(model_name) {
            return Err(format!(
                "Memory already allocated for model '{}'",
                model_name
            ));
        }

        let primary_memory_id = match primary_id {
            Some(id) => {
                self.check_id_available(id, &allocations)?;
                MemoryId::new(id)
            }
            None => {
                let mut next_id = self.next_id.borrow_mut();
                while self.is_id_in_use(*next_id, &allocations) {
                    *next_id += 1;
                }
                let id = MemoryId::new(*next_id);
                *next_id += 1;
                id
            }
        };

        let secondary_memory_id = match secondary_id {
            Some(id) => {
                self.check_id_available(id, &allocations)?;
                Some(MemoryId::new(id))
            }
            None => None,
        };

        allocations.insert(
            model_name.to_string(),
            (primary_memory_id, secondary_memory_id),
        );

        Ok((primary_memory_id, secondary_memory_id))
    }

    /// Get memory for a specific memory ID
    pub fn get_memory(&self, id: MemoryId) -> Memory {
        self.inner.borrow().get(id)
    }

    /// Get allocated memory IDs for a model
    pub fn get_model_memory(&self, model_name: &str) -> Option<(MemoryId, Option<MemoryId>)> {
        self.allocations.borrow().get(model_name).cloned()
    }

    /// List all allocated models
    pub fn list_allocated_models(&self) -> Vec<String> {
        self.allocations.borrow().keys().cloned().collect()
    }

    /// Check if a memory ID is available
    fn check_id_available(
        &self,
        id: u8,
        allocations: &HashMap<String, (MemoryId, Option<MemoryId>)>,
    ) -> Result<(), String> {
        if self.is_id_in_use(id, allocations) {
            return Err(format!("Memory ID {} is already in use", id));
        }
        Ok(())
    }

    /// Check if a memory ID is in use
    fn is_id_in_use(
        &self,
        id: u8,
        allocations: &HashMap<String, (MemoryId, Option<MemoryId>)>,
    ) -> bool {
        let memory_id = MemoryId::new(id);
        for (_, (primary_id, secondary_id)) in allocations.iter() {
            if *primary_id == memory_id {
                return true;
            }
            if let Some(secondary_id) = secondary_id {
                if *secondary_id == memory_id {
                    return true;
                }
            }
        }
        false
    }

    /// Reserve memory IDs for a range (useful for ATP which uses 0-9)
    pub fn reserve_memory_range(
        &self,
        start: u8,
        end: u8,
        _description: &str,
    ) -> Result<(), String> {
        let mut allocations = self.allocations.borrow_mut();

        for id in start..=end {
            if self.is_id_in_use(id, &allocations) {
                return Err(format!("Memory ID {} is already in use", id));
            }
        }

        // Reserve the range by adding a special entry
        allocations.insert(
            format!("__reserved_{}_{}", start, end),
            (MemoryId::new(start), Some(MemoryId::new(end))),
        );

        // Update next_id to skip reserved range
        let mut next_id = self.next_id.borrow_mut();
        if *next_id <= end {
            *next_id = end + 1;
        }

        Ok(())
    }
}

impl Default for MemoryManager {
    fn default() -> Self {
        Self::new()
    }
}
