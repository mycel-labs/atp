use crate::domain::models::account::AccountState;
use crate::domain::models::signer::SignatureAlgorithm;
use atp_caip::curve::Curve;
use candid::CandidType;
use serde::{Deserialize, Serialize};

#[derive(CandidType, Clone, Serialize, Deserialize, Debug)]
pub struct AccountReply {
    pub id: String,
    pub owner: String,
    pub public_key_hex: String,
    pub algorithm: SignatureAlgorithm,
    pub curve: Curve,
    pub account_state: AccountState,
    pub approved_address: String,
}
