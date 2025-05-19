use candid::{CandidType, Decode, Encode, Principal};
use ic_stable_structures::{storable::Bound, Storable};
use serde::{Deserialize, Serialize};
use std::borrow::Cow;

use crate::domain::models::signer::{Curve, SignatureAlgorithm};
use crate::generate_getters;
use crate::utils::ic::api::get_ic_api;

#[derive(CandidType, Clone, Serialize, Deserialize, PartialEq, Debug)]
pub enum AccountState {
    #[serde(rename = "locked")]
    Locked,
    #[serde(rename = "unlocked")]
    Unlocked,
    #[serde(rename = "active")]
    Active,
}

impl Storable for AccountState {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), AccountState).unwrap()
    }

    const BOUND: Bound = Bound::Unbounded;
}

#[derive(CandidType, Clone, Serialize, Deserialize)]
pub struct Account {
    id: String,
    owner: Principal,
    public_key: Vec<u8>,
    algorithm: SignatureAlgorithm,
    curve: Curve,
    account_state: AccountState,
    approved_address: Option<Principal>,
}

impl Storable for Account {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Account).unwrap()
    }

    const BOUND: Bound = Bound::Unbounded;
}

#[derive(CandidType, Clone, Serialize, Deserialize)]
pub struct AccountReply {
    pub id: String,
    pub owner: String,
    pub public_key_hex: String,
    pub algorithm: SignatureAlgorithm,
    pub curve: Curve,
    pub account_state: AccountState,
    pub approved_address: String,
}

impl Account {
    // Constructor method for creating a new account
    pub fn new(
        id: String,
        owner: Principal,
        public_key: Vec<u8>,
        algorithm: SignatureAlgorithm,
        curve: Curve,
        approved_address: Principal,
    ) -> Self {
        Account {
            id,
            owner,
            public_key,
            algorithm,
            curve,
            account_state: AccountState::Locked,
            approved_address: Some(approved_address),
        }
    }

    generate_getters!(
        id: String,
        owner: Principal,
        public_key: Vec<u8>,
        algorithm: SignatureAlgorithm,
        curve: Curve,
        account_state: AccountState,
        approved_address: Option<Principal>
    );

    // Create a new account AccountReply
    pub fn to_account_reply(&self) -> AccountReply {
        AccountReply {
            id: self.id.clone(),
            owner: self.owner.to_string(),
            public_key_hex: hex::encode(&self.public_key),
            algorithm: self.algorithm.clone(),
            curve: self.curve.clone(),
            account_state: self.account_state.clone(),
            approved_address: match &self.approved_address {
                Some(address) => address.to_string(),
                None => "".to_string(),
            },
        }
    }

    // Method to check if the caller is approved
    pub fn is_approved(&self, caller: Principal) -> bool {
        match self.approved_address {
            Some(address) => address == caller,
            None => false,
        }
    }

    // Method to check if the owner is the caller
    pub fn is_owner(&self, caller: Principal) -> bool {
        self.owner == caller
    }

    // Transfer the account to a new owner, only allowed if locked and approved
    pub fn transfer_account(&mut self, to: Principal) -> Result<Account, String> {
        let ic_api = get_ic_api();
        if self.is_approved(ic_api.caller()) {
            if self.account_state == AccountState::Locked {
                // Reset the owner and remove the approved address
                self.owner = to;
                self.approved_address = None;
                // Unlock the account
                self.account_state = AccountState::Unlocked;
                Ok(self.clone())
            } else {
                Err("Account must be locked to transfer".to_string())
            }
        } else {
            Err("Caller is not approved to transfer the account".to_string())
        }
    }

    // Approve an address, allowing only the owner to approve
    pub fn approve_address(&mut self, address: Principal) -> Result<Account, String> {
        let ic_api = get_ic_api();
        if self.is_owner(ic_api.caller()) {
            // Check if the address is already approved
            match &self.approved_address {
                Some(approved_address) => {
                    if approved_address == &address {
                        // Return an error if the address is already approved
                        Err("This account is already approved".to_string())
                    } else {
                        // Approve the address if not already approved
                        self.approved_address = Some(address);
                        Ok(self.clone())
                    }
                }
                // Approve the address if no address is already approved
                None => {
                    self.approved_address = Some(address);
                    Ok(self.clone())
                }
            }
        } else {
            Err("Caller is not the owner of the account".to_string())
        }
    }

    // Revoke an address, ensuring only the owner can revoke
    pub fn revoke_address(&mut self, address: Principal) -> Result<Account, String> {
        let ic_api = get_ic_api();
        if self.is_owner(ic_api.caller()) {
            match &self.approved_address {
                // Revoke the address if it is already approved
                Some(approved_address) => {
                    if approved_address == &address {
                        self.approved_address = None;
                        Ok(self.clone())
                    } else {
                        Err("This account is not approved".to_string())
                    }
                }
                None => Err("This account is not approved".to_string()),
            }
        } else {
            Err("Caller is not the owner of the account".to_string())
        }
    }
    // Unlock the account, only allowed if the caller is approved
    pub fn unlock(&mut self) -> Result<Account, String> {
        match self.account_state {
            AccountState::Locked => {
                // Check if the caller is approved application
                let ic_api = get_ic_api();
                if self.is_approved(ic_api.caller()) || self.is_approved(ic_api.id()) {
                    self.account_state = AccountState::Unlocked;
                    Ok(self.clone())
                } else {
                    Err("Caller is not approved".to_string())
                }
            }
            AccountState::Unlocked => Err("Account is already unlocked".to_string()),
            AccountState::Active => Err("Account is already active".to_string()),
        }
    }

    // Lock the account, only allowed if the caller is approved
    pub fn lock(&mut self) -> Result<Account, String> {
        match self.account_state {
            AccountState::Locked => Err("Account is already locked".to_string()),
            AccountState::Unlocked => {
                if self.is_approved(ic_cdk::api::caller()) || self.is_approved(ic_cdk::api::id()) {
                    // Check if the caller is approved application
                    self.account_state = AccountState::Locked;
                    Ok(self.clone())
                } else {
                    Err("Caller is not approved".to_string())
                }
            }
            AccountState::Active => Err("Account is already active".to_string()),
        }
    }

    // Activate the account, only allowed if the caller is owner
    pub fn activate(&mut self) -> Result<Account, String> {
        match self.account_state {
            AccountState::Locked => Err("Account is locked".to_string()),
            AccountState::Active => Err("Account is already activated".to_string()),
            AccountState::Unlocked => {
                // Check if the caller is the owner
                let ic_api = get_ic_api();
                if self.is_owner(ic_api.caller()) {
                    self.account_state = AccountState::Active;
                    Ok(self.clone())
                } else {
                    Err("Caller is not the owner".to_string())
                }
            }
        }
    }
}
