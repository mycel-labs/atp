# IC-NoSQL

A type-safe NoSQL database library for Internet Computer canisters with automatic memory management.

## Overview

IC-NoSQL provides a high-level interface for storing and querying structured data in Internet Computer canisters. It handles memory allocation, serialization, and provides type-safe operations with pagination support.

## Features

- **Type Safety**: Compile-time type checking for all database operations
- **Automatic Memory Management**: Handles memory allocation with conflict prevention
- **Multiple Model Support**: Store different data types in the same canister
- **Pagination**: Built-in pagination for efficient querying
- **Data Persistence**: Data survives canister upgrades
- **CRUD Operations**: Complete Create, Read, Update, Delete support

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
ic-nosql = { path = "packages/ic-nosql" }
```

## Quick Start

### 1. Define Your Models

```rust
use ic_nosql::Model;
use candid::{CandidType, Deserialize};
use serde::Serialize;

#[derive(Model, CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct User {
    pub id: String,
    pub username: String,
    pub email: String,
    pub created_at: u64,
}

#[derive(Model, CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct Post {
    pub id: String,
    pub user_id: String,
    pub title: String,
    pub content: String,
    pub created_at: u64,
}
```

### 2. Initialize the Database

```rust
use ic_nosql::DatabaseManager;

static mut DB_MANAGER: Option<DatabaseManager> = None;

#[ic_cdk::init]
fn init() {
    unsafe {
        DB_MANAGER = Some(DatabaseManager::new());
        
        if let Some(db) = &mut DB_MANAGER {
            // Register models with unique memory IDs
            db.register_model("users", Some(10), None).expect("Failed to register users");
            db.register_model("posts", Some(11), None).expect("Failed to register posts");
        }
    }
}
```

### 3. Database Operations

```rust
// Create
let user = User {
    id: "user_123".to_string(),
    username: "alice".to_string(),
    email: "alice@example.com".to_string(),
    created_at: ic_cdk::api::time(),
};

unsafe {
    if let Some(db) = &mut DB_MANAGER {
        db.insert("users", &user.id, &user)?;
    }
}

// Read
let user = unsafe {
    DB_MANAGER.as_ref()
        .unwrap()
        .get::<User>("users", "user_123")?
        .ok_or("User not found")?
};

// Query with pagination
let response = unsafe {
    DB_MANAGER.as_ref()
        .unwrap()
        .query::<User>("users", 10, 1)?
};

// Update
user.email = "newemail@example.com".to_string();
unsafe {
    if let Some(db) = &mut DB_MANAGER {
        db.update("users", &user.id, &user)?;
    }
}

// Delete
unsafe {
    if let Some(db) = &mut DB_MANAGER {
        db.delete("users", "user_123")?;
    }
}
```

## API Reference

### DatabaseManager

#### Registration
- `register_model(name: &str, memory_id: Option<u8>, max_size: Option<u32>) -> Result<()>`

#### CRUD Operations
- `insert<T: Model>(collection: &str, id: &str, data: &T) -> Result<()>`
- `get<T: Model>(collection: &str, id: &str) -> Result<Option<T>>`
- `update<T: Model>(collection: &str, id: &str, data: &T) -> Result<()>`
- `delete(collection: &str, id: &str) -> Result<()>`

#### Querying
- `query<T: Model>(collection: &str, limit: usize, page: usize) -> Result<QueryResponse<T>>`
- `stats() -> Vec<String>`

### Model Trait

Derive the `Model` trait on your structs:

```rust
#[derive(Model, CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct MyModel {
    pub id: String,
    // ... other fields
}
```

## Memory Layout

IC-NoSQL uses stable memory with automatic allocation:

- Each model gets a unique memory ID (0-255)
- Automatic conflict detection and prevention

## Testing

The package includes comprehensive tests and stress tests:

```bash
# Build example canister
cargo build --package example-canister --target wasm32-unknown-unknown --release

# Run stress tests (requires PocketIC)
export POCKET_IC_BIN=$(which pocket-ic)
cargo test --package ic-nosql-tests --test stress_tests
```

## Example Canister

See `example-canister/` for a complete implementation demonstrating:

- User management system
- Blog posts with comments
- Pagination and CRUD operations
- Canister upgrade persistence

### Available Endpoints

#### User Management
- `create_user(username: text, email: text) -> Result<User, text>`
- `get_user(id: text) -> Result<User, text>`
- `list_users(page: nat, size: nat) -> Result<vec User, text>`

#### Post Management
- `create_post(user_id: text, title: text, content: text) -> Result<Post, text>`
- `get_post(id: text) -> Result<Post, text>`
- `list_posts(page: nat, size: nat) -> Result<vec Post, text>`

#### Comment Management
- `create_comment(post_id: text, user_id: text, content: text) -> Result<Comment, text>`
- `get_comment(id: text) -> Result<Comment, text>`
- `list_comments(page: nat, size: nat) -> Result<vec Comment, text>`

## Best Practices

1. **Memory Management**: Always register models during `init()` or `post_upgrade()`
2. **ID Generation**: Use proper ID generation strategies (UUIDs, timestamps, etc.)
3. **Error Handling**: Always handle database errors appropriately
4. **Pagination**: Use pagination for large result sets to avoid query limits
5. **Validation**: Validate data before storing (e.g., check if referenced entities exist)
6. **Upgrades**: Re-register all models after canister upgrades

## Troubleshooting

### Common Issues

1. **Memory Conflicts**: Ensure unique memory IDs for different models
2. **Upgrade Issues**: Always re-register models after upgrades
3. **Serialization Errors**: Ensure all fields are properly serializable
4. **Memory Limits**: Consider pagination for large datasets

### Debug Tips

- Use `ic_cdk::println!` for debugging in canisters
- Check memory usage with `stats()` method
- Verify model registration before operations
- Test upgrades in development environment

## Development

### Running Tests on NixOS

If using NixOS with the IC development environment:

```bash
# Start nix-shell with IC tools
nix-shell https://github.com/ninegua/ic-nix/releases/latest/download/dfx-env.tar.gz

# Set PocketIC binary path
export POCKET_IC_BIN=$(which pocket-ic)

# Run tests
cargo test
```

## References

- [Internet Computer Documentation](https://internetcomputer.org/docs)
- [PocketIC Documentation](https://docs.rs/pocket-ic)
- [Candid Documentation](https://github.com/dfinity/candid)
- [IC-CDK Documentation](https://docs.rs/ic-cdk)
