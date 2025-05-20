# ATP - Account Transfer Protocol Developer Documentation

## Table of Contents
- [Concept Overview](#concept-overview)
- [Architecture and Implementation](#architecture-and-implementation)
- [State Transitions](#state-transitions)
- [API Reference](#api-reference)
- [How to Try It](#how-to-try-it)
- [Integration Guide](#integration-guide)
- [Use Cases](#use-cases)

## Concept Overview

ATP (Account Transfer Protocol) is a protocol designed for transferring entire accounts across different blockchains or within the same blockchain. Unlike traditional methods that transfer individual assets, ATP treats an account as a unified object that can contain multiple assets (tokens, NFTs, DeFi positions, etc.).

### Key Features

- **Entire Account Transfer**: Transfer all assets in an account at once rather than individual tokens
- **Cross-Chain Flexibility**: Move accounts across different blockchain ecosystems
- **Secure Key Management**: Uses Internet Computer's threshold ECDSA and Schnorr signing capabilities
- **State-Based Security Model**: Accounts transition through different states (Locked, Unlocked, Active) to ensure secure transfers

### Why ATP?

Traditional cross-chain solutions like atomic swaps and bridge-based transfers face several limitations:

1. **Cryptographic Curve Dependency**: Atomic swaps require compatible cryptographic curves between chains
2. **Counterparty Finding**: Difficulty in matching counterparties for swaps
3. **Bridge Security Risks**: Vulnerabilities in bridge smart contracts can lead to asset loss

ATP addresses these issues by treating the account as the transferable unit, abstracting away underlying differences between chains and providing a more general and secure approach to asset transfers.

## Architecture and Implementation

ATP is implemented as a canister (smart contract) on the Internet Computer using Rust. It follows a clean architecture pattern with distinct layers:

### Domain Layer

Contains the core business logic and models that define what an account is and how it behaves.

### Application Layer

Implements use cases and services that orchestrate domain objects to fulfill business requirements.

### Infrastructure Layer

Provides concrete implementations for repositories and external services.

### Endpoints Layer

Exposes the API to consumers through Candid interfaces.

### Key Components

- **Account**: The main entity representing a transferable account with properties like owner, public key, and state
- **SignerRepository**: Manages cryptographic operations using Internet Computer's threshold signature schemes
- **AccountRepository**: Stores and retrieves account data
- **AccountService**: Orchestrates operations on accounts

## State Transitions

Accounts in ATP go through various state transitions during their lifecycle:

### Account States

1. **Locked**: The initial state of an account after creation. In this state:
   - The account is owned by the creator
   - An approved address (usually an application like a DEX) is set
   - Only the approved address can transfer or unlock the account

2. **Unlocked**: An intermediate state after transfer. In this state:
   - The account has a new owner
   - The owner must activate the account to use it
   - No approved address exists in this state

3. **Active**: The final state where the account can be used. In this state:
   - The owner can sign messages and transactions
   - The owner can approve addresses for future transfers
   - Only the owner can perform actions with the account

### State Transition Diagram

```
                ┌───────────┐
 create_account │           │
     ───────────► LOCKED    │
                │           │
                └─────┬─────┘
                      │
                      │ transfer_account
                      │ (by approved address)
                      ▼
                ┌───────────┐
                │           │
                │ UNLOCKED  │
                │           │
                └─────┬─────┘
                      │
                      │ activate_account
                      │ (by owner)
                      ▼
                ┌───────────┐
                │           │
                │ ACTIVE    │
                │           │
                └───────────┘
```

## API Reference

### Account Management

#### create_account
```candid
create_account: (algorithm: SignatureAlgorithm, curve: Curve, approved_address: principal) -> (variant { Ok: AccountReply; Err: text; });
```
Creates a new account with the specified signature algorithm, curve, and approved address. The caller becomes the owner of the account.

Parameters:
- `algorithm`: The signature algorithm to use (ECDSA or Schnorr)
- `curve`: The curve to use (secp256k1 or ed25519)
- `approved_address`: The principal that is approved to transfer the account

Returns:
- `AccountReply` with account details on success
- Error message on failure

#### unlock_account
```candid
unlock_account: (account_id: text) -> (variant { Ok: AccountReply; Err: text; });
```
Unlocks a locked account. Only the approved address can call this method.

Parameters:
- `account_id`: ID of the account to unlock

Returns:
- `AccountReply` with updated account details on success
- Error message on failure

#### transfer_account
```candid
transfer_account: (account_id: text, to: principal) -> (variant { Ok: AccountReply; Err: text; });
```
Transfers account ownership to another principal. Only the approved address can call this method, and the account must be in the Locked state.

Parameters:
- `account_id`: ID of the account to transfer
- `to`: Principal ID of the new owner

Returns:
- `AccountReply` with updated account details on success
- Error message on failure

#### activate_account
```candid
activate_account: (account_id: text) -> (variant { Ok: AccountReply; Err: text; });
```
Activates an unlocked account. Only the owner can call this method, and the account must be in the Unlocked state.

Parameters:
- `account_id`: ID of the account to activate

Returns:
- `AccountReply` with updated account details on success
- Error message on failure

#### get_account
```candid
get_account: (account_id: text) -> (variant { Ok: AccountReply; Err: text; }) query;
```
Retrieves account details. Anyone can call this method.

Parameters:
- `account_id`: ID of the account to retrieve

Returns:
- `AccountReply` with account details on success
- Error message on failure

### Signing Operations

#### sign
```candid
sign: (account_id: text, message_hex: text) -> (variant { Ok: text; Err: text; });
```
Signs a message with the account's private key. Only the owner can call this method, and the account must be in the Active state.

Parameters:
- `account_id`: ID of the account to use for signing
- `message_hex`: Hex-encoded message to sign

Returns:
- Hex-encoded signature on success
- Error message on failure

#### sign_eip1559_transaction
```candid
sign_eip1559_transaction: (account_id: text, tx_request: Eip1559TransactionRequestDTO) -> (variant { Ok: text; Err: text; });
```
Signs an EIP-1559 Ethereum transaction. Only the owner can call this method, and the account must be in the Active state with ECDSA/secp256k1.

Parameters:
- `account_id`: ID of the account to use for signing
- `tx_request`: Transaction request details

Returns:
- Hex-encoded signed transaction on success
- Error message on failure

#### get_eth_address
```candid
get_eth_address: (account_id: text) -> (variant { Ok: text; Err: text; }) query;
```
Retrieves the Ethereum address derived from the account's public key. Anyone can call this method.

Parameters:
- `account_id`: ID of the account

Returns:
- Hex-encoded Ethereum address on success
- Error message on failure

## How to Try It

### Setting up the Development Environment

1. Make sure you have the DFINITY SDK (dfx) installed:
```bash
sh -ci "$(curl -fsSL https://internetcomputer.org/install.sh)"
```

2. Clone the ATP repository:
```bash
git clone https://github.com/mycel-labs/atp
cd atp
```

3. Start the local Internet Computer replica:
```bash
dfx start --background
```

4. Deploy the ATP canister:
```bash
dfx deploy
```

### Testing Endpoints

Once deployed, you can interact with the ATP canister using `dfx canister call`:

#### Create an account
```bash
dfx canister call atp create_account '(variant {ecdsa}, variant {secp256k1}, principal "YOUR_PRINCIPAL_ID")'
```

#### Get account details
```bash
dfx canister call atp get_account '("YOUR_ACCOUNT_ID")'
```

#### Transfer an account
```bash
dfx canister call atp transfer_account '("YOUR_ACCOUNT_ID", principal "NEW_OWNER_PRINCIPAL_ID")'
```

#### Activate an account
```bash
dfx canister call atp activate_account '("YOUR_ACCOUNT_ID")'
```

#### Sign a message
```bash
dfx canister call atp sign '("YOUR_ACCOUNT_ID", "48656c6c6f20576f726c64")'  # "Hello World" in hex
```

### Using the Candid UI

The Internet Computer also provides a web-based UI for interacting with canisters:

1. Start the local replica (if not already running):
```bash
dfx start --background
```

2. Open the Candid UI:
```bash
dfx canister id __Candid_UI
```

3. Visit `http://localhost:4943/?canisterId=<candid_ui_canister_id>&id=<atp_canister_id>` in your browser.

## Integration Guide

### Integrating ATP with Other Canisters

Other canisters can interact with ATP through inter-canister calls. Here's how to integrate:

#### 1. Import the ATP Interface

Create a did file with the ATP interface:

```candid
// atp.did
type SignatureAlgorithm = variant {
  ecdsa;
  schnorr;
};

type Curve = variant {
  secp256k1;
  ed25519;
};

type AccountState = variant {
  locked;
  unlocked;
  active;
};

type AccountReply = record {
  id: text;
  owner: text;
  public_key_hex: text;
  algorithm: SignatureAlgorithm;
  curve: Curve;
  account_state: AccountState;
  approved_address: text;
};

type Eip1559TransactionRequestDTO = record {
  to: opt text;
  from: opt text;
  nonce: opt text;
  value: opt text;
  gas: opt text;
  max_priority_fee_per_gas: opt text;
  max_fee_per_gas: opt text;
  data: opt vec nat8;
  chain_id: opt text;
};

service : {
  create_account: (algorithm: SignatureAlgorithm, curve: Curve, approved_address: principal) -> (variant { Ok: AccountReply; Err: text; });
  unlock_account: (account_id: text) -> (variant { Ok: AccountReply; Err: text; });
  transfer_account: (account_id: text, to: principal) -> (variant { Ok: AccountReply; Err: text; });
  activate_account: (account_id: text) -> (variant { Ok: AccountReply; Err: text; });
  get_account: (account_id: text) -> (variant { Ok: AccountReply; Err: text; }) query;
  sign: (account_id: text, message_hex: text) -> (variant { Ok: text; Err: text; });
  sign_eip1559_transaction: (account_id: text, tx_request: Eip1559TransactionRequestDTO) -> (variant { Ok: text; Err: text; });
  get_eth_address: (account_id: text) -> (variant { Ok: text; Err: text; }) query;
}
```

#### 2. Example: Create and Transfer an Account from Another Canister

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

#### 3. Example: Account Swapping Between Two Users

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
