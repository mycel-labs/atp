//! Memory management module for database operations
//!
//! This module provides memory management utilities for IC stable memory,
//! including memory allocation, virtual memory handling, and memory manager abstractions.

pub use self::manager::MemoryManager;
pub use self::stable_memory::{create_stable_btree_map, get_memory, Memory};
pub use ic_stable_structures::memory_manager::MemoryId;

pub mod manager;
pub mod stable_memory;
