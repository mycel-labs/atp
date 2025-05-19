use ic_stable_structures::{
    memory_manager::{MemoryId, MemoryManager, VirtualMemory},
    DefaultMemoryImpl, StableBTreeMap,
};
use std::cell::RefCell;

/// Type alias for memory used in the database
pub type Memory = VirtualMemory<DefaultMemoryImpl>;

thread_local! {
    pub static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> =
        RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));
}

/// Get a virtual memory segment with the specified ID
pub fn get_memory(id: MemoryId) -> Memory {
    MEMORY_MANAGER.with(|m| m.borrow().get(id))
}

/// Create a new stable BTree map with the specified memory ID
pub fn create_stable_btree_map<K, V>(memory_id: MemoryId) -> StableBTreeMap<K, V, Memory>
where
    K: Ord + Clone + ic_stable_structures::Storable,
    V: Clone + ic_stable_structures::Storable,
{
    let memory = get_memory(memory_id);
    StableBTreeMap::init(memory)
}
