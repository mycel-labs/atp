//! Example canister demonstrating ic-nosql usage
//!
//! This canister shows how to use ic-nosql to manage different types of data
//! in a single canister with proper memory management.

use ic_cdk::api::management_canister::main::raw_rand;
use ic_cdk_macros::{init, post_upgrade, pre_upgrade, query, update};
use ic_nosql::{CandidType, DatabaseManager, Deserialize, Serialize};
use std::cell::RefCell;

// Example models
#[derive(Debug, Clone, CandidType, Deserialize, Serialize)]
pub struct User {
    pub id: String,
    pub username: String,
    pub email: String,
    pub created_at: u64,
}

#[derive(Debug, Clone, CandidType, Deserialize, Serialize)]
pub struct Post {
    pub id: String,
    pub user_id: String,
    pub title: String,
    pub content: String,
    pub created_at: u64,
}

#[derive(Debug, Clone, CandidType, Deserialize, Serialize)]
pub struct Comment {
    pub id: String,
    pub post_id: String,
    pub user_id: String,
    pub content: String,
    pub created_at: u64,
}

// Global database manager
thread_local! {
    static DB_MANAGER: RefCell<DatabaseManager> = RefCell::new(DatabaseManager::new());
}

#[init]
fn init() {
    DB_MANAGER.with(|db| {
        let db = db.borrow();

        // Register models with specific memory ranges
        db.register_model("users", Some(10), None)
            .expect("Failed to register users model");
        db.register_model("posts", Some(11), None)
            .expect("Failed to register posts model");
        db.register_model("comments", Some(12), None)
            .expect("Failed to register comments model");
    });
}

#[pre_upgrade]
fn pre_upgrade() {
    // Stable memory is automatically handled by ic-nosql
    ic_cdk::println!("Pre-upgrade: Database state is preserved in stable memory");
}

#[post_upgrade]
fn post_upgrade() {
    // Re-register models after upgrade
    init();
    ic_cdk::println!("Post-upgrade: Models re-registered");
}

// User management functions
#[update]
async fn create_user(username: String, email: String) -> Result<User, String> {
    // Generate a simple ID (in production, use a proper ID generation strategy)
    let random_bytes = raw_rand()
        .await
        .map_err(|e| format!("Failed to generate random bytes: {:?}", e))?;
    let id = format!("user_{}", hex::encode(&random_bytes.0[..8]));

    let user = User {
        id: id.clone(),
        username,
        email,
        created_at: ic_cdk::api::time(),
    };

    DB_MANAGER.with(|db| {
        let db = db.borrow();
        db.insert("users", &id, &user)?;
        Ok(user)
    })
}

#[query]
fn get_user(id: String) -> Result<User, String> {
    DB_MANAGER.with(|db| {
        let db = db.borrow();
        db.get::<User>("users", &id)
    })
}

#[query]
fn list_users(page: usize, size: usize) -> Result<Vec<User>, String> {
    DB_MANAGER.with(|db| {
        let db = db.borrow();
        let response = db.query::<User>("users", size, page)?;
        Ok(response.results.into_iter().map(|doc| doc.data).collect())
    })
}

// Post management functions
#[update]
async fn create_post(user_id: String, title: String, content: String) -> Result<Post, String> {
    // Verify user exists
    let _user = get_user(user_id.clone())?;

    // Generate post ID
    let random_bytes = raw_rand()
        .await
        .map_err(|e| format!("Failed to generate random bytes: {:?}", e))?;
    let id = format!("post_{}", hex::encode(&random_bytes.0[..8]));

    let post = Post {
        id: id.clone(),
        user_id,
        title,
        content,
        created_at: ic_cdk::api::time(),
    };

    DB_MANAGER.with(|db| {
        let db = db.borrow();
        db.insert("posts", &id, &post)?;
        Ok(post)
    })
}

#[query]
fn get_post(id: String) -> Result<Post, String> {
    DB_MANAGER.with(|db| {
        let db = db.borrow();
        db.get::<Post>("posts", &id)
    })
}

#[query]
fn list_posts(page: usize, size: usize) -> Result<Vec<Post>, String> {
    DB_MANAGER.with(|db| {
        let db = db.borrow();
        let response = db.query::<Post>("posts", size, page)?;
        Ok(response.results.into_iter().map(|doc| doc.data).collect())
    })
}

// Comment management functions
#[update]
async fn create_comment(
    post_id: String,
    user_id: String,
    content: String,
) -> Result<Comment, String> {
    // Verify post and user exist
    let _post = get_post(post_id.clone())?;
    let _user = get_user(user_id.clone())?;

    // Generate comment ID
    let random_bytes = raw_rand()
        .await
        .map_err(|e| format!("Failed to generate random bytes: {:?}", e))?;
    let id = format!("comment_{}", hex::encode(&random_bytes.0[..8]));

    let comment = Comment {
        id: id.clone(),
        post_id,
        user_id,
        content,
        created_at: ic_cdk::api::time(),
    };

    DB_MANAGER.with(|db| {
        let db = db.borrow();
        db.insert("comments", &id, &comment)?;
        Ok(comment)
    })
}

#[query]
fn get_comment(id: String) -> Result<Comment, String> {
    DB_MANAGER.with(|db| {
        let db = db.borrow();
        db.get::<Comment>("comments", &id)
    })
}

#[query]
fn list_comments(page: usize, size: usize) -> Result<Vec<Comment>, String> {
    DB_MANAGER.with(|db| {
        let db = db.borrow();
        let response = db.query::<Comment>("comments", size, page)?;
        Ok(response.results.into_iter().map(|doc| doc.data).collect())
    })
}

// Database statistics
#[query]
fn get_database_stats() -> Vec<String> {
    DB_MANAGER.with(|db| {
        let db = db.borrow();
        db.list_models()
    })
}

// Health check
#[query]
fn health_check() -> String {
    "OK".to_string()
}

// Export the Candid interface
ic_cdk::export_candid!();
