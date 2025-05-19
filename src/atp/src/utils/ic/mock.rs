//! Mock implementations of IC API for testing

use candid::Principal;
use std::cell::RefCell;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use super::api::IcApi;

/// A configurable mock for the IC API
pub struct MockIcApi {
    caller: RefCell<Principal>,
    id: RefCell<Principal>,
    time: RefCell<u64>,
    logs: RefCell<Vec<String>>,
}

impl Default for MockIcApi {
    fn default() -> Self {
        Self {
            caller: RefCell::new(Principal::anonymous()),
            id: RefCell::new(
                Principal::from_text("rrkah-fqaaa-aaaaa-aaaaq-cai")
                    .unwrap_or_else(|_| Principal::anonymous()),
            ),
            time: RefCell::new(
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_nanos() as u64,
            ),
            logs: RefCell::new(Vec::new()),
        }
    }
}

impl MockIcApi {
    /// Create a new mock API with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the caller principal for testing
    pub fn with_caller(self, caller: Principal) -> Self {
        *self.caller.borrow_mut() = caller;
        self
    }

    /// Set the canister ID for testing
    pub fn with_id(self, id: Principal) -> Self {
        *self.id.borrow_mut() = id;
        self
    }

    /// Set the current time for testing
    pub fn with_time(self, time: u64) -> Self {
        *self.time.borrow_mut() = time;
        self
    }

    /// Get the logs that have been captured
    pub fn get_logs(&self) -> Vec<String> {
        self.logs.borrow().clone()
    }
}

impl IcApi for MockIcApi {
    fn caller(&self) -> Principal {
        self.caller.borrow().clone()
    }

    fn id(&self) -> Principal {
        self.id.borrow().clone()
    }

    fn time(&self) -> u64 {
        self.time.borrow().clone()
    }

    fn println(&self, message: &str) {
        self.logs.borrow_mut().push(message.to_string());
    }
}

/// A more sophisticated mock that can record and verify API calls
pub struct SpyIcApi {
    mock: MockIcApi,
    calls: RefCell<HashMap<String, usize>>,
}

impl SpyIcApi {
    /// Create a new spy API with default mock values
    pub fn new() -> Self {
        Self {
            mock: MockIcApi::default(),
            calls: RefCell::new(HashMap::new()),
        }
    }

    /// Configure the underlying mock
    pub fn with_mock(self, mock: MockIcApi) -> Self {
        Self {
            mock,
            calls: self.calls,
        }
    }

    /// Check how many times a method was called
    pub fn times_called(&self, method: &str) -> usize {
        *self.calls.borrow().get(method).unwrap_or(&0)
    }

    /// Reset call counts
    pub fn reset_calls(&self) {
        self.calls.borrow_mut().clear();
    }

    /// Record a method call
    fn record_call(&self, method: &str) {
        let mut calls = self.calls.borrow_mut();
        *calls.entry(method.to_string()).or_insert(0) += 1;
    }
}

impl IcApi for SpyIcApi {
    fn caller(&self) -> Principal {
        self.record_call("caller");
        self.mock.caller()
    }

    fn id(&self) -> Principal {
        self.record_call("id");
        self.mock.id()
    }

    fn time(&self) -> u64 {
        self.record_call("time");
        self.mock.time()
    }

    fn println(&self, message: &str) {
        self.record_call("println");
        self.mock.println(message);
    }
}
