//! Test library for ATP and ic-nosql canister testing
//!
//! This library provides organized test utilities and modules for testing
//! different canisters (ATP, ic-nosql) with shared common utilities.

// Common test utilities used by all canister tests
pub mod test_utils;

// ATP canister specific testing module
pub mod atp {
    pub mod atp_test_utils;
    #[cfg(test)]
    pub mod integration_tests;
}

// pub mod ic_nosql {
// pub mod ic_nosql_test_utils;
// #[cfg(test)]
// pub mod stress_tests;
// }

// Re-export common utilities for convenience
pub use test_utils::{
    assert_performance_within_bounds, assert_success_rate, ensure_canister_built,
    get_canister_wasm_path, run_parallel_operations, PerformanceMetrics, TestConfig,
    TestDataGenerator, TestEnvironment,
};
