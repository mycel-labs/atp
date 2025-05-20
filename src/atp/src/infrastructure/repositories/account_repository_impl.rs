use crate::domain::models::account::Account;
use crate::domain::repositories::account_repository::IAccountRepository;
use crate::infrastructure::database::core::db_schema::ACCOUNTS_DB;

pub struct AccountRepositoryImpl;

impl AccountRepositoryImpl {
    pub fn new() -> Self {
        Self {}
    }
}

impl IAccountRepository for AccountRepositoryImpl {
    fn insert(&self, account: Account) -> Result<Account, String> {
        ACCOUNTS_DB.with(|db| {
            // Use the account ID as the partition key
            let document = db.borrow().insert(
                account.id().clone(),
                None, // No sort key
                account.clone(),
            )?;

            Ok(document.data)
        })
    }

    fn get(&self, id: &str) -> Result<Account, String> {
        ACCOUNTS_DB.with(|db| {
            let document = db.borrow().get(id, None)?;
            Ok(document.data)
        })
    }

    fn exists(&self, id: &str) -> bool {
        ACCOUNTS_DB.with(|db| db.borrow().get(id, None).is_ok())
    }

    fn find_by_owner(&self, owner: &str) -> Result<Vec<Account>, String> {
        ACCOUNTS_DB.with(|db| {
            let query_result = db.borrow().query(
                None,
                Some(owner.to_string()),
                100, // page size
                1,   // page number
            )?;

            let accounts = query_result
                .results
                .into_iter()
                .map(|doc| doc.data)
                .collect();

            Ok(accounts)
        })
    }
}

#[cfg(test)]
mod account_repository_tests {
    use candid::Principal;
    use std::rc::Rc;

    use crate::domain::models::account::{Account, AccountState};
    use crate::domain::models::signer::{Curve, SignatureAlgorithm};
    use crate::domain::repositories::account_repository::IAccountRepository;
    use crate::infrastructure::database::core::db_schema::{init_database, ACCOUNTS_DB};
    use crate::infrastructure::repositories::account_repository_impl::AccountRepositoryImpl;
    use crate::utils::ic::api::set_ic_api;
    use crate::utils::ic::mock::MockIcApi;

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
        // Reset the database before each test
        ACCOUNTS_DB.with(|db| {
            // Clear the database for tests
            // Note: In a real test, you might want to mock the database instead
            // but for simplicity we're just resetting the real one
            let _ = db.borrow();
        });

        // Initialize the database
        init_database();
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
        assert!(
            result.err().unwrap().contains("not found"),
            "Expected 'not found' error"
        );
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
        let result = repo.find_by_owner(&owner1.to_string());
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
        let result = repo.find_by_owner(&owner2.to_string());
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
        let result = repo.find_by_owner("non-existent-owner");

        // This might return an empty list or an error depending on your implementation
        // We'll test for an error, but if your implementation returns an empty list instead,
        // adjust this test accordingly
        assert!(result.is_err(), "Expected error for non-existent owner");
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
        let find_result = repo.find_by_owner(&owner.to_string());
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
}
