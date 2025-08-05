# ATP - Account Transfer Protocol

ATP (Account Transfer Protocol) is a protocol designed for transferring entire accounts across different blockchains or within the same blockchain. Unlike traditional methods that transfer individual assets, ATP treats an account as a unified object that can contain multiple assets (tokens, NFTs, DeFi positions, etc.).

## Key Features

- **Entire Account Transfer**: Transfer all assets in an account at once rather than individual tokens
- **Cross-Chain Flexibility**: Move accounts across different blockchain ecosystems
- **Secure Key Management**: Uses Internet Computer's threshold ECDSA and Schnorr signing capabilities
- **State-Based Security Model**: Accounts transition through different states (Locked, Unlocked, Active) to ensure secure transfers

## Documentation

For detailed documentation, see the following:

- [Overview](./docs/overview.md) - Concept and key features
- [Architecture](./docs/architecture.md) - Implementation details and state transitions
- [API Reference](./docs/api_reference.md) - Endpoint documentation
- [Getting Started](./docs/getting_started.md) - Setup and testing guide
- [Integration Guide](./docs/integration_guide.md) - How to integrate with other canisters
- [Contribution Guide](./docs/contribution_guide.md) - How to contribute to ATP

## Network Configuration
ATP provides pre-built binaries for different environments:
- **Local**: `atp-local.wasm` (uses `dfx_test_key`)
- **Test**: `atp-test.wasm` (uses `test_key_1`)
- **Production**: `atp-production.wasm` (uses `key_1`)

Download the appropriate binaries from the [latest GitHub release](https://github.com/mycel-labs/atp/releases/latest) and update your dfx.json accordingly.

See the [Getting Started](./docs/getting_started.md) guide for more details on network configuration.

## Quick Start

### Using Pre-built Binaries (Recommended)

Create a new project and configure it to use ATP from GitHub releases:

```bash
# Initialize dfx project
dfx new my_project
cd my_project
```

Update dfx.json to use ATP binary
```json
{
  "canisters": {
    "atp": {
      "type": "custom",
      "candid": "https://github.com/mycel-labs/atp/releases/latest/download/atp-local.did",
      "wasm": "https://github.com/mycel-labs/atp/releases/latest/download/atp-local.wasm"
    }
  },
}
```

Replace `atp-local` with `atp-test` or `atp-production` as needed for your target environment.


## State Transitions

Accounts in ATP go through various state transitions:

```mermaid
stateDiagram-v2
    [*] --> Locked: create_account()
    Locked --> Unlocked: transfer_account() by approved address
    Locked --> Unlocked: unlock() by owner
    Unlocked --> Locked: lock() by owner or approved address
    Unlocked --> Active: activate() by owner
    Active --> Active: sign() by owner
    Active --> [*]
```

## Key Endpoints

- `create_account`: Create a new account with specified algorithm and curve
- `transfer_account`: Transfer account ownership to another principal
- `activate_account`: Activate an unlocked account
- `sign`: Sign a message with the account's private key
- `sign_eip1559_transaction`: Sign an Ethereum transaction

For more details, see the [API Reference](./docs/api_reference.md).

## Reference
- [Internet Computer Documentation](https://internetcomputer.org/docs)
- [Rust Canister Development Guide](https://internetcomputer.org/docs/current/developer-docs/backend/rust/)
- [Candid Introduction](https://internetcomputer.org/docs/current/developer-docs/backend/candid/)
- [ninegua/ic-nix](https://github.com/ninegua/ic-nix)
