# ATP Integration Guide

This guide explains how to integrate ATP with other canisters and applications.

## Integrating ATP with Other Canisters

Other canisters can interact with ATP through inter-canister calls. Here's how to integrate:

### Using the ATP Interface

Instead of recreating the interface, you can directly use the existing ATP Candid interface file (`atp.did`) in your project:

```bash
# Copy the atp.did file to your project
cp /path/to/atp/src/atp/atp.did /path/to/your/project/
```

### Example: Create and Transfer an Account from Another Canister

```rust
use candid::{CandidType, Principal};
use ic_cdk::api::call::call;
use serde::Deserialize;

// ATP types
#[derive(CandidType, Clone, Deserialize)]
enum SignatureAlgorithm {
    #[serde(rename = "ecdsa")]
    Ecdsa,
    #[serde(rename = "schnorr")]
    Schnorr,
}

#[derive(CandidType, Clone, Deserialize)]
enum Curve {
    #[serde(rename = "secp256k1")]
    Secp256k1,
    #[serde(rename = "ed25519")]
    Ed25519,
}

#[derive(CandidType, Clone, Deserialize)]
enum AccountState {
    #[serde(rename = "locked")]
    Locked,
    #[serde(rename = "unlocked")]
    Unlocked,
    #[serde(rename = "active")]
    Active,
}

#[derive(CandidType, Clone, Deserialize)]
struct AccountReply {
    id: String,
    owner: String,
    public_key_hex: String,
    algorithm: SignatureAlgorithm,
    curve: Curve,
    account_state: AccountState,
    approved_address: String,
}

// Request/Response types
#[derive(CandidType, Clone, Deserialize)]
struct CreateAccountRequest {
    algorithm: SignatureAlgorithm,
    curve: Curve,
    approved_address: Principal,
}

#[derive(CandidType, Clone, Deserialize)]
struct CreateAccountResponse {
    account: AccountReply,
}

#[derive(CandidType, Clone, Deserialize)]
struct TransferAccountRequest {
    account_id: String,
    to: Principal,
}

#[derive(CandidType, Clone, Deserialize)]
struct TransferAccountResponse {
    account: AccountReply,
}

#[derive(CandidType, Clone, Deserialize)]
struct GetAccountRequest {
    account_id: String,
}

#[derive(CandidType, Clone, Deserialize)]
struct GetAccountResponse {
    account: AccountReply,
}

// ATP canister ID (replace with your actual canister ID)
const ATP_CANISTER_ID: &str = "your_atp_canister_id";

// Note: Make sure you're using the correct ATP binary for your network:
// - atp-local.wasm: For local development with dfx replica (uses "dfx_test_key")
// - atp-test.wasm: For testing on the Internet Computer mainnet (uses "test_key_1")
// - atp-production.wasm: For production deployments (uses "key_1")
// Download the appropriate binary from: https://github.com/mycel-labs/atp/releases/latest

#[ic_cdk::update]
async fn create_and_transfer_account(to: Principal) -> Result<AccountReply, String> {
    // Step 1: Create an account
    let create_request = CreateAccountRequest {
        algorithm: SignatureAlgorithm::Ecdsa,
        curve: Curve::Secp256k1,
        approved_address: ic_cdk::id(), // Use this canister as the approved address
    };
    
    let create_result: Result<CreateAccountResponse, String> = call(
        Principal::from_text(ATP_CANISTER_ID).unwrap(),
        "create_account",
        (create_request,),
    )
    .await
    .map_err(|e| format!("Error calling create_account: {:?}", e))?;
    
    let account = match create_result {
        Ok(response) => response.account,
        Err(e) => return Err(format!("Failed to create account: {}", e)),
    };
    
    // Step 2: Transfer the account to the specified principal
    let transfer_request = TransferAccountRequest {
        account_id: account.id.clone(),
        to,
    };
    
    let transfer_result: Result<TransferAccountResponse, String> = call(
        Principal::from_text(ATP_CANISTER_ID).unwrap(),
        "transfer_account",
        (transfer_request,),
    )
    .await
    .map_err(|e| format!("Error calling transfer_account: {:?}", e))?;
    
    match transfer_result {
        Ok(response) => Ok(response.account),
        Err(e) => Err(format!("Failed to transfer account: {}", e)),
    }
}
```

### Example: Account Swapping Between Two Users

```rust
// Function to facilitate account swapping between two users
async fn swap_accounts(
    user1_account_id: String,
    user2_account_id: String,
    user1: Principal,
    user2: Principal,
) -> Result<(), String> {
    // Step 1: Get a reference to this canister (acts as the approved entity)
    let this_canister = ic_cdk::id();
    
    // Step 2: Get account details to verify ownership and state
    let get_account1_request = GetAccountRequest {
        account_id: user1_account_id.clone(),
    };
    
    let account1_result: Result<GetAccountResponse, String> = call(
        Principal::from_text(ATP_CANISTER_ID).unwrap(),
        "get_account",
        (get_account1_request,),
    )
    .await
    .map_err(|e| format!("Error getting account1: {:?}", e))?;
    
    let get_account2_request = GetAccountRequest {
        account_id: user2_account_id.clone(),
    };
    
    let account2_result: Result<GetAccountResponse, String> = call(
        Principal::from_text(ATP_CANISTER_ID).unwrap(),
        "get_account",
        (get_account2_request,),
    )
    .await
    .map_err(|e| format!("Error getting account2: {:?}", e))?;
    
    // Step 3: Verify accounts are in the right state
    let account1 = match account1_result {
        Ok(response) => {
            if response.account.account_state != AccountState::Locked {
                return Err("Account 1 must be locked for swapping".to_string());
            }
            response.account
        },
        Err(e) => return Err(format!("Failed to get account 1: {}", e)),
    };
    
    let account2 = match account2_result {
        Ok(response) => {
            if response.account.account_state != AccountState::Locked {
                return Err("Account 2 must be locked for swapping".to_string());
            }
            response.account
        },
        Err(e) => return Err(format!("Failed to get account 2: {}", e)),
    };
    
    // Step 4: Transfer account1 to user2
    let transfer1_request = TransferAccountRequest {
        account_id: user1_account_id,
        to: user2,
    };
    
    let transfer1_result: Result<TransferAccountResponse, String> = call(
        Principal::from_text(ATP_CANISTER_ID).unwrap(),
        "transfer_account",
        (transfer1_request,),
    )
    .await
    .map_err(|e| format!("Error transferring account1: {:?}", e))?;
    
    // Step 5: Transfer account2 to user1
    let transfer2_request = TransferAccountRequest {
        account_id: user2_account_id,
        to: user1,
    };
    
    let transfer2_result: Result<TransferAccountResponse, String> = call(
        Principal::from_text(ATP_CANISTER_ID).unwrap(),
        "transfer_account",
        (transfer2_request,),
    )
    .await
    .map_err(|e| format!("Error transferring account2: {:?}", e))?;
    
    Ok(())
}
```
