//! Database module providing core database functionality
//!
//! This module contains the main database implementation, database manager,
//! and all related types for the NoSQL database system.

pub use self::manager::DatabaseManager;
pub use self::nosql_db::Database;
pub use self::types::{CompositeKey, CompositeKeys, Document, QueryResponse};

pub mod manager;
pub mod nosql_db;
pub mod types;
