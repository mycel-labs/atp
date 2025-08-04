use ic_nosql::{
    traits::{Model, Repository},
    DatabaseManager,
};
use std::cell::RefCell;

use crate::domain::models::account::Account;
use crate::domain::repositories::account_repository::IAccountRepository;

thread_local! {
    static DB_MANAGER: RefCell<Option<DatabaseManager>> = RefCell::new(None);
    static ACCOUNT_REPOSITORY: RefCell<Option<AccountRepositoryImpl>> = RefCell::new(None);
}

#[derive(Clone)]
pub struct AccountRepositoryImpl {}

impl AccountRepositoryImpl {
    pub fn new() -> Self {
        Self {}
    }

    /// Initialize the database manager and account repository
    pub fn init() -> Result<(), String> {
        // Initialize database manager
        let db_manager = DatabaseManager::new();

        // Register the Account model with secondary index for owner queries
        db_manager.register_model("accounts", Some(0), Some(1))?;

        // Store the database manager
        DB_MANAGER.with(|manager| {
            *manager.borrow_mut() = Some(db_manager);
        });

        // Initialize repository instance
        ACCOUNT_REPOSITORY.with(|repo| {
            *repo.borrow_mut() = Some(AccountRepositoryImpl::new());
        });

        Ok(())
    }

    /// Get the global account repository instance
    pub fn global() -> Self {
        ACCOUNT_REPOSITORY.with(|repo| match &*repo.borrow() {
            Some(instance) => instance.clone(),
            None => panic!(
                "AccountRepositoryImpl not initialized! Call AccountRepositoryImpl::init() first."
            ),
        })
    }

    /// Get a database instance for Account operations
    fn get_database(&self) -> Result<ic_nosql::Database<Account, String>, String> {
        DB_MANAGER.with(|manager| {
            let manager = manager.borrow();
            let db_manager = manager.as_ref().ok_or("Database manager not initialized")?;

            // Create database with secondary key function for owner queries
            db_manager.get_database(
                "accounts",
                Some(Box::new(|account: &Account| account.get_secondary_key())),
            )
        })
    }
}

impl IAccountRepository for AccountRepositoryImpl {
    fn insert(&self, account: Account) -> Result<Account, String> {
        let db = self.get_database()?;
        let document = db.insert(
            account.get_primary_key(),
            None, // No sort key for primary operations
            account.clone(),
        )?;
        Ok(document.data)
    }

    fn get(&self, id: &str) -> Result<Account, String> {
        let db = self.get_database()?;
        let document = db.get(id, None)?;
        Ok(document.data)
    }

    fn exists(&self, id: &str) -> bool {
        match self.get(id) {
            Ok(_) => true,
            Err(_) => false,
        }
    }

    fn find_by_owner(
        &self,
        owner: &str,
        page_size: usize,
        page: usize,
    ) -> Result<Vec<Account>, String> {
        let db = self.get_database()?;

        // Query using secondary index for owner
        let query_result = db.query(
            None,                    // No specific partition key
            Some(owner.to_string()), // Use owner as secondary key
            page_size,               // page size
            page,                    // page number
        )?;

        let accounts = query_result
            .results
            .into_iter()
            .map(|doc| doc.data)
            .collect();

        Ok(accounts)
    }
}

impl Repository<Account> for AccountRepositoryImpl {
    type Error = String;

    fn save(&self, model: &Account) -> Result<Account, Self::Error> {
        self.insert(model.clone())
    }

    fn find_by_id(
        &self,
        id: &<Account as Model>::PrimaryKey,
    ) -> Result<Option<Account>, Self::Error> {
        match self.get(id) {
            Ok(account) => Ok(Some(account)),
            Err(_) => Ok(None),
        }
    }

    fn find_all(&self) -> Result<Vec<Account>, Self::Error> {
        let db = self.get_database()?;
        let query_result = db.query(
            None, // No collection filter
            None, // No secondary key filter
            1000, // large page size to get all
            1,    // page number
        )?;

        let accounts = query_result
            .results
            .into_iter()
            .map(|doc| doc.data)
            .collect();

        Ok(accounts)
    }

    fn delete(&self, _id: &<Account as Model>::PrimaryKey) -> Result<bool, Self::Error> {
        // TODO: ic-nosql Database doesn't have a delete method, so we'll return false for now
        // This would need to be implemented in the ic-nosql library
        Ok(false)
    }

    fn exists(&self, id: &<Account as Model>::PrimaryKey) -> Result<bool, Self::Error> {
        Ok(IAccountRepository::exists(self, id))
    }
}

#[cfg(test)]
mod account_repository_tests {
    use candid::Principal;
    use std::rc::Rc;

    use crate::domain::models::account::{Account, AccountState};
    use crate::domain::models::signer::SignatureAlgorithm;
    use crate::domain::repositories::account_repository::IAccountRepository;
    use crate::infrastructure::repositories::account_repository_impl::AccountRepositoryImpl;
    use crate::utils::ic::api::set_ic_api;
    use crate::utils::ic::mock::MockIcApi;
    use atp_caip::curve::Curve;

    // Helper function to create a test account
    fn create_test_account(id: &str, owner: Principal) -> Account {
        // Set up the mock
        let mock_api = MockIcApi::new().with_caller(owner.clone());
        set_ic_api(Rc::new(mock_api));

        Account::new(
            id.to_string(),
            owner,
            vec![1, 2, 3], // dummy public_key
            SignatureAlgorithm::Ecdsa,
            Curve::Secp256k1,
            owner, // approved_address is the same as owner for test simplicity
        )
    }

    // Set up a clean test environment before each test
    fn setup() -> AccountRepositoryImpl {
        // Initialize the repository
        AccountRepositoryImpl::init().expect("Failed to initialize repository");
        // Create a new repository implementation
        AccountRepositoryImpl::new()
    }

    #[test]
    fn test_insert_account() {
        let repo = setup();
        let owner = Principal::from_text("2vxsx-fae").unwrap();
        let account = create_test_account("test-id-1", owner);

        // Test insertion
        let result = repo.insert(account.clone());
        assert!(
            result.is_ok(),
            "Failed to insert account: {:?}",
            result.err()
        );

        // Verify the inserted account matches the original
        let inserted_account = result.unwrap();
        assert_eq!(inserted_account.id(), account.id());
        assert_eq!(inserted_account.owner(), account.owner());
        assert_eq!(inserted_account.algorithm(), account.algorithm());
        assert_eq!(inserted_account.account_state(), account.account_state());
    }

    #[test]
    fn test_get_account() {
        let repo = setup();
        let owner = Principal::from_text("2vxsx-fae").unwrap();
        let account = create_test_account("test-id-2", owner);

        // Insert the account first
        let _ = repo
            .insert(account.clone())
            .expect("Failed to insert account");

        // Test retrieval
        let result = repo.get("test-id-2");
        assert!(result.is_ok(), "Failed to get account: {:?}", result.err());

        // Verify the retrieved account matches the original
        let retrieved_account = result.unwrap();
        assert_eq!(retrieved_account.id(), account.id());
        assert_eq!(retrieved_account.owner(), account.owner());
        assert_eq!(retrieved_account.algorithm(), account.algorithm());
    }

    #[test]
    fn test_get_nonexistent_account() {
        let repo = setup();

        // Try to get a non-existent account
        let result = repo.get("non-existent-id");
        assert!(result.is_err(), "Expected error for non-existent account");
    }

    #[test]
    fn test_exists() {
        let repo = setup();
        let owner = Principal::from_text("2vxsx-fae").unwrap();
        let account = create_test_account("test-id-3", owner);

        // Insert the account first
        let _ = repo.insert(account).expect("Failed to insert account");

        // Test exists for existing account
        assert!(repo.exists("test-id-3"), "Expected account to exist");

        // Test exists for non-existing account
        assert!(
            !repo.exists("non-existent-id"),
            "Expected account to not exist"
        );
    }

    #[test]
    fn test_find_by_owner() {
        let repo = setup();
        let owner1 = Principal::from_text("2vxsx-fae").unwrap();
        let owner2 = Principal::from_text("rrkah-fqaaa-aaaaa-aaaaq-cai").unwrap();

        // Create and insert multiple accounts for the same owner
        let accounts_owner1 = vec![
            create_test_account("test-id-4", owner1),
            create_test_account("test-id-5", owner1),
            create_test_account("test-id-6", owner1),
        ];

        // Create and insert an account for a different owner
        let account_owner2 = create_test_account("test-id-7", owner2);

        // Insert all accounts
        for account in accounts_owner1.iter() {
            let _ = repo
                .insert(account.clone())
                .expect("Failed to insert account");
        }
        let _ = repo
            .insert(account_owner2)
            .expect("Failed to insert account");

        // Test finding accounts by owner1
        let result = repo.find_by_owner(&owner1.to_string(), 100, 1);
        assert!(
            result.is_ok(),
            "Failed to find accounts by owner: {:?}",
            result.err()
        );

        let found_accounts = result.unwrap();
        assert_eq!(
            found_accounts.len(),
            3,
            "Expected to find 3 accounts for owner1"
        );

        // Verify all found accounts have the correct owner
        for account in found_accounts {
            assert_eq!(
                account.owner(),
                &owner1,
                "Found account with incorrect owner"
            );
        }

        // Test finding accounts by owner2
        let result = repo.find_by_owner(&owner2.to_string(), 100, 1);
        assert!(
            result.is_ok(),
            "Failed to find accounts by owner: {:?}",
            result.err()
        );

        let found_accounts = result.unwrap();
        assert_eq!(
            found_accounts.len(),
            1,
            "Expected to find 1 account for owner2"
        );
        assert_eq!(
            found_accounts[0].owner(),
            &owner2,
            "Found account with incorrect owner"
        );
    }

    #[test]
    fn test_find_by_nonexistent_owner() {
        let repo = setup();

        // Test finding accounts by a non-existent owner
        let result = repo.find_by_owner("non-existent-owner", 100, 1);

        // This might return an empty list or an error depending on your implementation
        // We'll test for an error, but if your implementation returns an empty list instead,
        // adjust this test accordingly
        match result {
            Ok(accounts) => assert!(
                accounts.is_empty(),
                "Expected empty list for non-existent owner"
            ),
            Err(_) => {} // Also acceptable
        }
    }

    #[test]
    fn test_update_account() {
        let repo = setup();
        let owner = Principal::from_text("2vxsx-fae").unwrap();
        let mut account = create_test_account("test-id-8", owner);

        // Insert the account first
        let _ = repo
            .insert(account.clone())
            .expect("Failed to insert account");

        // Modify the account
        // Note: We're using a simple state change as an example
        // In a real application, you might use Account methods instead
        let result = account.unlock();
        assert!(result.is_ok(), "Failed to unlock account");
        account = result.unwrap();

        // Update the account in the repository
        let update_result = repo.insert(account.clone());
        assert!(update_result.is_ok(), "Failed to update account");

        // Verify the update was successful
        let retrieved = repo.get("test-id-8").expect("Failed to get account");
        assert_eq!(
            retrieved.account_state(),
            &AccountState::Unlocked,
            "Account state not updated"
        );
    }

    #[test]
    fn test_multiple_operations() {
        let repo = setup();
        let owner = Principal::from_text("2vxsx-fae").unwrap();

        // Test a sequence of operations
        // 1. Insert
        let account = create_test_account("test-id-9", owner);
        let insert_result = repo.insert(account.clone());
        assert!(insert_result.is_ok(), "Failed to insert account");

        // 2. Verify exists
        assert!(
            repo.exists("test-id-9"),
            "Account should exist after insertion"
        );

        // 3. Get and verify
        let get_result = repo.get("test-id-9");
        assert!(get_result.is_ok(), "Failed to get account");
        let retrieved = get_result.unwrap();
        assert_eq!(
            retrieved.id(),
            account.id(),
            "Retrieved account ID doesn't match"
        );

        // 4. Find by owner
        let find_result = repo.find_by_owner(&owner.to_string(), 100, 1);
        assert!(find_result.is_ok(), "Failed to find accounts by owner");
        let found_accounts = find_result.unwrap();
        assert!(
            !found_accounts.is_empty(),
            "Should find at least one account for owner"
        );
        assert!(
            found_accounts.iter().any(|a| a.id() == account.id()),
            "Should find the inserted account"
        );
    }

    #[test]
    fn test_advanced_features() {
        let repo = setup();
        let owner = Principal::from_text("2vxsx-fae").unwrap();

        // Test that accounts are properly stored with secondary indexing
        let account1 = create_test_account("feature-test-1", owner);
        let account2 = create_test_account("feature-test-2", owner);

        // Insert both accounts
        let _ = repo
            .insert(account1.clone())
            .expect("Failed to insert account1");
        let _ = repo
            .insert(account2.clone())
            .expect("Failed to insert account2");

        // Verify both accounts can be found by owner using secondary index
        let found_accounts = repo
            .find_by_owner(&owner.to_string(), 100, 1)
            .expect("Failed to find accounts");
        assert!(
            found_accounts.len() >= 2,
            "Should find at least 2 accounts for owner"
        );

        // Verify both specific accounts exist
        let account1_retrieved = repo.get(account1.id()).expect("Failed to get account1");
        let account2_retrieved = repo.get(account2.id()).expect("Failed to get account2");

        assert_eq!(account1_retrieved.id(), account1.id());
        assert_eq!(account2_retrieved.id(), account2.id());
    }
}
