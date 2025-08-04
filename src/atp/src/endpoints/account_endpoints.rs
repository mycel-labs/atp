use ic_cdk::{query, update};

use crate::application::dtos::account_messages::*;
use crate::application::services::account_service::AccountService;
use crate::infrastructure::repositories::account_repository_impl::AccountRepositoryImpl;
use crate::infrastructure::repositories::signer_repository_impl::SignerRepositoryImpl;

// Initialize repositories for service
fn get_repositories() -> (AccountRepositoryImpl, SignerRepositoryImpl) {
    // Create repository instances
    let account_repository = AccountRepositoryImpl::global();
    let signer_repository = SignerRepositoryImpl::global();
    (account_repository, signer_repository)
}

/// Create a new account with the given parameters
///
/// This function will generate a new key pair and create an account.
/// The caller will be set as the owner of the account.
#[update]
pub async fn create_account(
    request: CreateAccountRequest,
) -> Result<CreateAccountResponse, String> {
    let (account_repository, signer_repository) = get_repositories();
    let service = AccountService::new(account_repository, signer_repository);

    // Use the caller as the owner
    let owner = ic_cdk::api::caller();

    // Create the account
    service.create_account(request, owner).await
}

/// Unlock an account
///
/// Only the approved address can unlock an account.
/// The account must be in the Locked state.
#[update]
pub fn unlock_account(request: UnlockAccountRequest) -> Result<UnlockAccountResponse, String> {
    let (account_repository, signer_repository) = get_repositories();
    let service = AccountService::new(account_repository, signer_repository);

    // Unlock the account
    service.unlock_account(request)
}

/// Transfer an account
///
/// Only the approved address can transfer an account
/// The account must be in the Locked state.
#[update]
pub fn transfer_account(
    request: TransferAccountRequest,
) -> Result<TransferAccountResponse, String> {
    let (account_repository, signer_repository) = get_repositories();
    let service = AccountService::new(account_repository, signer_repository);

    // Transfer the account
    service.transfer_account(request)
}

/// Activate an account
///
/// Only the owner can activate an account.
/// The account must be in the Unlocked state.
#[update]
pub fn activate_account(
    request: ActivateAccountRequest,
) -> Result<ActivateAccountResponse, String> {
    let (account_repository, signer_repository) = get_repositories();
    let service = AccountService::new(account_repository, signer_repository);

    // Activate the account
    service.activate_account(request)
}

/// Get account details
///
/// Retrieves the details of an account by its ID.
/// Anyone can query account details.
#[query]
pub fn get_account(request: GetAccountRequest) -> Result<GetAccountResponse, String> {
    let (account_repository, signer_repository) = get_repositories();
    let service = AccountService::new(account_repository, signer_repository);

    // Get the account
    service.get_account(request)
}

/// Sign a message with the account's private key
///
/// Only the owner can sign messages.
/// The account must be in the Active state.
#[update]
pub async fn sign(request: SignRequest) -> Result<SignResponse, String> {
    let (account_repository, signer_repository) = get_repositories();
    let service = AccountService::new(account_repository, signer_repository);

    // Sign the message
    service.sign(request).await
}

/// Sign an EIP-1559 transaction with the account's private key
///
/// Only the owner can sign transactions.
/// The account must be in the Active state.
/// The account must use ECDSA signature algorithm and secp256k1 curve.
#[update]
pub async fn sign_eip1559_transaction(
    request: SignEip1559TransactionRequest,
) -> Result<SignEip1559TransactionResponse, String> {
    let (account_repository, signer_repository) = get_repositories();
    let service = AccountService::new(account_repository, signer_repository);

    // Sign the transaction
    service.sign_eip1559_transaction(request).await
}

/// Generate a blockchain address for any supported chain
///
/// This unified endpoint supports multiple blockchains through CAIP chain identifiers.
/// Anyone can generate addresses for any supported chain.
#[query]
pub fn generate_address(
    request: GenerateAddressRequest,
) -> Result<GenerateAddressResponse, String> {
    let (account_repository, signer_repository) = get_repositories();
    let service = AccountService::new(account_repository, signer_repository);

    // Generate address for the specified chain
    service.generate_address(request)
}

// Export the Candid interface
ic_cdk::export_candid!();
