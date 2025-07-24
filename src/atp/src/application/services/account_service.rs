use candid::Principal;
use ethers_core::types::transaction::eip1559::Eip1559TransactionRequest;

use crate::application::dtos::account_messages::*;
use crate::application::dtos::account_reply::AccountReply;
use crate::domain::models::account::{Account, AccountState};
use crate::domain::models::signer::{Curve, SignatureAlgorithm};
use crate::domain::repositories::account_repository::IAccountRepository;
use crate::domain::repositories::signer_repository::ISignerRepository;
use crate::infrastructure::repositories::account_repository_impl::AccountRepositoryImpl;
use crate::infrastructure::repositories::signer_repository_impl::SignerRepositoryImpl;
use crate::utils::eth_utils::{generate_eth_address_from_sec1, sha256};

pub struct AccountService {
    account_repository: AccountRepositoryImpl,
    signer_repository: SignerRepositoryImpl,
}

impl AccountService {
    pub fn new(
        account_repository: AccountRepositoryImpl,
        signer_repository: SignerRepositoryImpl,
    ) -> Self {
        Self {
            account_repository,
            signer_repository,
        }
    }
    // Convert domain model to DTO
    pub fn to_account_reply(&self, account: &Account) -> AccountReply {
        AccountReply {
            id: account.id().clone(),
            owner: account.owner().to_string(),
            public_key_hex: hex::encode(account.public_key()),
            algorithm: account.algorithm().clone(),
            curve: account.curve().clone(),
            account_state: account.account_state().clone(),
            approved_address: match account.approved_address() {
                Some(address) => address.to_string(),
                None => "".to_string(),
            },
        }
    }

    pub async fn create_account(
        &self,
        request: CreateAccountRequest,
        owner: Principal,
    ) -> Result<CreateAccountResponse, String> {
        // Generate a unique account ID
        let principal = ic_cdk::api::caller().to_string();
        let timestamp = ic_cdk::api::time();
        let id_string = format!("{}{}", principal, timestamp);
        let id = hex::encode(sha256(&id_string));

        // Generate a new public key
        let public_key = self
            .signer_repository
            .generate_public_key(request.algorithm.clone(), request.curve.clone(), id.clone())
            .await?;

        // Create a new account
        let account = Account::new(
            id,
            owner,
            public_key.public_key,
            request.algorithm.clone(),
            request.curve.clone(),
            request.approved_address,
        );

        let created_account = self.account_repository.insert(account.clone())?;
        Ok(CreateAccountResponse {
            account: self.to_account_reply(&created_account),
        })
    }

    pub fn unlock_account(
        &self,
        request: UnlockAccountRequest,
    ) -> Result<UnlockAccountResponse, String> {
        // Check if the account exists
        let mut account = self.account_repository.get(&request.account_id)?;
        // unlock the account
        account.unlock()?;
        // update the account in the repository
        let updated_account = self.account_repository.insert(account.clone())?;
        Ok(UnlockAccountResponse {
            account: self.to_account_reply(&updated_account),
        })
    }

    pub fn transfer_account(
        &self,
        request: TransferAccountRequest,
    ) -> Result<TransferAccountResponse, String> {
        // Check if the account exists
        let mut account = self.account_repository.get(&request.account_id)?;
        // Transfer the account
        account.transfer_account(request.to)?;
        // update the account in the repository
        let updated_account = self.account_repository.insert(account.clone())?;
        Ok(TransferAccountResponse {
            account: self.to_account_reply(&updated_account),
        })
    }

    pub fn activate_account(
        &self,
        request: ActivateAccountRequest,
    ) -> Result<ActivateAccountResponse, String> {
        // Check if the account exists
        let mut account = self.account_repository.get(&request.account_id)?;
        // unlock the account
        account.activate()?;
        // update the account in the repository
        let updated_account = self.account_repository.insert(account.clone())?;
        Ok(ActivateAccountResponse {
            account: self.to_account_reply(&updated_account),
        })
    }

    pub fn get_account(&self, request: GetAccountRequest) -> Result<GetAccountResponse, String> {
        let account = self.account_repository.get(&request.account_id)?;
        Ok(GetAccountResponse {
            account: self.to_account_reply(&account),
        })
    }

    pub async fn sign(&self, request: SignRequest) -> Result<SignResponse, String> {
        // Check if the account exists
        let account = self.account_repository.get(&request.account_id)?;

        // Check if the account is active
        if account.account_state().clone() != AccountState::Active {
            return Err("Account is not activated".to_string());
        }
        let message_bytes = match hex::decode(&request.message_hex) {
            Ok(bytes) => bytes,
            Err(_) => return Err("Invalid hex string".to_string()),
        };
        // Check if the caller is the owner of the account
        if account.is_owner(ic_cdk::api::caller()) {
            let signature = self
                .signer_repository
                .sign(
                    account.algorithm().clone(),
                    account.curve().clone(),
                    message_bytes,
                    account.id().clone(),
                )
                .await?;
            Ok(SignResponse {
                signature: hex::encode(signature.signature),
            })
        } else {
            Err("Caller is not the owner of the account".to_string())
        }
    }

    pub async fn sign_eip1559_transaction(
        &self,
        request: SignEip1559TransactionRequest,
    ) -> Result<SignEip1559TransactionResponse, String> {
        // Check if the account exists
        let account = self.account_repository.get(&request.account_id)?;
        // Check if the signature algorithm is ECDSA
        if account.algorithm().clone() != SignatureAlgorithm::Ecdsa {
            return Err("Signature algorithm is not ECDSA".to_string());
        }
        // Check if the curve is secp256k1
        if account.curve().clone() != Curve::Secp256k1 {
            return Err("Curve is not secp256k1".to_string());
        }

        // Check if the account is active
        if account.account_state().clone() != AccountState::Active {
            return Err("Account is not activated".to_string());
        }
        // Check if the caller is the owner of the account
        if account.is_owner(ic_cdk::api::caller()) {
            let tx = Eip1559TransactionRequest::try_from(request.tx_request)?;
            let signature = self
                .signer_repository
                .sign_eip1559_transaction(tx, account.id().clone())
                .await?;
            Ok(SignEip1559TransactionResponse { signature })
        } else {
            Err("Caller is not the owner of the account".to_string())
        }
    }

    pub fn get_eth_address(
        &self,
        request: GetEthAddressRequest,
    ) -> Result<GetEthAddressResponse, String> {
        // Check if the account exists
        let account = self.account_repository.get(&request.account_id)?;

        // Check if the signature algorithm is ECDSA
        if account.algorithm().clone() != SignatureAlgorithm::Ecdsa {
            return Err("Signature algorithm is not ECDSA".to_string());
        }
        // Check if the curve is secp256k1
        if account.curve().clone() != Curve::Secp256k1 {
            return Err("Curve is not secp256k1".to_string());
        }

        let address = generate_eth_address_from_sec1(account.public_key().clone())?;
        Ok(GetEthAddressResponse { address })
    }
}
