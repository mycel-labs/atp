//! Utilities module providing helper functions
//!
//! This module contains utility functions for serialization, deserialization,
//! and other common operations used throughout the database system.

pub use self::serialization::{deserialize_from_bytes, serialize_to_bytes};

pub mod serialization;
