use candid::{CandidType, Principal};
use ethers_core::types::transaction::eip1559::Eip1559TransactionRequest;
use serde::{Deserialize, Serialize};
use std::future::Future;

use crate::domain::models::signer::SignatureAlgorithm;
use atp_caip::curve::Curve;

type CanisterId = Principal;

// Key ID for Schnorr and ECDSA keys
#[derive(CandidType, Serialize, Debug)]
pub enum SchnorrKeyIdAlgorithm {
    #[serde(rename = "bip340secp256k1")]
    SchnorrBip340Secp256k1,
    #[serde(rename = "ed25519")]
    SchnorrEd25519,
}

#[derive(CandidType, Serialize, Debug)]
pub enum EcdsaKeyIdCurve {
    #[serde(rename = "secp256k1")]
    Ecdsa,
}

#[derive(CandidType, Serialize, Debug)]
pub struct SchnorrKeyId {
    pub algorithm: SchnorrKeyIdAlgorithm,
    pub name: String,
}

#[derive(CandidType, Serialize, Debug)]
pub struct EcdsaKeyId {
    pub curve: EcdsaKeyIdCurve,
    pub name: String,
}

// Request for a public key
#[derive(CandidType, Serialize, Debug)]
pub struct SchnorrPublicKeyRequest {
    pub canister_id: Option<CanisterId>,
    pub derivation_path: Vec<Vec<u8>>,
    pub key_id: SchnorrKeyId,
}

#[derive(CandidType, Serialize, Debug)]
pub struct EcdsaPublicKeyRequest {
    pub canister_id: Option<CanisterId>,
    pub derivation_path: Vec<Vec<u8>>,
    pub key_id: EcdsaKeyId,
}

// Request for a signature
#[derive(CandidType, Serialize, Debug)]
pub struct SchnorrSignatureRequest {
    pub message: Vec<u8>,
    pub derivation_path: Vec<Vec<u8>>,
    pub key_id: SchnorrKeyId,
}

#[derive(CandidType, Serialize, Debug)]
pub struct EcdsaSignatureRequest {
    pub message_hash: Vec<u8>,
    pub derivation_path: Vec<Vec<u8>>,
    pub key_id: EcdsaKeyId,
}

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
        derivation_path: String,
    ) -> impl Future<Output = Result<PublicKeyReply, String>> + Send;

    fn sign(
        &self,
        algorithm: SignatureAlgorithm,
        curve: Curve,
        message_hash: Vec<u8>,
        derivation_path: String,
    ) -> impl Future<Output = Result<SignatureReply, String>> + Send;

    fn sign_eip1559_transaction(
        &self,
        tx: Eip1559TransactionRequest,
        derivation_path: String,
    ) -> impl Future<Output = Result<String, String>> + Send;
}
