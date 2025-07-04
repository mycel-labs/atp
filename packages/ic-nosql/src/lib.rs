//! # IC-NoSQL: A flexible NoSQL database library for Internet Computer canisters
//!
//! This library provides a comprehensive NoSQL database solution built on top of
//! IC's stable memory structures, with support for multiple models, secondary indexes,
//! flexible memory management, and type-safe operations.
//!
//! ## Features
//!
//! - **Multiple Model Support**: Register and manage different data models in a single canister
//! - **Secondary Indexes**: Optional secondary indexes for efficient querying
//! - **Memory Management**: Automatic memory allocation with conflict prevention
//! - **Type Safety**: Compile-time type checking for all database operations
//! - **Pagination**: Built-in pagination support for large result sets
//! - **Macros**: Easy model definition with the `define_model!` macro
//!
//! ## Quick Start
//!
//! ```rust
//! use ic_nosql::{DatabaseManager, CandidType, Deserialize, Serialize};
//!
//! // Define a model
//! #[derive(Debug, Clone, CandidType, Deserialize, Serialize)]
//! pub struct User {
//!     pub id: String,
//!     pub username: String,
//!     pub email: String,
//! }
//!
//! // Initialize database manager
//! let db_manager = DatabaseManager::new();
//!
//! // Register the model
//! db_manager.register_model("users", None, None).unwrap();
//!
//! // Use the database
//! let user = User {
//!     id: "1".to_string(),
//!     username: "alice".to_string(),
//!     email: "alice@example.com".to_string()
//! };
//!
//! db_manager.insert("users", "1", &user).unwrap();
//! let retrieved_user = db_manager.get::<User>("users", "1").unwrap();
//! ```

// Database core functionality
pub mod database;
pub mod macros;
pub mod memory;
pub mod traits;
pub mod utils;

// Re-export core types and functionality for easy access
pub use database::{Database, DatabaseManager};
pub use memory::{MemoryId, MemoryManager};
pub use traits::{Model, Query, Repository};
//pub use macros::define_model;

// Re-export commonly used external types
pub use candid::{CandidType, Deserialize};
pub use ic_stable_structures::Storable;
pub use serde::Serialize;
