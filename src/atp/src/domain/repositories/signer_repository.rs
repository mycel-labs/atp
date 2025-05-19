use candid::CandidType;
use ethers_core::types::transaction::eip1559::Eip1559TransactionRequest;
use serde::{Deserialize, Serialize};
use std::future::Future;

use crate::domain::models::signer::{Curve, SignatureAlgorithm};

#[derive(CandidType, Serialize, Deserialize, Debug)]
pub struct PublicKeyReply {
    pub public_key: Vec<u8>,
    pub chain_code: Vec<u8>,
}

#[derive(CandidType, Serialize, Deserialize, Debug)]
pub struct SignatureReply {
    pub signature: Vec<u8>,
}

/// Interface for the threshold signer service
pub trait ISignerRepository {
    fn generate_public_key(
        &self,
        algorithm: SignatureAlgorithm,
        curve: Curve,
        derivation_path: &str,
    ) -> impl Future<Output = Result<PublicKeyReply, String>> + Send;

    fn sign(
        &self,
        algorithm: SignatureAlgorithm,
        curve: Curve,
        message_hash: Vec<u8>,
        derivation_path: &str,
    ) -> impl Future<Output = Result<SignatureReply, String>> + Send;

    fn sign_eip1559_transaction(
        &self,
        tx: Eip1559TransactionRequest,
        derivation_path: &str,
    ) -> impl Future<Output = Result<String, String>> + Send;
}
