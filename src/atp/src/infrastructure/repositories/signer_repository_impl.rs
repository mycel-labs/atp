use atp_caip::curve::Curve;
use candid::Principal;
use ethers_core::abi::ethereum_types::U256;
use ethers_core::types::transaction::eip1559::Eip1559TransactionRequest;
use ethers_core::types::Signature;
use ethers_core::utils::{hex, keccak256};

use std::cell::RefCell;
use std::future::Future;

use crate::domain::models::signer::SignatureAlgorithm;
use crate::domain::repositories::signer_repository::{
    EcdsaKeyId, EcdsaKeyIdCurve, EcdsaPublicKeyRequest, EcdsaSignatureRequest, ISignerRepository,
    PublicKeyReply, SchnorrKeyId, SchnorrKeyIdAlgorithm, SchnorrPublicKeyRequest,
    SchnorrSignatureRequest, SignatureReply,
};

thread_local! {
    static SIGNER_REPOSITORY: RefCell<Option<SignerRepositoryImpl>> = RefCell::new(None);
}

#[derive(Clone)]
pub struct SignerRepositoryImpl {
    key_id: String,
}

impl SignerRepositoryImpl {
    pub fn new(key_id: String) -> Self {
        Self { key_id }
    }

    /// Initialize the global signer repository
    pub fn init(key_id: String) {
        SIGNER_REPOSITORY.with(|repo| {
            *repo.borrow_mut() = Some(SignerRepositoryImpl { key_id });
        });
    }

    /// Get the global signer repository instance
    pub fn global() -> Self {
        SIGNER_REPOSITORY.with(|repo| match &*repo.borrow() {
            Some(instance) => instance.clone(),
            None => {
                panic!("SignerRepository not initialized! Call SignerRepositoryImpl::init() first.")
            }
        })
    }
}

impl ISignerRepository for SignerRepositoryImpl {
    fn generate_public_key(
        &self,
        algorithm: SignatureAlgorithm,
        curve: Curve,
        derivation_path: String,
    ) -> impl Future<Output = Result<PublicKeyReply, String>> {
        async move {
            let result = match algorithm {
                SignatureAlgorithm::Ecdsa => match curve {
                    Curve::Secp256k1 => {
                        let request = EcdsaPublicKeyRequest {
                            canister_id: None,
                            derivation_path: vec![derivation_path.as_bytes().to_vec()],
                            key_id: EcdsaKeyId {
                                curve: EcdsaKeyIdCurve::Ecdsa,
                                name: self.key_id.clone(),
                            },
                        };

                        let (response,): (PublicKeyReply,) = ic_cdk::call(
                            Principal::management_canister(),
                            "ecdsa_public_key",
                            (request,),
                        )
                        .await
                        .map_err(|e| format!("generate_public_key failed {}", e.1))?;
                        response
                    }
                    Curve::Ed25519 => return Err("Curve not supported for ECDSA".to_string()),
                },
                SignatureAlgorithm::Schnorr => match curve {
                    Curve::Secp256k1 => {
                        let request = SchnorrPublicKeyRequest {
                            canister_id: None,
                            derivation_path: vec![derivation_path.as_bytes().to_vec()],
                            key_id: SchnorrKeyId {
                                algorithm: SchnorrKeyIdAlgorithm::SchnorrBip340Secp256k1,
                                name: self.key_id.clone(),
                            },
                        };

                        let (response,): (PublicKeyReply,) = ic_cdk::call(
                            Principal::management_canister(),
                            "schnorr_public_key",
                            (request,),
                        )
                        .await
                        .map_err(|e| format!("generate_public_key failed {}", e.1))?;
                        response
                    }
                    Curve::Ed25519 => {
                        let request = SchnorrPublicKeyRequest {
                            canister_id: None,
                            derivation_path: vec![derivation_path.as_bytes().to_vec()],
                            key_id: SchnorrKeyId {
                                algorithm: SchnorrKeyIdAlgorithm::SchnorrEd25519,
                                name: self.key_id.clone(),
                            },
                        };

                        let (response,): (PublicKeyReply,) = ic_cdk::call(
                            Principal::management_canister(),
                            "schnorr_public_key",
                            (request,),
                        )
                        .await
                        .map_err(|e| format!("generate_public_key failed {}", e.1))?;
                        response
                    }
                },
            };
            Ok(result)
        }
    }

    fn sign(
        &self,
        algorithm: SignatureAlgorithm,
        curve: Curve,
        message_hash: Vec<u8>,
        derivation_path: String,
    ) -> impl Future<Output = Result<SignatureReply, String>> {
        async move {
            match algorithm {
                SignatureAlgorithm::Ecdsa => {
                    let request = EcdsaSignatureRequest {
                        message_hash: message_hash.to_vec(),
                        derivation_path: vec![derivation_path.as_bytes().to_vec()],
                        key_id: EcdsaKeyId {
                            curve: EcdsaKeyIdCurve::Ecdsa,
                            name: self.key_id.clone(),
                        },
                    };

                    let (response,): (SignatureReply,) = ic_cdk::api::call::call_with_payment(
                        Principal::management_canister(),
                        "sign_with_ecdsa",
                        (request,),
                        27_000_000_000,
                    )
                    .await
                    .map_err(|e| format!("sign failed {}", e.1))?;

                    Ok(response)
                }
                SignatureAlgorithm::Schnorr => match curve {
                    Curve::Secp256k1 => {
                        let request = SchnorrSignatureRequest {
                            message: message_hash.to_vec(),
                            derivation_path: vec![derivation_path.as_bytes().to_vec()],
                            key_id: SchnorrKeyId {
                                algorithm: SchnorrKeyIdAlgorithm::SchnorrBip340Secp256k1,
                                name: self.key_id.clone(),
                            },
                        };

                        let (response,): (SignatureReply,) = ic_cdk::api::call::call_with_payment(
                            Principal::management_canister(),
                            "sign_with_schnorr",
                            (request,),
                            27_000_000_000,
                        )
                        .await
                        .map_err(|e| format!("sign failed {}", e.1))?;
                        Ok(response)
                    }
                    Curve::Ed25519 => {
                        let request = SchnorrSignatureRequest {
                            message: message_hash.to_vec(),
                            derivation_path: vec![derivation_path.as_bytes().to_vec()],
                            key_id: SchnorrKeyId {
                                algorithm: SchnorrKeyIdAlgorithm::SchnorrEd25519,
                                name: self.key_id.clone(),
                            },
                        };

                        let (response,): (SignatureReply,) = ic_cdk::api::call::call_with_payment(
                            Principal::management_canister(),
                            "sign_with_schnorr",
                            (request,),
                            27_000_000_000,
                        )
                        .await
                        .map_err(|e| format!("sign failed {}", e.1))?;
                        Ok(response)
                    }
                },
            }
        }
    }

    fn sign_eip1559_transaction(
        &self,
        tx: Eip1559TransactionRequest,
        derivation_path: String,
    ) -> impl Future<Output = Result<String, String>> {
        async move {
            const EIP1559_TX_ID: u8 = 2;

            // Get the public key
            let public_key = self
                .generate_public_key(
                    SignatureAlgorithm::Ecdsa,
                    Curve::Secp256k1,
                    derivation_path.clone(),
                )
                .await?
                .public_key;

            // Prepare transaction for signing
            let mut unsigned_tx_bytes = tx.rlp().to_vec();
            unsigned_tx_bytes.insert(0, EIP1559_TX_ID);

            let txhash = keccak256(&unsigned_tx_bytes);

            let signature = self
                .sign(
                    SignatureAlgorithm::Ecdsa,
                    Curve::Secp256k1,
                    txhash.to_vec(),
                    derivation_path.clone(),
                )
                .await?
                .signature;

            // Recover signature parity
            let v = recover_signature_parity(&txhash, &signature, &public_key)
                .map_err(|e| format!("Signature recovery failed: {}", e))?;

            let signature = Signature {
                v: v as u64,
                r: U256::from_big_endian(&signature[0..32]),
                s: U256::from_big_endian(&signature[32..64]),
            };

            // Create signed transaction
            let mut signed_tx_bytes = tx.rlp_signed(&signature).to_vec();
            signed_tx_bytes.insert(0, EIP1559_TX_ID);

            Ok(format!("0x{}", hex::encode(&signed_tx_bytes)))
        }
    }
}

fn recover_signature_parity(message: &[u8], signature: &[u8], pubkey: &[u8]) -> Result<u8, String> {
    use ethers_core::k256::ecdsa::{RecoveryId, Signature, VerifyingKey};
    let sig = Signature::try_from(&signature[..64])
        .map_err(|e| format!("Invalid signature format: {}", e))?;

    let orig_key = VerifyingKey::from_sec1_bytes(pubkey)
        .map_err(|e| format!("Invalid public key format: {}", e))?;

    // Try both possible recovery IDs
    for recovery_id in [0u8, 1] {
        if let Ok(recovered_key) = VerifyingKey::recover_from_prehash(
            message,
            &sig,
            RecoveryId::try_from(recovery_id).map_err(|e| format!("Invalid recovery ID: {}", e))?,
        ) {
            if recovered_key == orig_key {
                return Ok(recovery_id);
            }
        }
    }

    Err("Could not recover matching public key with either recovery ID".to_string())
}
