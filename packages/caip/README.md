# CAIP

A Rust implementation of Chain Agnostic Improvement Proposals (CAIP) standards for blockchain interoperability. This crate provides standardized identifiers and primitives for working with multiple blockchain networks in a chain-agnostic manner.

## Overview

CAIP implements the following CAIP standards:

- **CAIP-2**: Chain ID specification for blockchain identification
- **CAIP-10**: Account ID specification for blockchain account identification  
- **CAIP-19**: Asset ID specification for blockchain asset identification

The crate enables developers to build cross-chain applications by providing:

- Standardized identifiers for chains, accounts, and assets
- Type-safe financial primitives with decimal precision
- Trading pair representations for cross-chain and same-chain swaps
- Comprehensive error handling and validation
- Serialization support for configuration and API integration

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
caip = { path = "../caip" }
```

Or if published to crates.io:

```toml
[dependencies]
caip = "0.1.0"
```

## Quick Start

```rust
use caip::{ChainId, AssetId, Money, TokenPair};
use std::str::FromStr;

// Create chain identifiers
let ethereum = ChainId::new("eip155", "1")?;
let solana = ChainId::new("solana", "mainnet")?;

// Create asset identifiers
let eth = AssetId::new(ethereum.clone(), "slip44", "60")?;
let usdc = AssetId::new(ethereum, "erc20", "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48")?;
let sol = AssetId::new(solana, "slip44", "501")?;

// Create money amounts with proper decimals
let eth_amount = Money::from_decimal_str("1.5", 18)?;
let usdc_amount = Money::from_decimal_str("1000.50", 6)?;

// Create trading pairs
let eth_usdc_pair = TokenPair::new(eth.clone(), usdc.clone());
let cross_chain_pair = TokenPair::new(eth, sol);

println!("ETH amount: {}", eth_amount); // "1.5"
println!("Cross-chain pair: {}", cross_chain_pair.is_cross_chain()); // true
```

## Core Types

### Chain Identifiers (CAIP-2)

Chain IDs follow the format `namespace:reference` and identify specific blockchain networks:

```rust
use caip::ChainId;

// Ethereum mainnet
let ethereum = ChainId::new("eip155", "1")?;
assert_eq!(ethereum.to_string(), "eip155:1");

// Solana mainnet
let solana = ChainId::new("solana", "mainnet")?;
assert_eq!(solana.to_string(), "solana:mainnet");

// Parse from string
let chain = ChainId::from_str("eip155:1")?;
assert_eq!(chain.namespace(), "eip155");
assert_eq!(chain.reference(), "1");
```

### Asset Identifiers (CAIP-19)

Asset IDs follow the format `chain_id/asset_namespace:asset_reference`:

```rust
use caip::{AssetId, AssetIdBase, ChainId};

let ethereum = ChainId::new("eip155", "1")?;

// Native ETH
let eth = AssetId::new(ethereum.clone(), "slip44", "60")?;
assert_eq!(eth.to_string(), "eip155:1/slip44:60");

// USDC token
let usdc = AssetId::new(
    ethereum, 
    "erc20", 
    "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"
)?;
assert_eq!(usdc.to_string(), "eip155:1/erc20:0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48");

// Chain-agnostic asset identifier
let asset_base = AssetIdBase::new("slip44", "60")?;
assert_eq!(asset_base.to_string(), "slip44:60");
```

### Account Identifiers (CAIP-10)

Account IDs follow the format `chain_id:account_address`:

```rust
use caip::{AccountId, ChainId};

let ethereum = ChainId::new("eip155", "1")?;
let account = AccountId::new(
    ethereum, 
    "0xab16a96d359ec26a11e2c2b3d8f8b8942d5bfcdb"
)?;

assert_eq!(account.to_string(), "eip155:1:0xab16a96d359ec26a11e2c2b3d8f8b8942d5bfcdb");
assert_eq!(account.account_address(), "0xab16a96d359ec26a11e2c2b3d8f8b8942d5bfcdb");
```

## Financial Primitives

### Money Type

The `Money` type provides precise decimal arithmetic for cryptocurrency amounts:

```rust
use caip::Money;
use ethers_core::types::U256;

// Create from decimal string (human-readable)
let eth_amount = Money::from_decimal_str("1.5", 18)?;
assert_eq!(eth_amount.to_decimal_string(), "1.5");

// Create from raw amount (base units)
let wei_amount = Money::from_raw("1500000000000000000", 18)?;
assert_eq!(wei_amount.to_decimal_string(), "1.5");

// Arithmetic operations
let amount1 = Money::from_decimal_str("10.5", 6)?;
let amount2 = Money::from_decimal_str("5.25", 6)?;

let sum = amount1.add(&amount2)?;
assert_eq!(sum.to_decimal_string(), "15.75");

let difference = amount1.sub(&amount2)?;
assert_eq!(difference.to_decimal_string(), "5.25");

// Percentage calculations
let fee = amount1.percentage(3); // 3%
let basis_points_fee = amount1.basis_points(250); // 2.5%

// Convert to floating point (use with caution)
let float_value = amount1.to_f64();

// Check if zero
assert!(!amount1.is_zero());
assert!(Money::zero(18)?.is_zero());
```

### Asset Type

Combines an asset identifier with a money amount:

```rust
use caip::{Asset, AssetId, Money, ChainId};

let ethereum = ChainId::new("eip155", "1")?;
let eth_id = AssetId::new(ethereum, "slip44", "60")?;
let eth_amount = Money::from_decimal_str("2.5", 18)?;

let asset = Asset::new(eth_id, eth_amount);

// Calculate USD value given price per token
let usd_value = asset.usd_value(3000.0); // $3000 per ETH
assert_eq!(usd_value, 7500.0); // 2.5 * 3000
```

### Token Pairs

Represent trading relationships between two assets:

```rust
use caip::{TokenPair, AssetId, ChainId};

let ethereum = ChainId::new("eip155", "1")?;
let solana = ChainId::new("solana", "mainnet")?;

let eth = AssetId::new(ethereum, "slip44", "60")?;
let sol = AssetId::new(solana, "slip44", "501")?;

let pair = TokenPair::new(eth.clone(), sol.clone());

// Check if cross-chain
assert!(pair.is_cross_chain());

// Check if pair involves specific asset
assert!(pair.involves_asset(&eth));

// Get the other asset in the pair
assert_eq!(pair.get_other_asset(&eth), Some(&sol));

// Create reverse pair
let reverse_pair = pair.reverse();
assert_eq!(reverse_pair.from_asset, sol);
assert_eq!(reverse_pair.to_asset, eth);

// String representation
assert_eq!(pair.to_pair_string(), "eip155:1/slip44:60-solana:mainnet/slip44:501");

// Parse from string
let parsed_pair = TokenPair::from_pair_string("eip155:1/slip44:60-solana:mainnet/slip44:501")?;
assert_eq!(parsed_pair, pair);
```

## Namespace Enums

Predefined enums for common blockchain and asset namespaces:

```rust
use caip::{ChainNamespace, AssetNamespace};

// Chain namespaces
let evm_chains = ChainNamespace::Eip155;
let solana_chains = ChainNamespace::Solana;
let cosmos_chains = ChainNamespace::Cosmos;

assert_eq!(evm_chains.as_str(), "eip155");

// Asset namespaces
let native_tokens = AssetNamespace::Slip44;
let erc20_tokens = AssetNamespace::Erc20;
let nfts = AssetNamespace::Erc721;
let solana_tokens = AssetNamespace::Spl;

assert_eq!(erc20_tokens.as_str(), "erc20");
```

## Cryptographic Curves

Support for different cryptographic signature algorithms:

```rust
use caip::Curve;

let ethereum_curve = Curve::Secp256k1;
let solana_curve = Curve::Ed25519;
```

## Error Handling

Comprehensive error types for validation and parsing failures:

```rust
use caip::{CaipError, ChainId, AssetId, Money};

// Invalid chain ID format
match ChainId::new("", "1") {
    Err(CaipError::InvalidChainId(msg)) => println!("Invalid chain: {}", msg),
    _ => unreachable!(),
}

// Invalid asset ID format
match AssetId::from_str("invalid-format") {
    Err(CaipError::InvalidAssetId(msg)) => println!("Invalid asset: {}", msg),
    _ => unreachable!(),
}

// Decimal overflow
match Money::new(ethers_core::types::U256::zero(), 100) {
    Err(CaipError::DecimalOverflow { max, got }) => {
        println!("Too many decimals: {} > {}", got, max);
    },
    _ => unreachable!(),
}

// Invalid amount format
match Money::from_decimal_str("not-a-number", 18) {
    Err(CaipError::InvalidAmount(msg)) => println!("Invalid amount: {}", msg),
    _ => unreachable!(),
}
```

## Validation

All identifiers are validated against CAIP specification regex patterns:

- **Chain ID**: `^([-a-z0-9]{3,8}):([-a-zA-Z0-9]{1,32})$`
- **Account ID**: `^([-a-z0-9]{3,8}):([-a-zA-Z0-9]{1,32}):([-a-zA-Z0-9]{1,128})$`
- **Asset ID**: `^([-a-z0-9]{3,8}):([-a-zA-Z0-9]{1,32})/([-a-z0-9]{3,8}):([-a-zA-Z0-9]{1,64})$`
- **Asset ID Base**: `^([-a-z0-9]{3,8}):([-a-zA-Z0-9]{1,64})$`

## Integration with SolverOS Chain Registry

This crate is designed to work seamlessly with `solveros-chain-registry`:

```rust
// The chain registry uses CAIP types for configuration
use chain_registry::ChainRegistry;
use caip::{AssetId, ChainId};

let registry = ChainRegistry::default()?;

// Find trading routes between assets
let from_asset = AssetId::from_str("eip155:1/slip44:60")?; // ETH
let to_asset = AssetId::from_str("solana:mainnet/slip44:501")?; // SOL

let routes = registry.find_trading_routes(&from_asset, &to_asset, 3);
for route in routes {
    println!("Route: {:?}", route);
}

// Get chains supporting specific cryptographic curves
let secp256k1_chains = registry.get_chains_by_curve(&caip::Curve::Secp256k1);
```

## Serialization

All types support Serde serialization for configuration files and APIs:

```rust
use caip::{ChainId, AssetId, TokenPair};
use serde_json;

let chain = ChainId::new("eip155", "1")?;
let json = serde_json::to_string(&chain)?;
println!("{}", json); // "eip155:1"

let asset = AssetId::new(chain, "slip44", "60")?;
let toml = toml::to_string(&asset)?;

// TokenPair with trading configuration
let mut pair = TokenPair::new(asset.clone(), asset.clone());
pair.fee_percentage = Some(0.3);
pair.min_trade_amount = Some("0.01".to_string());

let config_json = serde_json::to_string_pretty(&pair)?;
```

## Testing

Run the test suite to verify functionality:

```bash
cargo test
```

The crate includes comprehensive tests covering:

- CAIP identifier validation and parsing
- Money arithmetic operations and edge cases
- TokenPair creation and manipulation
- Error handling scenarios
- Serialization round-trips

## Examples

### Cross-Chain Trading Scenario

```rust
use caip::{ChainId, AssetId, Money, Asset, TokenPair};

// Set up chains
let ethereum = ChainId::new("eip155", "1")?;
let solana = ChainId::new("solana", "mainnet")?;

// Set up assets
let eth = AssetId::new(ethereum, "slip44", "60")?;
let sol = AssetId::new(solana, "slip44", "501")?;

// Create asset holdings
let eth_holding = Asset::new(
    eth.clone(),
    Money::from_decimal_str("2.5", 18)?
);
let sol_holding = Asset::new(
    sol.clone(), 
    Money::from_decimal_str("100.0", 9)?
);

// Create trading pair
let trading_pair = TokenPair::new(eth, sol);

// Calculate portfolio value
let eth_price = 3000.0; // $3000 per ETH
let sol_price = 150.0;  // $150 per SOL

let total_value = eth_holding.usd_value(eth_price) + sol_holding.usd_value(sol_price);
println!("Portfolio value: ${:.2}", total_value); // $22,500.00

println!("Trading pair: {}", trading_pair);
println!("Is cross-chain: {}", trading_pair.is_cross_chain());
```

### Multi-Chain Asset Management

```rust
use caip::{ChainId, AssetId, AssetIdBase, Money};
use std::collections::HashMap;

// Define supported chains
let chains = vec![
    ChainId::new("eip155", "1")?,      // Ethereum
    ChainId::new("eip155", "137")?,    // Polygon
    ChainId::new("solana", "mainnet")?, // Solana
];

// Define USDC on different chains
let mut usdc_assets = HashMap::new();

// USDC on Ethereum
usdc_assets.insert(
    "ethereum".to_string(),
    AssetId::new(
        chains[0].clone(),
        "erc20",
        "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"
    )?
);

// USDC on Polygon
usdc_assets.insert(
    "polygon".to_string(),
    AssetId::new(
        chains[1].clone(),
        "erc20", 
        "0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174"
    )?
);

// USDC on Solana
usdc_assets.insert(
    "solana".to_string(),
    AssetId::new(
        chains[2].clone(),
        "spl",
        "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"
    )?
);

// Create balances
let mut balances = HashMap::new();
balances.insert("ethereum", Money::from_decimal_str("1000.50", 6)?);
balances.insert("polygon", Money::from_decimal_str("500.25", 6)?);
balances.insert("solana", Money::from_decimal_str("750.75", 6)?);

// Calculate total USDC across all chains
let total_usdc = balances.values()
    .try_fold(Money::zero(6)?, |acc, amount| acc.add(amount))?;

println!("Total USDC across chains: {}", total_usdc); // 2251.5
```

## Dependencies

- `ethers-core`: For U256 big integer support
- `serde`: For serialization/deserialization
- `regex`: For CAIP format validation
- `thiserror`: For error handling
- `lazy_static`: For compiled regex patterns

## CAIP Standards References

- [CAIP-2: Chain ID Specification](https://github.com/ChainAgnostic/CAIPs/blob/master/CAIPs/caip-2.md)
- [CAIP-10: Account ID Specification](https://github.com/ChainAgnostic/CAIPs/blob/master/CAIPs/caip-10.md)
- [CAIP-19: Asset ID Specification](https://github.com/ChainAgnostic/CAIPs/blob/master/CAIPs/caip-19.md)

## License

This project is part of the SolverOS ecosystem. See the main repository for license information.
