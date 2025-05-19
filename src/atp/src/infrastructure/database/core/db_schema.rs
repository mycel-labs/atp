use ic_stable_structures::memory_manager::MemoryId;
use std::cell::RefCell;

use crate::domain::models::account::Account;
use crate::infrastructure::database::core::nosql_db::Database;
use crate::infrastructure::database::core::types::{CompositeKey, CompositeKeys, Document};
use crate::infrastructure::database::memory::stable_memory::create_stable_btree_map;

// Memory IDs for different tables
pub const ACCOUNTS_MEMORY_ID: MemoryId = MemoryId::new(0);
pub const ACCOUNTS_INDEX_MEMORY_ID: MemoryId = MemoryId::new(1);

// Thread local for the accounts database
thread_local! {
    pub static ACCOUNTS_DB: RefCell<Database<Account, String>> = {
        // Create primary map for accounts
        let accounts_map = RefCell::new(create_stable_btree_map::<CompositeKey, Document<Account>>(ACCOUNTS_MEMORY_ID));

        // Create secondary index by owner
        let accounts_by_owner = RefCell::new(create_stable_btree_map::<String, CompositeKeys>(ACCOUNTS_INDEX_MEMORY_ID));

        // Function to extract the owner principal as the secondary key
        let get_owner = Box::new(|account: &Account| {
            Some(account.owner().to_string())
        });

        RefCell::new(Database::new(
            accounts_map,
            Some(accounts_by_owner),
            Some(get_owner)
        ))
    };
}

/// Initialize all database tables
pub fn init_database() {
    // Initialize the stable memory for accounts
    ACCOUNTS_DB.with(|_| {});
    // Add initialization for other tables here
}
