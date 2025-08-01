# Getting Started with ATP

This guide will help you set up and start using the Account Transfer Protocol (ATP) on the Internet Computer.

## Setting up the Development Environment

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

## Network Configuration

ATP uses different key IDs depending on the network you're deploying to. The key ID is defined in `src/atp/src/utils/config.rs`:

```rust
// Default: For local development with dfx replica
pub const KEY_ID: &str = "dfx_test_key";

// For testing on the Internet Computer mainnet (uncomment when needed)
// pub const KEY_ID: &str = "test_key_1";

// For production deployments on the Internet Computer mainnet (uncomment when needed)
// pub const KEY_ID: &str = "key_1";
```

When switching networks:
1. Open `src/atp/src/utils/config.rs`
2. Comment out the current key ID
3. Uncomment the key ID for your target network
4. Rebuild and redeploy the canister

## Testing Endpoints

Once deployed, you can interact with the ATP canister using the Candid UI (recommended) or through the command line.

### Using the Candid UI (Recommended)

When you deploy the canister using `dfx deploy`, the command will output a URL to access the Candid UI. This is the recommended approach for testing the endpoints as it provides a user-friendly interface.

### Using the Command Line

Alternatively, you can use the `dfx canister call` command to interact with the ATP canister:

#### Create an account
```bash
dfx canister call atp create_account '(record { algorithm = variant {ecdsa}; curve = variant {secp256k1}; approved_address = principal "YOUR_PRINCIPAL_ID" })'
```

#### Get account details
```bash
dfx canister call atp get_account '(record { account_id = "YOUR_ACCOUNT_ID" })'
```

#### Transfer an account
```bash
dfx canister call atp transfer_account '(record { account_id = "YOUR_ACCOUNT_ID"; to = principal "NEW_OWNER_PRINCIPAL_ID" })'
```

#### Activate an account
```bash
dfx canister call atp activate_account '(record { account_id = "YOUR_ACCOUNT_ID" })'
```

#### Sign a message
```bash
dfx canister call atp sign '(record { account_id = "YOUR_ACCOUNT_ID"; message_hex = "48656c6c6f20576f726c64" })'  # "Hello World" in hex
```

## Example Implementation

Here's a simple example of how to create and activate an account:

```bash
# Step 1: Deploy the ATP canister
dfx deploy

# Step 2: Get your principal ID
MY_PRINCIPAL=$(dfx identity get-principal)

# Step 3: Create an account with ECDSA/secp256k1
RESULT=$(dfx canister call atp create_account "(record { algorithm = variant {ecdsa}; curve = variant {secp256k1}; approved_address = principal \"$MY_PRINCIPAL\" })")

# Step 4: Extract the account ID from the result
ACCOUNT_ID=$(echo $RESULT | grep -o 'id = "[^"]*' | cut -d'"' -f2)

# Step 5: Transfer the account to another principal
## Replace with an actual principal
OTHER_PRINCIPAL="aaaaa-aa"
dfx canister call atp transfer_account "(record { account_id = \"$ACCOUNT_ID\"; to = principal \"$OTHER_PRINCIPAL\" })"

# Step 6: As the new owner, activate the account
# (You would need to switch identities or use a different terminal)
dfx canister call atp activate_account "(record { account_id = \"$ACCOUNT_ID\" })"

# Step 7: Sign a message with the activated account
dfx canister call atp sign "(record { account_id = \"$ACCOUNT_ID\"; message_hex = \"48656c6c6f20576f726c64\" })"
```

This example demonstrates the complete lifecycle of an account from creation to activation and usage.
