# Chain Utils

A Rust library for generating blockchain addresses from public keys obtained via ICP threshold signatures, with support for CAIP (Chain Agnostic Improvement Proposals) chain identifiers.

## Features

- **Multi-chain support**: Generate addresses for different blockchain networks
- **CAIP integration**: Uses CAIP-2 chain identifiers for standardized chain specification
- **ICP threshold signature compatibility**: Designed to work with public keys from ICP threshold ECDSA/Schnorr signatures
- **Comprehensive testing**: Full test coverage with validation for multiple scenarios

## Supported Chains

| Namespace | Chain | Address Format | Public Key Format |
|-----------|-------|----------------|-------------------|
| `eip155` | Ethereum & EVM chains | 0x-prefixed hex (42 chars) | SEC1-encoded hex string |
| `solana` | Solana | Base58-encoded (32-44 chars) | 32-byte hex string |

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
atp-chain-utils = "0.1.0"
atp-caip = "0.1.0"
```

## Usage

### Basic Address Generation

```rust
use atp_chain_utils::address::generate_address;
use atp_caip::chain_id::ChainId;

// Generate Ethereum address
let eth_pubkey = "04a34b99f22c790c4e36b2b3c2c35a36db06226e41c692fc82b8b56ac1c540c5bd5b8dec5235a0fa8722476c7709c02559e3aa73aa03918ba2d492eea75abea235";
let eth_chain = ChainId::new("eip155", "1").unwrap(); // Ethereum mainnet
let eth_address = generate_address(eth_pubkey.to_string(), eth_chain)?;
println!("Ethereum address: {}", eth_address); // 0x1234...

// Generate Solana address  
let sol_pubkey = "e258d6e13adfb7b6eb771e0c9e8b1e3d4e3f1a2b3c4d5e6f7a8b9c0d1e2f3a4b";
let sol_chain = ChainId::new("solana", "mainnet").unwrap(); // Solana mainnet
let sol_address = generate_address(sol_pubkey.to_string(), sol_chain)?;
println!("Solana address: {}", sol_address); // Fe3d...
```

### Chain-Specific Generation

```rust
use atp_chain_utils::eip155::address as eth;
use atp_chain_utils::solana::address as sol;

// Direct Ethereum address generation
let eth_addr = eth::generate_address(eth_pubkey.to_string())?;

// Direct Solana address generation  
let sol_addr = sol::generate_address(sol_pubkey.to_string())?;
```

## Public Key Formats

### Ethereum (EIP155)

Accepts SEC1-encoded public keys in hex format:
- **Uncompressed**: 65 bytes starting with `04` (130 hex chars)
- **Compressed**: 33 bytes starting with `02` or `03` (66 hex chars)

```rust
// Uncompressed (recommended for ICP threshold signatures)
let uncompressed = "04a34b99f22c790c4e36b2b3c2c35a36db06226e41c692fc82b8b56ac1c540c5bd5b8dec5235a0fa8722476c7709c02559e3aa73aa03918ba2d492eea75abea235";

// Compressed
let compressed = "0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798";
```

### Solana

Requires exactly 32-byte public keys in hex format (64 hex characters):

```rust
let solana_pubkey = "e258d6e13adfb7b6eb771e0c9e8b1e3d4e3f1a2b3c4d5e6f7a8b9c0d1e2f3a4b";
```

## Error Handling

All functions return `Result<String, String>` with descriptive error messages:

```rust
match generate_address(pubkey, chain_id) {
    Ok(address) => println!("Generated address: {}", address),
    Err(error) => eprintln!("Failed to generate address: {}", error),
}
```

Common errors:
- `"Unsupported namespace: <namespace>"` - Chain not supported

## Testing

Run the test suite:

```bash
cargo test
```

