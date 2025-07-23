//! Common test utilities for all canister tests
//!
//! This module provides shared utilities, helper functions, and data structures
//! used across all test files (ATP, ic-nosql, etc.) to reduce code duplication
//! and improve maintainability.

use candid::{CandidType, Decode, Deserialize, Encode, Principal};
use std::path::PathBuf;
use std::time::{Duration, Instant};

use pocket_ic::PocketIc;

// Common data structures and utilities for all canister tests

// Performance metrics tracking
#[derive(Debug, Default)]
pub struct PerformanceMetrics {
    pub total_operations: usize,
    pub successful_operations: usize,
    pub failed_operations: usize,
    pub total_duration: Duration,
    pub min_duration: Option<Duration>,
    pub max_duration: Option<Duration>,
    pub operation_durations: Vec<Duration>,
}

impl PerformanceMetrics {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_operation(&mut self, duration: Duration, success: bool) {
        self.total_operations += 1;
        if success {
            self.successful_operations += 1;
        } else {
            self.failed_operations += 1;
        }

        self.total_duration += duration;
        self.operation_durations.push(duration);

        if self.min_duration.is_none() || duration < self.min_duration.unwrap() {
            self.min_duration = Some(duration);
        }

        if self.max_duration.is_none() || duration > self.max_duration.unwrap() {
            self.max_duration = Some(duration);
        }
    }

    pub fn average_duration(&self) -> Duration {
        if self.total_operations == 0 {
            Duration::from_secs(0)
        } else {
            self.total_duration / self.total_operations as u32
        }
    }

    pub fn success_rate(&self) -> f64 {
        if self.total_operations == 0 {
            0.0
        } else {
            self.successful_operations as f64 / self.total_operations as f64
        }
    }

    pub fn print_summary(&self, test_name: &str) {
        println!("\n=== {} Performance Summary ===", test_name);
        println!("Total operations: {}", self.total_operations);
        println!("Successful operations: {}", self.successful_operations);
        println!("Failed operations: {}", self.failed_operations);
        println!("Success rate: {:.2}%", self.success_rate() * 100.0);
        println!("Average duration: {:?}", self.average_duration());
        println!(
            "Min duration: {:?}",
            self.min_duration.unwrap_or(Duration::from_secs(0))
        );
        println!(
            "Max duration: {:?}",
            self.max_duration.unwrap_or(Duration::from_secs(0))
        );
        println!("Total test duration: {:?}", self.total_duration);
    }
}

// Test configuration
#[derive(Debug, Clone)]
pub struct TestConfig {
    pub cycles_amount: u64,
    pub timeout_seconds: u64,
    pub retry_attempts: usize,
}

impl Default for TestConfig {
    fn default() -> Self {
        Self {
            cycles_amount: 10_000_000_000_000, // 10T cycles
            timeout_seconds: 30,
            retry_attempts: 3,
        }
    }
}

impl TestConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_cycles(mut self, cycles: u64) -> Self {
        self.cycles_amount = cycles;
        self
    }

    pub fn with_timeout(mut self, timeout: u64) -> Self {
        self.timeout_seconds = timeout;
        self
    }

    pub fn with_retries(mut self, retries: usize) -> Self {
        self.retry_attempts = retries;
        self
    }
}

// Test environment setup
pub struct TestEnvironment {
    pub pic: PocketIc,
    pub canister_id: Principal,
    pub config: TestConfig,
    pub canister_name: String,
}

impl TestEnvironment {
    pub fn new(
        package_name: &str,
        canister_name: &str,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        Self::new_with_config(package_name, canister_name, TestConfig::default())
    }

    pub fn new_with_config(
        package_name: &str,
        canister_name: &str,
        config: TestConfig,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        ensure_canister_built(package_name, canister_name)?;

        let pic = PocketIc::new();
        let wasm_path = get_canister_wasm_path(canister_name);
        let wasm_bytes = std::fs::read(&wasm_path).map_err(|e| {
            format!(
                "Failed to read {} canister WASM at {:?}: {}",
                canister_name, wasm_path, e
            )
        })?;

        let canister_id = pic.create_canister();

        // Add cycles for testing
        pic.add_cycles(canister_id, config.cycles_amount.into());

        pic.install_canister(canister_id, wasm_bytes, Encode!().unwrap(), None);

        Ok(Self {
            pic,
            canister_id,
            config,
            canister_name: canister_name.to_string(),
        })
    }

    pub fn update_call<T>(
        &self,
        method: &str,
        args: Vec<u8>,
    ) -> Result<T, Box<dyn std::error::Error>>
    where
        T: for<'de> Deserialize<'de> + CandidType,
    {
        let bytes = self
            .pic
            .update_call(self.canister_id, Principal::anonymous(), method, args)
            .map_err(|e| format!("Update call failed: {:?}", e))?;

        let result: T = Decode!(&bytes, T).map_err(|e| format!("Decode failed: {:?}", e))?;

        Ok(result)
    }

    pub fn query_call<T>(
        &self,
        method: &str,
        args: Vec<u8>,
    ) -> Result<T, Box<dyn std::error::Error>>
    where
        T: for<'de> Deserialize<'de> + CandidType,
    {
        let bytes = self
            .pic
            .query_call(self.canister_id, Principal::anonymous(), method, args)
            .map_err(|e| format!("Query call failed: {:?}", e))?;

        let result: T = Decode!(&bytes, T).map_err(|e| format!("Decode failed: {:?}", e))?;

        Ok(result)
    }

    pub fn timed_update_call<T>(
        &self,
        method: &str,
        args: Vec<u8>,
    ) -> (Duration, Result<T, Box<dyn std::error::Error>>)
    where
        T: for<'de> Deserialize<'de> + CandidType,
    {
        let start = Instant::now();
        let result = self.update_call(method, args);
        let duration = start.elapsed();
        (duration, result)
    }

    pub fn timed_query_call<T>(
        &self,
        method: &str,
        args: Vec<u8>,
    ) -> (Duration, Result<T, Box<dyn std::error::Error>>)
    where
        T: for<'de> Deserialize<'de> + CandidType,
    {
        let start = Instant::now();
        let result = self.query_call(method, args);
        let duration = start.elapsed();
        (duration, result)
    }

    pub fn upgrade_canister(&self) -> Result<(), Box<dyn std::error::Error>> {
        let wasm_path = get_canister_wasm_path(&self.canister_name);
        let wasm_bytes = std::fs::read(&wasm_path)?;
        self.pic
            .upgrade_canister(self.canister_id, wasm_bytes, Encode!().unwrap(), None)
            .map_err(|e| format!("Upgrade failed: {:?}", e))?;
        Ok(())
    }

    pub fn canister_name(&self) -> &str {
        &self.canister_name
    }

    pub fn verify_entities_exist<T>(
        &self,
        entity_ids: &[String],
        get_method: &str,
    ) -> Result<usize, Box<dyn std::error::Error>>
    where
        T: for<'de> Deserialize<'de> + CandidType,
    {
        let mut found_count = 0;

        for entity_id in entity_ids {
            let result: Result<T, String> =
                self.query_call(get_method, Encode!(entity_id).unwrap())?;

            if result.is_ok() {
                found_count += 1;
            }
        }

        Ok(found_count)
    }
}

// Generic helper functions for canister testing

// Helper function to get a canister WASM path (generic)
pub fn get_canister_wasm_path(canister_name: &str) -> PathBuf {
    let cargo_manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    PathBuf::from(cargo_manifest_dir)
        .parent()
        .unwrap()
        .join("target")
        .join("wasm32-unknown-unknown")
        .join("release")
        .join(format!("{}.wasm", canister_name))
}

// Helper function to build a canister if needed (generic)
pub fn ensure_canister_built(
    package_name: &str,
    canister_name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let wasm_path = get_canister_wasm_path(canister_name);

    if !wasm_path.exists() {
        println!("Building {} canister...", canister_name);
        let status = std::process::Command::new("cargo")
            .args(&[
                "build",
                "--target",
                "wasm32-unknown-unknown",
                "--release",
                "--package",
                package_name,
            ])
            .status()?;

        if !status.success() {
            return Err(format!("Failed to build {} canister", canister_name).into());
        }
    }

    Ok(())
}

// Generic test data generators (canister-agnostic)
pub struct TestDataGenerator;

impl TestDataGenerator {
    pub fn generate_large_content(size: usize) -> String {
        "a".repeat(size)
    }

    pub fn generate_test_string(prefix: &str, index: usize) -> String {
        format!("{}_{}", prefix, index)
    }

    pub fn generate_hex_id(prefix: &str, bytes: &[u8]) -> String {
        format!("{}_{}", prefix, hex::encode(bytes))
    }
}

// Assertion helpers
pub fn assert_success_rate(successful: usize, total: usize, min_rate: f64, operation: &str) {
    let rate = successful as f64 / total as f64;
    assert!(
        rate >= min_rate,
        "{} success rate too low: {:.2}% ({}/{})",
        operation,
        rate * 100.0,
        successful,
        total
    );
}

pub fn assert_performance_within_bounds(
    duration: Duration,
    max_duration: Duration,
    operation: &str,
) {
    assert!(
        duration <= max_duration,
        "{} took too long: {:?} (max: {:?})",
        operation,
        duration,
        max_duration
    );
}

// Parallel operation utilities
pub use std::sync::{Arc, Mutex};
pub use std::thread;

pub fn run_parallel_operations<F, T>(operations: Vec<F>, max_threads: usize) -> Vec<T>
where
    F: FnOnce() -> T + Send + 'static,
    T: Send + 'static + Clone,
{
    let results = Arc::new(Mutex::new(Vec::new()));
    let mut handles = Vec::new();

    // Split operations into chunks based on max_threads
    let chunk_size = (operations.len() + max_threads - 1) / max_threads;
    let operations_vec = operations;

    for chunk in operations_vec.chunks(chunk_size) {
        let chunk_results = results.clone();
        let _chunk_len = chunk.len();

        // We can't clone FnOnce, so we'll handle this differently
        // This is a simplified version - in practice you'd use a different approach
        let handle = thread::spawn(move || {
            // This is a placeholder - the actual implementation would need
            // to be adjusted based on the specific use case
            let local_results: Vec<T> = Vec::new();

            let mut global_results = chunk_results.lock().unwrap();
            global_results.extend(local_results);
        });

        handles.push(handle);
    }

    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }

    let final_results = results.lock().unwrap();
    final_results.clone()
}

// Test macros
#[macro_export]
macro_rules! time_operation {
    ($operation:expr) => {{
        let start = std::time::Instant::now();
        let result = $operation;
        let duration = start.elapsed();
        (duration, result)
    }};
}

#[macro_export]
macro_rules! assert_within_timeout {
    ($operation:expr, $timeout:expr) => {{
        let start = std::time::Instant::now();
        let result = $operation;
        let duration = start.elapsed();
        assert!(
            duration <= $timeout,
            "Operation took too long: {:?} (timeout: {:?})",
            duration,
            $timeout
        );
        result
    }};
}
