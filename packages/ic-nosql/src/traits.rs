//! Traits module providing database abstractions
//!
//! This module defines the core traits that provide abstractions for database operations,
//! model definitions, and repository patterns.

pub use self::database::{Database as DatabaseTrait, Query};
pub use self::model::Model;
pub use self::repository::Repository;

pub mod database;
pub mod model;
pub mod repository;
