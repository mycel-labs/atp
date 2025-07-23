use candid::Principal;
use ethers_core::types::transaction::eip1559::Eip1559TransactionRequest;

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
        algorithm: SignatureAlgorithm,
        curve: Curve,
        owner: Principal,
        approved_address: Principal,
    ) -> Result<Account, String> {
        // Generate a unique account ID
        let principal = ic_cdk::api::caller().to_string();
        let timestamp = ic_cdk::api::time();
        let id_string = format!("{}{}", principal, timestamp);
        let id = hex::encode(sha256(&id_string));

        // Generate a new public key
        let public_key = self
            .signer_repository
            .generate_public_key(algorithm.clone(), curve.clone(), id.clone())
            .await?;

        // Create a new account
        let account = Account::new(
            id,
            owner,
            public_key.public_key,
            algorithm.clone(),
            curve.clone(),
            approved_address,
        );

        self.account_repository.insert(account.clone())
    }

    pub fn unlock_account(&self, account_id: String) -> Result<Account, String> {
        // Check if the account exists
        let mut account = self.account_repository.get(&account_id)?;
        // unlock the account
        account.unlock()?;
        // update the account in the repository
        self.account_repository.insert(account.clone())
    }

    pub fn transfer_account(&self, account_id: String, to: Principal) -> Result<Account, String> {
        // Check if the account exists
        let mut account = self.account_repository.get(&account_id)?;
        // Transfer the account
        account.transfer_account(to)?;
        // update the account in the repository
        self.account_repository.insert(account.clone())
    }

    pub fn activate_account(&self, account_id: String) -> Result<Account, String> {
        // Check if the account exists
        let mut account = self.account_repository.get(&account_id)?;
        // unlock the account
        account.activate()?;
        // update the account in the repository
        self.account_repository.insert(account.clone())
    }

    pub fn get_account(&self, account_id: String) -> Result<Account, String> {
        self.account_repository.get(&account_id)
    }

    pub async fn sign(&self, account_id: String, message_hex: String) -> Result<String, String> {
        // Check if the account exists
        let account = self.account_repository.get(&account_id)?;

        // Check if the account is active
        if account.account_state().clone() != AccountState::Active {
            return Err("Account is not activated".to_string());
        }
        let message_bytes = match hex::decode(&message_hex) {
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
            Ok(hex::encode(signature.signature))
        } else {
            Err("Caller is not the owner of the account".to_string())
        }
    }

    pub async fn sign_eip1559_transaction(
        &self,
        account_id: String,
        tx: Eip1559TransactionRequest,
    ) -> Result<String, String> {
        // Check if the account exists
        let account = self.account_repository.get(&account_id)?;
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
            self.signer_repository
                .sign_eip1559_transaction(tx, account.id().clone())
                .await
        } else {
            Err("Caller is not the owner of the account".to_string())
        }
    }

    pub fn get_eth_address(&self, account_id: String) -> Result<String, String> {
        // Check if the account exists
        let account = self.account_repository.get(&account_id)?;

        // Check if the signature algorithm is ECDSA
        if account.algorithm().clone() != SignatureAlgorithm::Ecdsa {
            return Err("Signature algorithm is not ECDSA".to_string());
        }
        // Check if the curve is secp256k1
        if account.curve().clone() != Curve::Secp256k1 {
            return Err("Curve is not secp256k1".to_string());
        }

        generate_eth_address_from_sec1(account.public_key().clone())
    }
}
