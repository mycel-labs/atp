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

## Testing Endpoints

Once deployed, you can interact with the ATP canister using the Candid UI (recommended) or through the command line.

### Using the Candid UI (Recommended)

When you deploy the canister using `dfx deploy`, the command will output a URL to access the Candid UI. This is the recommended approach for testing the endpoints as it provides a user-friendly interface.

### Using the Command Line

Alternatively, you can use the `dfx canister call` command to interact with the ATP canister:

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

## Example Implementation

Here's a simple example of how to create and activate an account:

```bash
# Step 1: Deploy the ATP canister
dfx deploy

# Step 2: Get your principal ID
MY_PRINCIPAL=$(dfx identity get-principal)

# Step 3: Create an account with ECDSA/secp256k1
RESULT=$(dfx canister call atp create_account "(variant {ecdsa}, variant {secp256k1}, principal \"$MY_PRINCIPAL\")")

# Step 4: Extract the account ID from the result
ACCOUNT_ID=$(echo $RESULT | grep -o '"id": "[^"]*' | cut -d'"' -f4)

# Step 5: Transfer the account to another principal
OTHER_PRINCIPAL="aaaaa-aa"  # Replace with an actual principal
dfx canister call atp transfer_account "(\"$ACCOUNT_ID\", principal \"$OTHER_PRINCIPAL\")"

# Step 6: As the new owner, activate the account
# (You would need to switch identities or use a different terminal)
dfx canister call atp activate_account "(\"$ACCOUNT_ID\")"

# Step 7: Sign a message with the activated account
dfx canister call atp sign "(\"$ACCOUNT_ID\", \"48656c6c6f20576f726c64\")"
```

This example demonstrates the complete lifecycle of an account from creation to activation and usage.
