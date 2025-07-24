use candid::CandidType;
use ethers_core::types::{transaction::eip1559::Eip1559TransactionRequest, Address, U256};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(CandidType, Clone, Serialize, Deserialize, Debug)]
pub struct Eip1559TransactionRequestDTO {
    pub to: Option<String>,
    pub from: Option<String>,
    pub nonce: Option<String>,
    pub value: Option<String>,
    pub gas: Option<String>,
    pub max_priority_fee_per_gas: Option<String>,
    pub max_fee_per_gas: Option<String>,
    pub data: Option<Vec<u8>>,
    pub chain_id: Option<String>,
}

impl TryFrom<Eip1559TransactionRequestDTO> for Eip1559TransactionRequest {
    type Error = String;

    fn try_from(dto: Eip1559TransactionRequestDTO) -> Result<Self, Self::Error> {
        let mut tx = Eip1559TransactionRequest::new();

        if let Some(to) = dto.to {
            tx = tx.to(Address::from_str(&to).map_err(|e| e.to_string())?);
        }

        if let Some(from) = dto.from {
            tx = tx.from(Address::from_str(&from).map_err(|e| e.to_string())?);
        }

        if let Some(nonce) = dto.nonce {
            tx = tx.nonce(U256::from_dec_str(&nonce).map_err(|e| e.to_string())?);
        }

        if let Some(value) = dto.value {
            tx = tx.value(U256::from_dec_str(&value).map_err(|e| e.to_string())?);
        }

        if let Some(gas) = dto.gas {
            tx = tx.gas(U256::from_dec_str(&gas).map_err(|e| e.to_string())?);
        }

        if let Some(max_priority_fee) = dto.max_priority_fee_per_gas {
            tx = tx.max_priority_fee_per_gas(
                U256::from_dec_str(&max_priority_fee).map_err(|e| e.to_string())?,
            );
        }

        if let Some(max_fee) = dto.max_fee_per_gas {
            tx = tx.max_fee_per_gas(U256::from_dec_str(&max_fee).map_err(|e| e.to_string())?);
        }

        if let Some(data) = dto.data {
            tx = tx.data(data);
        }

        if let Some(chain_id) = dto.chain_id {
            tx = tx.chain_id(
                ethers_core::types::U64::from_dec_str(&chain_id).map_err(|e| e.to_string())?,
            );
        }

        Ok(tx)
    }
}

impl From<Eip1559TransactionRequest> for Eip1559TransactionRequestDTO {
    fn from(tx: Eip1559TransactionRequest) -> Self {
        Self {
            to: tx.to.map(|addr| format!("{:?}", addr)),
            from: tx.from.map(|addr| format!("{:?}", addr)),
            nonce: tx.nonce.map(|n| n.to_string()),
            value: tx.value.map(|v| v.to_string()),
            gas: tx.gas.map(|g| g.to_string()),
            max_priority_fee_per_gas: tx.max_priority_fee_per_gas.map(|f| f.to_string()),
            max_fee_per_gas: tx.max_fee_per_gas.map(|f| f.to_string()),
            data: tx.data.map(|d| d.to_vec()),
            chain_id: tx.chain_id.map(|c| c.to_string()),
        }
    }
}
