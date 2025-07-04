# IC-NoSQL Test Suite

This directory contains integration tests for the ic-nosql library using PocketIC.

## Structure

```
tests/
├── README.md                   # This file
├── integration_tests.rs        # PocketIC integration tests
└── example-canister/          # Example canister using ic-nosql
    ├── Cargo.toml
    ├── src/
    │   └── lib.rs             # Canister implementation
    └── example-canister.did   # Candid interface
```

## Example Canister

The example canister demonstrates how to use ic-nosql in a real Internet Computer canister. It implements a simple social media-like system with:

- **Users**: User management with username and email
- **Posts**: Blog posts created by users
- **Comments**: Comments on posts

### Features Demonstrated

1. **Multiple Model Support**: Different data types (User, Post, Comment) in the same canister
2. **Memory Management**: Automatic memory allocation with conflict prevention
3. **CRUD Operations**: Create, Read, Update, Delete operations
4. **Pagination**: Listing entities with pagination support
5. **Data Persistence**: Data survives canister upgrades
6. **Type Safety**: Compile-time type checking for all operations

### API Endpoints

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

#### Utilities
- `get_database_stats() -> vec text`
- `health_check() -> text`

## Running Tests

### Prerequisites

1. **Rust**: Install Rust via [rustup](https://rustup.rs/)
2. **wasm32-unknown-unknown target**: `rustup target add wasm32-unknown-unknown`
3. **PocketIC**: Included as a dev dependency

### Manual Testing

1. **Build the ic-nosql library**:
   ```bash
   cargo build --package ic-nosql
   ```

2. **Build the test canister**:
   ```bash
   cargo build --package example-canister --target wasm32-unknown-unknown --release
   ```

3. **Run stress tests**:
   ```bash
    cargo test --package ic-nosql-tests --test stress_tests
   ```
## Memory Layout

The example canister uses the following memory layout:

- **Memory ID 10**: Users storage
- **Memory ID 11**: Posts storage
- **Memory ID 12**: Comments storage
- **Memory IDs 0-9**: Reserved for ATP (if integrated)
- **Memory IDs 13+**: Available for additional models

## Usage Examples

### Creating a User
```rust
let user = User {
    id: "user_123".to_string(),
    username: "alice".to_string(),
    email: "alice@example.com".to_string(),
    created_at: ic_cdk::api::time(),
};

// Register the model first
db_manager.register_model("users", Some(10), None)?;

// Insert the user
db_manager.insert("users", &user.id, &user)?;
```

### Querying Data
```rust
// Get a specific user
let user = db_manager.get::<User>("users", "user_123")?;

// List users with pagination
let response = db_manager.query::<User>("users", 10, 1)?;
let users: Vec<User> = response.results.into_iter().map(|doc| doc.data).collect();
```

## Best Practices

1. **Memory Management**: Always register models during `init()` or `post_upgrade()`
2. **ID Generation**: Use proper ID generation strategies (the example uses simple random IDs)
3. **Error Handling**: Always handle database errors appropriately
4. **Pagination**: Use pagination for large result sets
5. **Validation**: Validate data before storing (e.g., check if referenced entities exist)

## Troubleshooting

### Common Issues

1. **Build Failures**: Ensure `wasm32-unknown-unknown` target is installed
2. **Test Failures**: Check that the WASM file is built before running tests
3. **Memory Conflicts**: Ensure unique memory IDs for different models
4. **Upgrade Issues**: Always re-register models after upgrades

### Debug Tips

- Use `ic_cdk::println!` for debugging in the canister
- Check the generated Candid interface for type mismatches
- Verify memory layout doesn't conflict with other storage systems

## References

- [IC-NoSQL Documentation](../packages/ic-nosql/src/lib.rs)
- [PocketIC Documentation](https://docs.rs/pocket-ic)
- [Internet Computer Documentation](https://internetcomputer.org/docs)
- [Candid Documentation](https://github.com/dfinity/candid)
