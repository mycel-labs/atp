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

// ATP canister ID (replace with your actual canister ID)
const ATP_CANISTER_ID: &str = "your_atp_canister_id";

#[ic_cdk::update]
async fn create_and_transfer_account(to: Principal) -> Result<AccountReply, String> {
    // Step 1: Create an account
    let algorithm = SignatureAlgorithm::Ecdsa;
    let curve = Curve::Secp256k1;
    
    // Use this canister as the approved address
    let approved_address = ic_cdk::id();
    
    let create_result: Result<AccountReply, String> = call(
        Principal::from_text(ATP_CANISTER_ID).unwrap(),
        "create_account",
        (algorithm, curve, approved_address),
    )
    .await
    .map_err(|e| format!("Error calling create_account: {:?}", e))?;
    
    let account = match create_result {
        Ok(account) => account,
        Err(e) => return Err(format!("Failed to create account: {}", e)),
    };
    
    // Step 2: Transfer the account to the specified principal
    let transfer_result: Result<AccountReply, String> = call(
        Principal::from_text(ATP_CANISTER_ID).unwrap(),
        "transfer_account",
        (account.id, to),
    )
    .await
    .map_err(|e| format!("Error calling transfer_account: {:?}", e))?;
    
    match transfer_result {
        Ok(account) => Ok(account),
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
    let account1_result: Result<AccountReply, String> = call(
        Principal::from_text(ATP_CANISTER_ID).unwrap(),
        "get_account",
        (user1_account_id.clone(),),
    )
    .await
    .map_err(|e| format!("Error getting account1: {:?}", e))?;
    
    let account2_result: Result<AccountReply, String> = call(
        Principal::from_text(ATP_CANISTER_ID).unwrap(),
        "get_account",
        (user2_account_id.clone(),),
    )
    .await
    .map_err(|e| format!("Error getting account2: {:?}", e))?;
    
    // Step 3: Verify accounts are in the right state
    let account1 = match account1_result {
        Ok(account) => {
            if account.account_state != AccountState::Locked {
                return Err("Account 1 must be locked for swapping".to_string());
            }
            account
        },
        Err(e) => return Err(format!("Failed to get account 1: {}", e)),
    };
    
    let account2 = match account2_result {
        Ok(account) => {
            if account.account_state != AccountState::Locked {
                return Err("Account 2 must be locked for swapping".to_string());
            }
            account
        },
        Err(e) => return Err(format!("Failed to get account 2: {}", e)),
    };
    
    // Step 4: Transfer account1 to user2
    let transfer1_result: Result<AccountReply, String> = call(
        Principal::from_text(ATP_CANISTER_ID).unwrap(),
        "transfer_account",
        (user1_account_id, user2),
    )
    .await
    .map_err(|e| format!("Error transferring account1: {:?}", e))?;
    
    // Step 5: Transfer account2 to user1
    let transfer2_result: Result<AccountReply, String> = call(
        Principal::from_text(ATP_CANISTER_ID).unwrap(),
        "transfer_account",
        (user2_account_id, user1),
    )
    .await
    .map_err(|e| format!("Error transferring account2: {:?}", e))?;
    
    Ok(())
}
```

## Use Cases

### 1. Multi-Asset Cross-Chain Swaps

ATP enables users to swap entire portfolios across different blockchains without needing to individually transfer each asset. This simplifies complex cross-chain operations:

- **DeFi Portfolio Transfers**: Move an entire DeFi position (including staked tokens, liquidity positions, etc.) from one chain to another
- **NFT Collection Transfers**: Transfer a collection of NFTs in a single operation
- **Multi-Token Wallets**: Swap wallets containing various tokens and assets

### 2. Decentralized Exchange Integration

DEXs can integrate with ATP to offer more sophisticated trading options:

- **Portfolio Swapping**: Instead of token-to-token trades, users can swap entire portfolios
- **Cross-Chain Trading**: Enable trading between assets on different blockchains without bridges
- **Batch Trading**: Execute multiple trades in a single transaction by swapping accounts

### 3. Blockchain Gaming

Game developers can use ATP to enable secure and efficient in-game asset transfers:

- **Character/Inventory Transfer**: Move a game character with all its inventory items across game worlds
- **Cross-Game Asset Usage**: Enable assets to be used across multiple games on different chains
- **Account Trading**: Allow players to trade entire game accounts securely

### 4. Credential Management

Organizations can use ATP for secure credential management:

- **Corporate Access Control**: Transfer access credentials between employees
- **Temporary Access Grants**: Grant temporary access to systems by transferring accounts
- **Credential Rotation**: Easily rotate access credentials across multiple systems

### 5. Protocol Integration for Ethereum Support

Since ATP supports signing Ethereum transactions (EIP-1559), it can be integrated with Ethereum-based protocols:

- **Multi-Chain Wallets**: Create wallets that work across Internet Computer and Ethereum ecosystems
- **Cross-Chain DApps**: Build applications that seamlessly operate across ICP and Ethereum
- **Ethereum Transaction Signing**: Use ATP accounts to sign and broadcast Ethereum transactions from the Internet Computer
