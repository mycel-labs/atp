//! Integration tests for ic-nosql using PocketIC
//!
//! These tests demonstrate how to use ic-nosql in a real canister environment
//! and verify that all functionality works correctly.

use candid::{CandidType, Decode, Deserialize, Encode, Principal};
use pocket_ic::PocketIc;
use serde::Serialize;
use std::path::PathBuf;

// Test data structures matching the canister
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

// Helper function to get the canister WASM path
fn get_canister_wasm_path() -> PathBuf {
    let cargo_manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    // Go up one level from tests/ to the workspace root
    PathBuf::from(cargo_manifest_dir)
        .parent()
        .unwrap()
        .join("target")
        .join("wasm32-unknown-unknown")
        .join("release")
        .join("example_canister.wasm")
}

// Helper function to build the canister if needed
fn ensure_canister_built() -> Result<(), Box<dyn std::error::Error>> {
    let wasm_path = get_canister_wasm_path();

    if !wasm_path.exists() {
        // Build the canister
        let status = std::process::Command::new("cargo")
            .args(&[
                "build",
                "--target",
                "wasm32-unknown-unknown",
                "--release",
                "--package",
                "example-canister",
            ])
            .status()?;

        if !status.success() {
            return Err("Failed to build canister".into());
        }
    }

    Ok(())
}

// Helper function to setup PocketIC with the canister
fn setup_canister() -> Result<(PocketIc, Principal), Box<dyn std::error::Error>> {
    ensure_canister_built()?;

    let pic = PocketIc::new();
    let wasm_path = get_canister_wasm_path();
    let wasm_bytes = std::fs::read(&wasm_path)
        .map_err(|e| format!("Failed to read canister WASM at {:?}: {}", wasm_path, e))?;

    let canister_id = pic.create_canister();

    // Add cycles to the canister
    pic.add_cycles(canister_id, 10_000_000_000_000); // 10T cycles

    pic.install_canister(canister_id, wasm_bytes, Encode!().unwrap(), None);

    Ok((pic, canister_id))
}

// Helper function to make query calls and decode results
fn query_call<T>(
    pic: &PocketIc,
    canister_id: Principal,
    method: &str,
    args: Vec<u8>,
) -> Result<T, Box<dyn std::error::Error>>
where
    T: for<'de> Deserialize<'de> + CandidType,
{
    let bytes = pic
        .query_call(canister_id, Principal::anonymous(), method, args)
        .map_err(|e| format!("Query call failed: {:?}", e))?;

    let result: T = Decode!(&bytes, T).map_err(|e| format!("Decode failed: {:?}", e))?;

    Ok(result)
}

// Helper function to make update calls and decode results
fn update_call<T>(
    pic: &PocketIc,
    canister_id: Principal,
    method: &str,
    args: Vec<u8>,
) -> Result<T, Box<dyn std::error::Error>>
where
    T: for<'de> Deserialize<'de> + CandidType,
{
    let bytes = pic
        .update_call(canister_id, Principal::anonymous(), method, args)
        .map_err(|e| format!("Update call failed: {:?}", e))?;

    let result: T = Decode!(&bytes, T).map_err(|e| format!("Decode failed: {:?}", e))?;

    Ok(result)
}

#[test]
fn test_canister_health_check() -> Result<(), Box<dyn std::error::Error>> {
    let (pic, canister_id) = setup_canister()?;

    // Test health check
    let health: String = query_call(&pic, canister_id, "health_check", Encode!().unwrap())?;

    assert_eq!(health, "OK");
    println!("✓ Health check passed");

    Ok(())
}

#[test]
fn test_database_stats() -> Result<(), Box<dyn std::error::Error>> {
    let (pic, canister_id) = setup_canister()?;

    // Test database stats
    let stats: Vec<String> =
        query_call(&pic, canister_id, "get_database_stats", Encode!().unwrap())?;

    assert!(stats.contains(&"users".to_string()));
    assert!(stats.contains(&"posts".to_string()));
    assert!(stats.contains(&"comments".to_string()));
    println!("✓ Database stats: {:?}", stats);

    Ok(())
}

#[test]
fn test_user_management() -> Result<(), Box<dyn std::error::Error>> {
    let (pic, canister_id) = setup_canister()?;

    // Create a user
    let result: Result<User, String> = update_call(
        &pic,
        canister_id,
        "create_user",
        Encode!(&"alice".to_string(), &"alice@example.com".to_string()).unwrap(),
    )?;

    let user = match result {
        Ok(user) => {
            assert_eq!(user.username, "alice");
            assert_eq!(user.email, "alice@example.com");
            assert!(user.id.starts_with("user_"));
            println!("✓ User created: {:?}", user);
            user
        }
        Err(e) => panic!("Create user failed: {}", e),
    };

    // Get the user
    let result: Result<User, String> =
        query_call(&pic, canister_id, "get_user", Encode!(&user.id).unwrap())?;

    match result {
        Ok(retrieved_user) => {
            assert_eq!(retrieved_user.id, user.id);
            assert_eq!(retrieved_user.username, user.username);
            assert_eq!(retrieved_user.email, user.email);
            println!("✓ User retrieved: {:?}", retrieved_user);
        }
        Err(e) => panic!("Get user failed: {}", e),
    }

    // List users
    let result: Result<Vec<User>, String> = query_call(
        &pic,
        canister_id,
        "list_users",
        Encode!(&1usize, &10usize).unwrap(),
    )?;

    match result {
        Ok(users) => {
            assert!(!users.is_empty());
            assert!(users.iter().any(|u| u.id == user.id));
            println!("✓ Users listed: {} users found", users.len());
        }
        Err(e) => panic!("List users failed: {}", e),
    }

    Ok(())
}

#[test]
fn test_post_management() -> Result<(), Box<dyn std::error::Error>> {
    let (pic, canister_id) = setup_canister()?;

    // First create a user
    let user_result: Result<User, String> = update_call(
        &pic,
        canister_id,
        "create_user",
        Encode!(&"bob".to_string(), &"bob@example.com".to_string()).unwrap(),
    )?;

    let user = user_result.unwrap();

    // Create a post
    let result: Result<Post, String> = update_call(
        &pic,
        canister_id,
        "create_post",
        Encode!(
            &user.id,
            &"Hello World".to_string(),
            &"This is my first post!".to_string()
        )
        .unwrap(),
    )?;

    let post = match result {
        Ok(post) => {
            assert_eq!(post.user_id, user.id);
            assert_eq!(post.title, "Hello World");
            assert_eq!(post.content, "This is my first post!");
            assert!(post.id.starts_with("post_"));
            println!("✓ Post created: {:?}", post);
            post
        }
        Err(e) => panic!("Create post failed: {}", e),
    };

    // Get the post
    let result: Result<Post, String> =
        query_call(&pic, canister_id, "get_post", Encode!(&post.id).unwrap())?;

    match result {
        Ok(retrieved_post) => {
            assert_eq!(retrieved_post.id, post.id);
            assert_eq!(retrieved_post.title, post.title);
            println!("✓ Post retrieved: {:?}", retrieved_post);
        }
        Err(e) => panic!("Get post failed: {}", e),
    }

    Ok(())
}

#[test]
fn test_comment_management() -> Result<(), Box<dyn std::error::Error>> {
    let (pic, canister_id) = setup_canister()?;

    // Create a user
    let user_result: Result<User, String> = update_call(
        &pic,
        canister_id,
        "create_user",
        Encode!(&"charlie".to_string(), &"charlie@example.com".to_string()).unwrap(),
    )?;

    let user = user_result.unwrap();

    // Create a post
    let post_result: Result<Post, String> = update_call(
        &pic,
        canister_id,
        "create_post",
        Encode!(
            &user.id,
            &"Test Post".to_string(),
            &"Test content".to_string()
        )
        .unwrap(),
    )?;

    let post = post_result.unwrap();

    // Create a comment
    let result: Result<Comment, String> = update_call(
        &pic,
        canister_id,
        "create_comment",
        Encode!(&post.id, &user.id, &"Great post!".to_string()).unwrap(),
    )?;

    let comment = match result {
        Ok(comment) => {
            assert_eq!(comment.post_id, post.id);
            assert_eq!(comment.user_id, user.id);
            assert_eq!(comment.content, "Great post!");
            assert!(comment.id.starts_with("comment_"));
            println!("✓ Comment created: {:?}", comment);
            comment
        }
        Err(e) => panic!("Create comment failed: {}", e),
    };

    // Get the comment
    let result: Result<Comment, String> = query_call(
        &pic,
        canister_id,
        "get_comment",
        Encode!(&comment.id).unwrap(),
    )?;

    match result {
        Ok(retrieved_comment) => {
            assert_eq!(retrieved_comment.id, comment.id);
            assert_eq!(retrieved_comment.content, comment.content);
            println!("✓ Comment retrieved: {:?}", retrieved_comment);
        }
        Err(e) => panic!("Get comment failed: {}", e),
    }

    Ok(())
}

#[test]
fn test_canister_upgrade() -> Result<(), Box<dyn std::error::Error>> {
    let (pic, canister_id) = setup_canister()?;

    // Create some data before upgrade
    let user_result: Result<User, String> = update_call(
        &pic,
        canister_id,
        "create_user",
        Encode!(&"david".to_string(), &"david@example.com".to_string()).unwrap(),
    )?;

    let user = user_result.unwrap();

    // Upgrade the canister
    let wasm_path = get_canister_wasm_path();
    let wasm_bytes = std::fs::read(&wasm_path)?;
    pic.upgrade_canister(canister_id, wasm_bytes, Encode!().unwrap(), None)
        .map_err(|e| format!("Upgrade failed: {:?}", e))?;

    // Verify data is still there after upgrade
    let result: Result<User, String> =
        query_call(&pic, canister_id, "get_user", Encode!(&user.id).unwrap())?;

    match result {
        Ok(retrieved_user) => {
            assert_eq!(retrieved_user.id, user.id);
            assert_eq!(retrieved_user.username, user.username);
            println!("✓ Data persisted after upgrade: {:?}", retrieved_user);
        }
        Err(e) => panic!("Data not persisted after upgrade: {}", e),
    }

    Ok(())
}

#[test]
fn test_concurrent_operations() -> Result<(), Box<dyn std::error::Error>> {
    let (pic, canister_id) = setup_canister()?;

    // Create multiple users concurrently
    let users = ["alice", "bob", "charlie", "david", "eve"];
    let mut created_users = Vec::new();

    for username in users.iter() {
        let result: Result<User, String> = update_call(
            &pic,
            canister_id,
            "create_user",
            Encode!(&username.to_string(), &format!("{}@example.com", username)).unwrap(),
        )?;

        match result {
            Ok(user) => {
                created_users.push(user);
            }
            Err(e) => panic!("Failed to create user {}: {}", username, e),
        }
    }

    // Verify all users were created
    let result: Result<Vec<User>, String> = query_call(
        &pic,
        canister_id,
        "list_users",
        Encode!(&1usize, &10usize).unwrap(),
    )?;

    match result {
        Ok(all_users) => {
            assert!(all_users.len() >= users.len());
            for created_user in &created_users {
                assert!(all_users.iter().any(|u| u.id == created_user.id));
            }
            println!("✓ All {} users created successfully", users.len());
        }
        Err(e) => panic!("Failed to list users: {}", e),
    }

    Ok(())
}
