# ATP Library and NoSQL Database Refactoring Strategy

## Executive Summary

This document outlines the refactoring strategy to transform the ATP (Account Transfer Protocol) package into a reusable library that can be consumed by other canisters (such as `../astraeus-ic/canister`). The strategy also includes extracting the NoSQL database as a separate library and restructuring the current canister as an example implementation. The current codebase has architectural issues including dependency inversion violations, tight coupling between domain models and persistence, and mixed responsibilities across layers.

## Current State Analysis

### ATP Package Structure
- **Location**: `/home/taru/src/ic/atp/src/atp/`
- **Architecture**: Clean architecture with Domain, Application, Infrastructure, and Endpoints layers
- **Core Components**:
  - **Domain**: Account and Signer models with business logic
  - **Application**: AccountService orchestrating business operations
  - **Infrastructure**: Database persistence and repository implementations
  - **Endpoints**: Candid API interface for external access

### NoSQL Database Structure
- **Location**: `/home/taru/src/ic/atp/src/atp/src/infrastructure/database/`
- **Technology**: Custom NoSQL database built on IC's stable memory structures
- **Features**: Primary/secondary indexes, pagination, CRUD operations
- **Storage**: Uses StableBTreeMap for persistent storage across canister upgrades

### Key Architectural Issues

1. **Dependency Inversion Violations**: Application services directly depend on concrete implementations
2. **Domain Model Coupling**: Domain models tightly coupled to persistence serialization
3. **Global State Management**: Thread-local singletons create tight coupling
4. **Mixed Responsibilities**: Infrastructure concerns leak into domain and application layers

## Library-First Refactoring Strategy

### Design Goals for Library Architecture

1. **Reusable Library**: ATP should be consumable by other canisters as a library dependency
2. **Clean Public APIs**: Well-defined interfaces for account management, signing, and blockchain operations
3. **Flexible Database Integration**: NoSQL database as separate library with pluggable implementations
4. **Example Implementation**: Current canister becomes demonstration of library usage
5. **Minimal Dependencies**: Libraries should have minimal external dependencies for easy integration

### Phase 1: Prepare for Library Design (Weeks 1-2)

#### 1.1 Fix Dependency Inversion
- **Target**: `src/atp/src/application/services/account_service.rs`
- **Action**: Modify AccountService to depend on trait interfaces instead of concrete implementations
- **Changes**:
  ```rust
  // Current (problematic):
  pub struct AccountService {
      account_repository: AccountRepositoryImpl,
      signer_repository: SignerRepositoryImpl,
  }

  // Refactored:
  pub struct AccountService {
      account_repository: Box<dyn IAccountRepository>,
      signer_repository: Box<dyn ISignerRepository>,
  }
  ```

#### 1.2 Implement Dependency Injection
- **Target**: `src/atp/src/lifecycle.rs`
- **Action**: Create factory pattern for repository creation
- **Create**: `src/atp/src/infrastructure/factories/repository_factory.rs`
- **Interface**:
  ```rust
  pub trait IRepositoryFactory {
      fn create_account_repository(&self) -> Box<dyn IAccountRepository>;
      fn create_signer_repository(&self) -> Box<dyn ISignerRepository>;
  }
  ```

#### 1.3 Decouple Domain Models from Persistence
- **Target**: `src/atp/src/domain/models/account.rs`
- **Action**: Remove `Storable` trait from domain models
- **Create**: `src/atp/src/infrastructure/persistence/` with separate persistence models
- **Implement**: Mapper pattern between domain and persistence models

### Phase 2: Create Library Structure (Weeks 3-4)

#### 2.1 Create Root Library Structure (Rust 2018 Style)
- **Location**: `/home/taru/src/ic/atp/crates/atp-lib/`
- **Purpose**: Main ATP library for external consumption
- **Structure**:
  ```
  atp-lib/
  ├── Cargo.toml
  ├── src/
  │   ├── lib.rs                    # Public API exports
  │   ├── account/                  # Account management API
  │   │   ├── manager.rs           # Account manager implementation
  │   │   └── types.rs             # Account types and DTOs
  │   ├── signing/                  # Signing operations API
  │   │   ├── signer.rs            # Signer implementation
  │   │   └── types.rs             # Signing types and DTOs
  │   ├── blockchain/              # Blockchain utilities
  │   │   ├── ethereum.rs          # Ethereum-specific utilities
  │   │   └── types.rs             # Blockchain types
  │   ├── config/                  # Configuration management
  │   │   └── network.rs           # Network configuration
  │   └── error/                   # Error handling
  │       ├── mod.rs               # Public error types
  │       └── types.rs             # Error definitions
  ```

#### 2.2 Create NoSQL Database Library
- **Location**: `/home/taru/src/ic/atp/crates/ic-nosql/`
- **Purpose**: Reusable NoSQL database for IC canisters with flexible model support
- **Structure**:
  ```
  ic-nosql/
  ├── Cargo.toml
  ├── src/
  │   ├── lib.rs                    # Public API exports
  │   ├── database/                 # Database implementation
  │   │   ├── nosql_db.rs          # Core database implementation
  │   │   ├── manager.rs           # Database manager for multiple models
  │   │   └── types.rs             # Database types and structures
  │   ├── memory/                   # Memory management
  │   │   ├── stable_memory.rs     # IC stable memory integration
  │   │   └── manager.rs           # Memory manager for allocation
  │   ├── traits/                   # Database traits
  │   │   ├── database.rs          # Database trait definitions
  │   │   ├── model.rs             # Model trait definitions
  │   │   └── repository.rs        # Repository trait definitions
  │   ├── macros/                   # Helper macros
  │   │   └── model.rs             # Model definition macros
  │   └── utils/                    # Utilities
  │       └── serialization.rs     # Serialization helpers
  ```

#### 2.3 Create ATP Storage Implementation
- **Location**: `/home/taru/src/ic/atp/crates/atp-storage/`
- **Purpose**: ATP-specific storage implementation using ic-nosql
- **Structure**:
  ```
  atp-storage/
  ├── Cargo.toml
  ├── src/
  │   ├── lib.rs                    # Public API exports
  │   ├── repositories/             # Repository implementations
  │   │   ├── account_repository.rs # Account storage implementation
  │   │   └── signer_repository.rs  # Signer storage implementation
  │   ├── models/                   # Persistence models
  │   │   └── account_model.rs     # Account persistence model
  │   ├── mappers/                  # Domain to persistence mappers
  │   │   └── account_mapper.rs    # Account mapper implementation
  │   └── schema/                   # Database schema
  │       └── account_schema.rs    # Account table schema
  ```

#### 2.4 Create Example Canister
- **Location**: `/home/taru/src/ic/atp/examples/atp-canister/`
- **Purpose**: Example implementation showing how to use ATP library
- **Structure**:
  ```
  atp-canister/
  ├── Cargo.toml
  ├── dfx.json
  ├── atp-canister.did
  ├── src/
  │   ├── lib.rs                    # Canister implementation
  │   ├── endpoints/                # Candid endpoints
  │   │   └── account_endpoints.rs # Account API endpoints
  │   ├── lifecycle/                # Canister lifecycle
  │   │   └── init.rs              # Initialization logic
  │   └── config/                   # Canister configuration
  │       └── settings.rs          # Configuration settings
  ```

### Phase 3: Implement Library Structure (Weeks 5-6)

#### 3.1 Implement ATP Library Core
- **Actions**:
  - Create public API in `crates/atp-lib/src/lib.rs` with clean interfaces
  - Move domain logic to library modules with proper abstraction
  - Design `AccountManager` as main entry point for account operations
  - Create `Signer` trait and implementations for signing operations
  - Implement blockchain utilities for Ethereum integration
  - Remove direct IC-specific dependencies from core library

#### 3.2 Implement NoSQL Database Library
- **Actions**:
  - Extract `src/atp/src/infrastructure/database/` to `crates/ic-nosql/src/database/`
  - Create generic database traits in `crates/ic-nosql/src/traits/`
  - Make database implementation generic and reusable
  - Add comprehensive documentation and examples
  - Create memory management abstractions
  - Implement serialization utilities

#### 3.3 Implement ATP Storage Layer
- **Actions**:
  - Create ATP-specific storage implementations in `crates/atp-storage/`
  - Move repository implementations with proper abstraction
  - Create persistence models separate from domain models
  - Implement mappers between domain and persistence models
  - Integrate with ic-nosql library for storage operations
  - Add schema management and initialization

#### 3.4 Create Example Canister
- **Actions**:
  - Transform current canister into example in `examples/atp-canister/`
  - Create simple API endpoints that demonstrate library usage
  - Implement lifecycle management using ATP library
  - Add configuration management
  - Create comprehensive documentation and usage examples

### Phase 4: Create Workspace and Integration (Week 7)

#### 4.1 Create Package Configuration (Rust 2018 Style)
- **Target**: `/home/taru/src/ic/atp/Cargo.toml`
- **Actions**:
  - Convert to single package with example workspace:
    ```toml
    [package]
    name = "atp"
    version = "0.1.0"
    edition = "2021"
    
    [lib]
    name = "atp"
    path = "lib.rs"
    
    [dependencies]
    ic-cdk = "0.13"
    ic-stable-structures = "0.6"
    candid = "0.10"
    k256 = { version = "0.13", features = ["ecdsa"] }
    ethers-core = "2.0"
    
    [[example]]
    name = "atp-canister"
    path = "examples/atp-canister/src/lib.rs"
    crate-type = ["cdylib"]
    ```

#### 4.2 Setup Module Dependencies (Rust 2018 Style)
- **Actions**:
  - Configure internal module dependencies within single package
  - Ensure clean separation between `atp`, `nosql`, and `storage` modules
  - Setup example canister to use main library
  - Create feature flags for optional functionality if needed

#### 4.3 Create Integration Layer (Rust 2018 Style)
- **Target**: `/home/taru/src/ic/atp/lib.rs`
- **Actions**:
  - Design clean public API for external consumption:
    ```rust
    //! ATP Library - Account Transfer Protocol for Internet Computer
    
    // ATP Core functionality
    pub mod atp;
    pub use atp::{AtpClient, AtpConfig, AccountManager, Account};
    pub use atp::signing::{Signer, SigningConfig, Signature};
    pub use atp::blockchain::{EthereumUtils, TransactionBuilder};
    pub use atp::error::{AtpError, AtpResult};
    
    // NoSQL Database functionality  
    pub mod nosql;
    pub use nosql::{DatabaseManager, Database, define_model};
    pub use nosql::traits::{Model, Repository, Query};
    pub use nosql::memory::{MemoryManager, MemoryId};
    
    // Storage functionality
    pub mod storage;
    pub use storage::{AtpStorage, AtpStorageConfig};
    pub use storage::repositories::{AccountRepository, SignerRepository};
    
    // Re-export commonly used types
    pub use candid::{CandidType, Deserialize};
    pub use ic_cdk::export::Principal;
    ```

#### 4.4 Update Build Configuration (Rust 2018 Style)
- **Actions**:
  - Update dfx.json for example canister only
  - Configure single package build system
  - Add integration tests for module boundaries
  - Setup library publishing configuration for single package
  - Create feature flags for optional IC-specific functionality

### Phase 5: Testing and Validation (Week 8)

#### 5.1 Update Test Suite
- **Actions**:
  - Move tests to appropriate packages
  - Update import paths in test files
  - Create integration tests for package boundaries
  - Ensure all existing functionality works

#### 5.2 Performance Validation
- **Actions**:
  - Benchmark before and after refactoring
  - Ensure no performance degradation
  - Validate memory usage patterns
  - Test canister upgrade scenarios

#### 5.3 Documentation Updates
- **Actions**:
  - Update README files for each package
  - Create package-specific documentation
  - Update architecture diagrams
  - Document new dependency injection patterns

## Library Dependencies and Structure

### Final Library Structure (Rust 2018 Style)
```
atp/
├── Cargo.toml             (Workspace configuration)
├── lib.rs                 (Main library - all public APIs)
├── src/
│   ├── atp/               (ATP library modules)
│   │   ├── account/       (Account management)
│   │   ├── signing/       (Signing operations)
│   │   ├── blockchain/    (Blockchain utilities)
│   │   ├── config/        (Configuration)
│   │   └── error/         (Error handling)
│   ├── nosql/             (NoSQL database modules)
│   │   ├── database/      (Database implementation)
│   │   ├── memory/        (Memory management)
│   │   ├── traits/        (Database traits)
│   │   ├── macros/        (Helper macros)
│   │   └── utils/         (Utilities)
│   └── storage/           (ATP storage modules)
│       ├── repositories/  (Repository implementations)
│       ├── models/        (Persistence models)
│       ├── mappers/       (Domain mappers)
│       └── schema/        (Database schema)
├── examples/
│   └── atp-canister/      (Example canister implementation)
└── docs/                  (Library documentation)
```

### Dependency Graph (Rust 2018 Style)
```
atp::nosql (minimal dependencies, core database)
    ↑
atp::storage (depends on: atp::nosql, atp::core)
    ↑
atp::* (all modules available from root lib.rs)
    ↑
examples/atp-canister (depends on: atp)
    ↑
../astraeus-ic/canister (depends on: atp)
```

### Library Responsibilities (Rust 2018 Style)

#### ATP Library (Single Package)
- **Purpose**: Complete ATP functionality with modular design
- **Dependencies**: Standard library, basic crypto, ic-stable-structures, candid
- **Root Exports in `lib.rs`**: 
  ```rust
  // ATP Core functionality
  pub use atp::{AtpClient, AtpConfig, AccountManager, Account};
  pub use atp::signing::{Signer, SigningConfig, Signature};
  pub use atp::blockchain::{EthereumUtils, TransactionBuilder};
  pub use atp::error::{AtpError, AtpResult};
  
  // NoSQL Database functionality
  pub use nosql::{DatabaseManager, Database, define_model};
  pub use nosql::traits::{Model, Repository, Query};
  pub use nosql::memory::{MemoryManager, MemoryId};
  
  // Storage functionality  
  pub use storage::{AtpStorage, AtpStorageConfig};
  pub use storage::repositories::{AccountRepository, SignerRepository};
  ```
- **Usage**: `cargo add atp`

#### Module Organization

**ATP Module (`src/atp/`)**:
- **Purpose**: Core ATP business logic
- **Dependencies**: Minimal - only standard library and basic crypto
- **Exports**: 
  - `AtpClient` - Main entry point
  - `AccountManager` - Account management
  - `Signer` traits - Signing operations
  - `EthereumUtils` - Blockchain utilities
  - Error types and result types

**NoSQL Module (`src/nosql/`)**:
- **Purpose**: Generic NoSQL database for Internet Computer
- **Dependencies**: ic-stable-structures, candid
- **Exports**: 
  - `DatabaseManager` - Central database manager for multiple models
  - `Database` trait and implementations
  - `define_model!` macro for easy model definition
  - Memory management utilities with automatic allocation
  - Serialization helpers
  - Query builders and repository traits

**Storage Module (`src/storage/`)**:
- **Purpose**: ATP-specific storage implementation
- **Dependencies**: Internal - depends on atp and nosql modules
- **Exports**: 
  - Storage implementations for ATP types
  - Repository implementations
  - Schema management
  - Migration utilities

### External Usage Example (Rust 2018 Style)

From `../astraeus-ic/canister/Cargo.toml`:
```toml
[dependencies]
atp = { path = "../atp" }
```

From `../astraeus-ic/canister/src/lib.rs`:
```rust
use atp::{AtpClient, AtpConfig, AtpStorageConfig};
use atp::{DatabaseManager, define_model};

// Define your own custom models for Astraeus
define_model! {
    #[derive(Debug, Clone, CandidType, Deserialize)]
    pub struct UserProfile {
        pub id: String,
        pub username: String,
        pub email: String,
        pub created_at: u64,
    }
}

define_model! {
    #[derive(Debug, Clone, CandidType, Deserialize)]
    pub struct Transaction {
        pub id: String,
        pub user_id: String,
        pub amount: u64,
        pub timestamp: u64,
    }
}

thread_local! {
    static DATABASE_MANAGER: RefCell<Option<DatabaseManager>> = RefCell::new(None);
    static ATP_CLIENT: RefCell<Option<AtpClient>> = RefCell::new(None);
}

#[ic_cdk::init]
fn init() {
    // Initialize database manager first
    let db_manager = DatabaseManager::new();
    
    // Initialize ATP storage (uses its own memory segments)
    let atp_config = AtpConfig::new()
        .with_network("testnet")
        .with_storage(AtpStorageConfig::new(&db_manager));
    
    let atp_client = AtpClient::new(atp_config).unwrap();
    
    // Initialize your custom models (uses separate memory segments)
    db_manager.register_model::<UserProfile>("user_profiles", 10, Some(11)).unwrap();
    db_manager.register_model::<Transaction>("transactions", 12, Some(13)).unwrap();
    
    // Store globally
    DATABASE_MANAGER.with(|db| *db.borrow_mut() = Some(db_manager));
    ATP_CLIENT.with(|client| *client.borrow_mut() = Some(atp_client));
}

// ATP functionality
#[ic_cdk::update]
async fn create_account(owner: String) -> Result<String, String> {
    ATP_CLIENT.with(|client| {
        client.borrow()
            .as_ref()
            .unwrap()
            .create_account(&owner)
            .map(|account| account.id)
            .map_err(|e| format!("Failed to create account: {}", e))
    })
}

// Your custom model functionality  
#[ic_cdk::update]
async fn create_user_profile(username: String, email: String) -> Result<String, String> {
    DATABASE_MANAGER.with(|db_manager| {
        let db = db_manager.borrow();
        let db = db.as_ref().unwrap();
        
        let user_profile = UserProfile {
            id: format!("user_{}", ic_cdk::api::time()),
            username,
            email,
            created_at: ic_cdk::api::time(),
        };
        
        db.insert("user_profiles", &user_profile.id, &user_profile)
            .map(|_| user_profile.id)
            .map_err(|e| format!("Failed to create user profile: {}", e))
    })
}

#[ic_cdk::query]
fn get_user_profile(user_id: String) -> Result<UserProfile, String> {
    DATABASE_MANAGER.with(|db_manager| {
        let db = db_manager.borrow();
        let db = db.as_ref().unwrap();
        
        db.get::<UserProfile>("user_profiles", &user_id)
            .map_err(|e| format!("Failed to get user profile: {}", e))
    })
}
```

## Library Benefits and Design Principles

### Benefits of Library-First Approach

1. **Reusability**: ATP can be used across multiple canisters and projects
2. **Modularity**: Clear separation between core logic, storage, and canister implementation
3. **Testability**: Libraries can be tested independently of canister environment
4. **Maintainability**: Focused responsibilities and clear boundaries
5. **Evolution**: Libraries can evolve independently with proper versioning

### Database Initialization Strategy

#### Memory Segment Allocation
The `ic-nosql` library uses a centralized memory management approach:

```rust
// Memory segments are allocated automatically
// ATP Storage uses segments 0-9 (reserved)
// Custom models use segments 10+ (user-defined)

// Example memory allocation:
// - ATP Accounts: Memory ID 0, 1
// - User Profiles: Memory ID 10, 11 
// - Transactions: Memory ID 12, 13
// - Custom Model A: Memory ID 14, 15
```

#### Initialization Flow
1. **Database Manager Creation**: Single `DatabaseManager` instance per canister
2. **ATP Storage Setup**: ATP storage registers its models automatically (Memory IDs 0-9)
3. **Custom Model Registration**: Your canister registers custom models (Memory IDs 10+)
4. **Type Safety**: All operations are type-safe and isolated per model

#### Example Integration Pattern
```rust
#[ic_cdk::init]
fn init() {
    // 1. Create database manager
    let db_manager = DatabaseManager::new();
    
    // 2. Initialize ATP (uses reserved memory segments 0-9)
    let atp_config = AtpConfig::new()
        .with_storage(AtpStorageConfig::new(&db_manager));
    let atp_client = AtpClient::new(atp_config).unwrap();
    
    // 3. Register your custom models (uses memory segments 10+)
    db_manager.register_model::<UserProfile>("users", 10, Some(11)).unwrap();
    db_manager.register_model::<Transaction>("txns", 12, Some(13)).unwrap();
    
    // 4. Store globally for use in endpoints
    store_globally(db_manager, atp_client);
}
```

### Design Principles

#### 1. **Dependency Inversion**
- Libraries depend on abstractions, not concrete implementations
- Higher-level modules don't depend on lower-level modules
- Both depend on abstractions (traits)

#### 2. **Single Responsibility**
- Each library has a single, well-defined purpose
- Clear boundaries between business logic, storage, and infrastructure

#### 3. **Open/Closed Principle**
- Libraries are open for extension but closed for modification
- New functionality can be added without changing existing code

#### 4. **Interface Segregation**
- Clients should not depend on interfaces they don't use
- Small, focused interfaces rather than large, monolithic ones

#### 5. **Minimal Dependencies**
- Core library (`atp-lib`) has minimal external dependencies
- Storage implementations contain IC-specific dependencies
- Clear separation between platform-agnostic and platform-specific code

#### 6. **Flexible Model Management**
- Models can be defined and registered independently
- Memory segments are allocated automatically
- Type-safe operations across different models
- No conflicts between ATP models and custom models

## Risk Mitigation

### Technical Risks
1. **Breaking Changes**: Extensive testing required to ensure no functionality regression
2. **Performance Impact**: Careful monitoring of abstraction overhead
3. **Build Complexity**: Workspace configuration requires careful management

### Mitigation Strategies
1. **Incremental Approach**: Phase-by-phase implementation with validation at each step
2. **Automated Testing**: Comprehensive test suite to catch regressions
3. **Feature Flags**: Ability to rollback changes if issues arise
4. **Documentation**: Clear migration guide for future developers

## Success Criteria

### Technical Success
- [ ] All packages compile independently
- [ ] No circular dependencies between packages
- [ ] Clean architecture principles followed
- [ ] All existing tests pass
- [ ] Performance benchmarks maintained

### Architectural Success
- [ ] Clear separation of concerns
- [ ] Dependency inversion principle followed
- [ ] Reusable database package
- [ ] Maintainable codebase structure
- [ ] Well-documented interfaces

## Timeline Summary

- **Weeks 1-2**: Prepare for separation (fix dependencies, implement DI)
- **Weeks 3-4**: Create package boundaries and structures
- **Weeks 5-6**: Implement package separation and move code
- **Week 7**: Update main package and build configuration
- **Week 8**: Testing, validation, and documentation

**Total Duration**: 8 weeks
**Critical Path**: Dependency inversion fixes → Package creation → Code migration → Testing
