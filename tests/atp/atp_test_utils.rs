//! Test utilities specific to ATP canister
//!
//! This module provides utilities for testing the ATP canister functionality
use crate::test_utils::{TestConfig, TestEnvironment};
use atp::application::dtos::account_reply::AccountReply;
use atp::application::dtos::eip1559::Eip1559TransactionRequestDTO;
use atp::domain::models::signer::{Curve, SignatureAlgorithm};
use candid::{Encode, Principal};

// Convenience function to create TestEnvironment for ATP canister
pub fn create_atp_canister_env() -> Result<TestEnvironment, Box<dyn std::error::Error>> {
    TestEnvironment::new("atp", "atp")
}

pub fn create_atp_canister_env_with_config(
    config: TestConfig,
) -> Result<TestEnvironment, Box<dyn std::error::Error>> {
    TestEnvironment::new_with_config("atp", "atp", config)
}

// Helper to create a new account
pub fn create_test_account(
    env: &TestEnvironment,
    algorithm: SignatureAlgorithm,
    curve: Curve,
    approved_address: Principal,
    caller: Principal,
) -> Result<AccountReply, Box<dyn std::error::Error>> {
    let result: Result<AccountReply, String> = env.update_call(
        "create_account",
        Encode!(&algorithm, &curve, &approved_address).unwrap(),
        Some(caller),
    )?;

    match result {
        Ok(account) => Ok(account),
        Err(e) => Err(e.into()),
    }
}

// Helper to get account details
pub fn get_account(
    env: &TestEnvironment,
    account_id: &str,
) -> Result<AccountReply, Box<dyn std::error::Error>> {
    let result: Result<AccountReply, String> =
        env.query_call("get_account", Encode!(&account_id).unwrap())?;

    match result {
        Ok(account) => Ok(account),
        Err(e) => Err(e.into()),
    }
}

// Helper to unlock an account
pub fn unlock_account(
    env: &TestEnvironment,
    account_id: &str,
    caller: Principal,
) -> Result<AccountReply, Box<dyn std::error::Error>> {
    let result: Result<AccountReply, String> = env.update_call(
        "unlock_account",
        Encode!(&account_id).unwrap(),
        Some(caller),
    )?;

    match result {
        Ok(account) => Ok(account),
        Err(e) => Err(e.into()),
    }
}

// Helper to transfer an account
pub fn transfer_account(
    env: &TestEnvironment,
    account_id: &str,
    to: Principal,
    caller: Principal,
) -> Result<AccountReply, Box<dyn std::error::Error>> {
    let result: Result<AccountReply, String> = env.update_call(
        "transfer_account",
        Encode!(&account_id, &to).unwrap(),
        Some(caller),
    )?;

    match result {
        Ok(account) => Ok(account),
        Err(e) => Err(e.into()),
    }
}

// Helper to activate an account
pub fn activate_account(
    env: &TestEnvironment,
    account_id: &str,
    caller: Principal,
) -> Result<AccountReply, Box<dyn std::error::Error>> {
    let result: Result<AccountReply, String> = env.update_call(
        "activate_account",
        Encode!(&account_id).unwrap(),
        Some(caller),
    )?;

    match result {
        Ok(account) => Ok(account),
        Err(e) => Err(e.into()),
    }
}

// Helper to sign a message
pub fn sign_message(
    env: &TestEnvironment,
    account_id: &str,
    message_hex: &str,
    caller: Principal,
) -> Result<String, Box<dyn std::error::Error>> {
    let result: Result<String, String> = env.update_call(
        "sign",
        Encode!(&account_id, &message_hex).unwrap(),
        Some(caller),
    )?;

    match result {
        Ok(signature) => Ok(signature),
        Err(e) => Err(e.into()),
    }
}

// Helper to sign EIP-1559 transaction
pub fn sign_eip1559_transaction(
    env: &TestEnvironment,
    account_id: &str,
    tx_request: Eip1559TransactionRequestDTO,
    caller: Principal,
) -> Result<String, Box<dyn std::error::Error>> {
    let result: Result<String, String> = env.update_call(
        "sign_eip1559_transaction",
        Encode!(&account_id, &tx_request).unwrap(),
        Some(caller),
    )?;

    match result {
        Ok(signature) => Ok(signature),
        Err(e) => Err(e.into()),
    }
}

// Helper to get Ethereum address
pub fn get_eth_address(
    env: &TestEnvironment,
    account_id: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let result: Result<String, String> =
        env.query_call("get_eth_address", Encode!(&account_id).unwrap())?;

    match result {
        Ok(address) => Ok(address),
        Err(e) => Err(e.into()),
    }
}

// Helper to create test EIP-1559 transaction data
pub fn create_test_eip1559_transaction() -> Eip1559TransactionRequestDTO {
    Eip1559TransactionRequestDTO {
        to: Some("0x742d35Cc9638C0532846e7a88a8020b38c4bC86E".to_string()),
        from: None,
        value: Some("1000000000000000000".to_string()), // 1 ETH in wei
        gas: Some("21000".to_string()),
        max_fee_per_gas: Some("20000000000".to_string()), // 20 gwei
        max_priority_fee_per_gas: Some("2000000000".to_string()), // 2 gwei
        nonce: Some("0".to_string()),
        chain_id: Some("1".to_string()), // Ethereum mainnet
        data: Some(vec![]),              // Empty data as Vec<u8>
    }
}
