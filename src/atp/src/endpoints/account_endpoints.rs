use candid::Principal;
use ethers_core::types::transaction::eip1559::Eip1559TransactionRequest;
use ic_cdk::{query, update};

use crate::application::dtos::account_reply::AccountReply;
use crate::application::dtos::eip1559::Eip1559TransactionRequestDTO;
use crate::application::services::account_service::AccountService;
use crate::domain::models::signer::{Curve, SignatureAlgorithm};
use crate::infrastructure::repositories::account_repository_impl::AccountRepositoryImpl;
use crate::infrastructure::repositories::signer_repository_impl::SignerRepositoryImpl;

/*
* dfx_test_key: Only available on the local replica started by dfx.
* test_key_1: Test key available on the ICP mainnet.
* key_1: Production key available on the ICP mainnet.
*/
pub const KEY_ID: &str = "dfx_test_key";
// pub const KEY_ID: &str = "test_key_1";
// pub const KEY_ID: &str = "key_1";

// Initialize repositories for service
fn init_repositories() -> (AccountRepositoryImpl, SignerRepositoryImpl) {
    // Create repository instances
    let account_repository = AccountRepositoryImpl::new();
    let signer_repository = SignerRepositoryImpl::new(KEY_ID.to_string());

    (account_repository, signer_repository)
}

/// Create a new account with the given parameters
///
/// This function will generate a new key pair and create an account.
/// The caller will be set as the owner of the account.
#[update]
pub async fn create_account(
    algorithm: SignatureAlgorithm,
    curve: Curve,
    approved_address: Principal,
) -> Result<AccountReply, String> {
    let (account_repository, signer_repository) = init_repositories();
    let service = AccountService::new(account_repository, signer_repository);

    // Use the caller as the owner
    let owner = ic_cdk::api::caller();

    // Create the account
    let account = service
        .create_account(algorithm, curve, owner, approved_address)
        .await?;

    // Convert to DTO before returning
    Ok(service.to_account_reply(&account))
}

/// Unlock an account
///
/// Only the approved address can unlock an account.
/// The account must be in the Locked state.
#[update]
pub fn unlock_account(account_id: String) -> Result<AccountReply, String> {
    let (account_repository, signer_repository) = init_repositories();
    let service = AccountService::new(account_repository, signer_repository);

    // Unlock the account
    let account = service.unlock_account(account_id)?;

    // Convert to DTO before returning
    Ok(service.to_account_reply(&account))
}

/// Transfer an account
///
/// Only the approved address can transfer an account
/// The account must be in the Locked state.
#[update]
pub fn transfer_account(account_id: String, to: Principal) -> Result<AccountReply, String> {
    let (account_repository, signer_repository) = init_repositories();
    let service = AccountService::new(account_repository, signer_repository);

    // Transfer the account
    let account = service.transfer_account(account_id, to)?;

    // Convert to DTO before returning
    Ok(service.to_account_reply(&account))
}

/// Activate an account
///
/// Only the owner can activate an account.
/// The account must be in the Unlocked state.
#[update]
pub fn activate_account(account_id: String) -> Result<AccountReply, String> {
    let (account_repository, signer_repository) = init_repositories();
    let service = AccountService::new(account_repository, signer_repository);

    // Activate the account
    let account = service.activate_account(account_id)?;

    // Convert to DTO before returning
    Ok(service.to_account_reply(&account))
}

/// Get account details
///
/// Retrieves the details of an account by its ID.
/// Anyone can query account details.
#[query]
pub fn get_account(account_id: String) -> Result<AccountReply, String> {
    let (account_repository, signer_repository) = init_repositories();
    let service = AccountService::new(account_repository, signer_repository);

    // Get the account
    let account = service.get_account(account_id)?;

    // Convert to DTO before returning
    Ok(service.to_account_reply(&account))
}

/// Sign a message with the account's private key
///
/// Only the owner can sign messages.
/// The account must be in the Active state.
#[update]
pub async fn sign(account_id: String, message_hex: String) -> Result<String, String> {
    let (account_repository, signer_repository) = init_repositories();
    let service = AccountService::new(account_repository, signer_repository);

    // Sign the message
    service.sign(account_id, message_hex).await
}

/// Sign an EIP-1559 transaction with the account's private key
///
/// Only the owner can sign transactions.
/// The account must be in the Active state.
/// The account must use ECDSA signature algorithm and secp256k1 curve.
#[update]
pub async fn sign_eip1559_transaction(
    account_id: String,
    tx_request: Eip1559TransactionRequestDTO,
) -> Result<String, String> {
    let (account_repository, signer_repository) = init_repositories();
    let service = AccountService::new(account_repository, signer_repository);

    let tx_request = Eip1559TransactionRequest::try_from(tx_request)?;
    // Sign the transaction
    service
        .sign_eip1559_transaction(account_id, tx_request)
        .await
}

/// Get the Ethereum address derived from the account's public key
///
/// The account must use ECDSA signature algorithm and secp256k1 curve.
/// Anyone can get the Ethereum address.
#[query]
pub fn get_eth_address(account_id: String) -> Result<String, String> {
    let (account_repository, signer_repository) = init_repositories();
    let service = AccountService::new(account_repository, signer_repository);

    // Generate Ethereum address
    service.get_eth_address(account_id)
}
