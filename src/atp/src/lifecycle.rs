use ic_cdk::api::time;
use ic_cdk::{heartbeat, init, post_upgrade, pre_upgrade};

use crate::infrastructure::repositories::account_repository_impl::AccountRepositoryImpl;
use crate::infrastructure::repositories::signer_repository_impl::SignerRepositoryImpl;
use crate::utils::config::KEY_ID;

/// Initialize the canister
/// This function is called exactly once when the canister is first deployed
#[init]
fn init() {
    ic_cdk::println!("[{}] Initializing canister", time());

    // Initialize the repositories
    SignerRepositoryImpl::init(KEY_ID.to_string());
    AccountRepositoryImpl::init().expect("Failed to initialize account repository");

    ic_cdk::println!("[{}] Canister initialized successfully", time());
}

/// Pre-upgrade hook
/// This function is called before a canister upgrade to save state
#[pre_upgrade]
fn pre_upgrade() {
    ic_cdk::println!("[{}] Preparing for canister upgrade", time());

    // If you need to store any additional data during upgrade
    // that can't be preserved in stable memory directly,
    // you can save it here
    //
    // Example: let some_data = get_some_data();
    // stable_save((some_data,)).unwrap_or_else(|e| {
    //     trap(&format!("Failed to save data to stable memory: {}", e));
    // });

    ic_cdk::println!("[{}] Pre-upgrade completed", time());
}

/// Post-upgrade hook
/// This function is called after a canister upgrade to restore state
#[post_upgrade]
fn post_upgrade() {
    ic_cdk::println!("[{}] Restoring after canister upgrade", time());

    // Re-initialize the repositories
    SignerRepositoryImpl::init(KEY_ID.to_string());
    AccountRepositoryImpl::init().expect("Failed to initialize account repository");

    // If you saved any additional data in pre_upgrade, restore it here
    //
    // Example: let (some_data,): (SomeType,) = stable_restore().unwrap_or_else(|e| {
    //     trap(&format!("Failed to restore data from stable memory: {}", e));
    // });
    // restore_some_data(some_data);

    ic_cdk::println!("[{}] Post-upgrade completed successfully", time());
}

/// Heartbeat function for periodic tasks
/// This is called regularly by the IC system
#[heartbeat]
fn heartbeat() {
    // Add any periodic cleanup or maintenance tasks here
    // Example: clean_old_records();
}
