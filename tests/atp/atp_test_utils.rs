//! Test utilities specific to ATP canister
//!
//! This module provides utilities for testing the ATP canister functionality
use crate::test_utils::{TestConfig, TestEnvironment};
use atp::application::dtos::account_messages::*;
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
) -> Result<CreateAccountResponse, Box<dyn std::error::Error>> {
    let request = CreateAccountRequest {
        algorithm,
        curve,
        approved_address,
    };

    let result: Result<CreateAccountResponse, String> =
        env.update_call("create_account", Encode!(&request).unwrap(), Some(caller))?;

    match result {
        Ok(response) => Ok(response),
        Err(e) => Err(e.into()),
    }
}

// Helper to get account details
pub fn get_account(
    env: &TestEnvironment,
    account_id: &str,
) -> Result<GetAccountResponse, Box<dyn std::error::Error>> {
    let request = GetAccountRequest {
        account_id: account_id.to_string(),
    };

    let result: Result<GetAccountResponse, String> =
        env.query_call("get_account", Encode!(&request).unwrap())?;

    match result {
        Ok(response) => Ok(response),
        Err(e) => Err(e.into()),
    }
}

// Helper to unlock an account
pub fn unlock_account(
    env: &TestEnvironment,
    account_id: &str,
    caller: Principal,
) -> Result<UnlockAccountResponse, Box<dyn std::error::Error>> {
    let request = UnlockAccountRequest {
        account_id: account_id.to_string(),
    };

    let result: Result<UnlockAccountResponse, String> =
        env.update_call("unlock_account", Encode!(&request).unwrap(), Some(caller))?;

    match result {
        Ok(response) => Ok(response),
        Err(e) => Err(e.into()),
    }
}

// Helper to transfer an account
pub fn transfer_account(
    env: &TestEnvironment,
    account_id: &str,
    to: Principal,
    caller: Principal,
) -> Result<TransferAccountResponse, Box<dyn std::error::Error>> {
    let request = TransferAccountRequest {
        account_id: account_id.to_string(),
        to,
    };

    let result: Result<TransferAccountResponse, String> =
        env.update_call("transfer_account", Encode!(&request).unwrap(), Some(caller))?;

    match result {
        Ok(response) => Ok(response),
        Err(e) => Err(e.into()),
    }
}

// Helper to activate an account
pub fn activate_account(
    env: &TestEnvironment,
    account_id: &str,
    caller: Principal,
) -> Result<ActivateAccountResponse, Box<dyn std::error::Error>> {
    let request = ActivateAccountRequest {
        account_id: account_id.to_string(),
    };

    let result: Result<ActivateAccountResponse, String> =
        env.update_call("activate_account", Encode!(&request).unwrap(), Some(caller))?;

    match result {
        Ok(response) => Ok(response),
        Err(e) => Err(e.into()),
    }
}

// Helper to sign a message
pub fn sign_message(
    env: &TestEnvironment,
    account_id: &str,
    message_hex: &str,
    caller: Principal,
) -> Result<SignResponse, Box<dyn std::error::Error>> {
    let request = SignRequest {
        account_id: account_id.to_string(),
        message_hex: message_hex.to_string(),
    };

    let result: Result<SignResponse, String> =
        env.update_call("sign", Encode!(&request).unwrap(), Some(caller))?;

    match result {
        Ok(response) => Ok(response),
        Err(e) => Err(e.into()),
    }
}

// Helper to sign EIP-1559 transaction
pub fn sign_eip1559_transaction(
    env: &TestEnvironment,
    account_id: &str,
    tx_request: Eip1559TransactionRequestDTO,
    caller: Principal,
) -> Result<SignEip1559TransactionResponse, Box<dyn std::error::Error>> {
    let request = SignEip1559TransactionRequest {
        account_id: account_id.to_string(),
        tx_request,
    };

    let result: Result<SignEip1559TransactionResponse, String> = env.update_call(
        "sign_eip1559_transaction",
        Encode!(&request).unwrap(),
        Some(caller),
    )?;

    match result {
        Ok(response) => Ok(response),
        Err(e) => Err(e.into()),
    }
}

// Helper to get Ethereum address
pub fn get_eth_address(
    env: &TestEnvironment,
    account_id: &str,
) -> Result<GetEthAddressResponse, Box<dyn std::error::Error>> {
    let request = GetEthAddressRequest {
        account_id: account_id.to_string(),
    };

    let result: Result<GetEthAddressResponse, String> =
        env.query_call("get_eth_address", Encode!(&request).unwrap())?;

    match result {
        Ok(response) => Ok(response),
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
