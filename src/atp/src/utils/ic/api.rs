//! Abstraction layer for Internet Computer API calls
//! This allows for dependency injection and easier testing

use candid::Principal;
use std::cell::RefCell;
use std::rc::Rc;

/// A trait that abstracts the IC API calls needed by the application
pub trait IcApi {
    /// Get the caller's principal
    fn caller(&self) -> Principal;

    /// Get the canister's own principal
    fn id(&self) -> Principal;

    /// Get the current IC time in nanoseconds
    fn time(&self) -> u64;

    /// Print a debug message to the IC console
    fn println(&self, message: &str);
}

/// Default implementation that uses the actual ic_cdk::api
pub struct DefaultIcApi;

impl IcApi for DefaultIcApi {
    fn caller(&self) -> Principal {
        ic_cdk::api::caller()
    }

    fn id(&self) -> Principal {
        ic_cdk::api::id()
    }

    fn time(&self) -> u64 {
        ic_cdk::api::time()
    }

    fn println(&self, message: &str) {
        ic_cdk::println!("{}", message);
    }
}

thread_local! {
    static CURRENT_IC_API: RefCell<Rc<dyn IcApi>> = RefCell::new(Rc::new(DefaultIcApi));
}

/// Set a custom IC API implementation (primarily for testing)
pub fn set_ic_api(api: Rc<dyn IcApi>) {
    CURRENT_IC_API.with(|current| {
        *current.borrow_mut() = api;
    });
}

/// Get a reference to the current IC API implementation
pub fn get_ic_api() -> Rc<dyn IcApi> {
    CURRENT_IC_API.with(|current| current.borrow().clone())
}
