use crate::application::dtos::account_reply::AccountReply;
use crate::application::dtos::eip1559::Eip1559TransactionRequestDTO;
use crate::domain::models::signer::{Curve, SignatureAlgorithm};
use atp_caip::chain_id::ChainId;
use candid::{CandidType, Principal};
use serde::{Deserialize, Serialize};

#[derive(CandidType, Clone, Serialize, Deserialize, Debug)]
pub struct CreateAccountRequest {
    pub algorithm: SignatureAlgorithm,
    pub curve: Curve,
    pub approved_address: Principal,
}

#[derive(CandidType, Clone, Serialize, Deserialize, Debug)]
pub struct CreateAccountResponse {
    pub account: AccountReply,
}

#[derive(CandidType, Clone, Serialize, Deserialize, Debug)]
pub struct UnlockAccountRequest {
    pub account_id: String,
}

#[derive(CandidType, Clone, Serialize, Deserialize, Debug)]
pub struct UnlockAccountResponse {
    pub account: AccountReply,
}

#[derive(CandidType, Clone, Serialize, Deserialize, Debug)]
pub struct TransferAccountRequest {
    pub account_id: String,
    pub to: Principal,
}

#[derive(CandidType, Clone, Serialize, Deserialize, Debug)]
pub struct TransferAccountResponse {
    pub account: AccountReply,
}

#[derive(CandidType, Clone, Serialize, Deserialize, Debug)]
pub struct ActivateAccountRequest {
    pub account_id: String,
}

#[derive(CandidType, Clone, Serialize, Deserialize, Debug)]
pub struct ActivateAccountResponse {
    pub account: AccountReply,
}

#[derive(CandidType, Clone, Serialize, Deserialize, Debug)]
pub struct GetAccountRequest {
    pub account_id: String,
}

#[derive(CandidType, Clone, Serialize, Deserialize, Debug)]
pub struct GetAccountResponse {
    pub account: AccountReply,
}

#[derive(CandidType, Clone, Serialize, Deserialize, Debug)]
pub struct SignRequest {
    pub account_id: String,
    pub message_hex: String,
}

#[derive(CandidType, Clone, Serialize, Deserialize, Debug)]
pub struct SignResponse {
    pub signature: String,
}

#[derive(CandidType, Clone, Serialize, Deserialize, Debug)]
pub struct SignEip1559TransactionRequest {
    pub account_id: String,
    pub tx_request: Eip1559TransactionRequestDTO,
}

#[derive(CandidType, Clone, Serialize, Deserialize, Debug)]
pub struct SignEip1559TransactionResponse {
    pub signature: String,
}

#[derive(CandidType, Clone, Serialize, Deserialize, Debug)]
pub struct GetEthAddressRequest {
    pub account_id: String,
}

#[derive(CandidType, Clone, Serialize, Deserialize, Debug)]
pub struct GetEthAddressResponse {
    pub address: String,
}

#[derive(CandidType, Clone, Serialize, Deserialize, Debug)]
pub struct GenerateAddressRequest {
    pub account_id: String,
    pub chain_id: ChainId,
}

#[derive(CandidType, Clone, Serialize, Deserialize, Debug)]
pub struct GenerateAddressResponse {
    pub address: String,
}
