//! Test utilities specific to ic-nosql and example canister
//!
//! This module provides utilities for testing the example canister functionality
//! including data structures, mock data generation, and helper functions.

use candid::Encode;

use crate::test_utils::{TestConfig, TestEnvironment};
// Re-export domain models from example canister
pub use example_canister::{Comment, Post, User};

// Test data generators for example canister
pub struct ExampleCanisterTestDataGenerator;

impl ExampleCanisterTestDataGenerator {
    pub fn generate_user(index: usize, prefix: &str) -> (String, String) {
        let username = format!("{}_{}", prefix, index);
        let email = format!("{}_{}@example.com", prefix, index);
        (username, email)
    }

    pub fn generate_post(index: usize, _user_id: &str, prefix: &str) -> (String, String) {
        let title = format!("{} Post {}", prefix, index);
        let content = format!("This is the content of {} post {}", prefix, index);
        (title, content)
    }

    pub fn generate_comment(index: usize, prefix: &str) -> String {
        format!("{} comment {}", prefix, index)
    }

    pub fn generate_large_content(size: usize) -> String {
        "a".repeat(size)
    }
}

// Convenience function to create TestEnvironment for example canister
pub fn create_example_canister_env() -> Result<TestEnvironment, Box<dyn std::error::Error>> {
    TestEnvironment::new("example-canister", "example_canister")
}

pub fn create_example_canister_env_with_config(
    config: TestConfig,
) -> Result<TestEnvironment, Box<dyn std::error::Error>> {
    TestEnvironment::new_with_config("example-canister", "example_canister", config)
}

// Common test patterns for example canister
pub fn create_users_batch(
    env: &TestEnvironment,
    count: usize,
    prefix: &str,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut user_ids = Vec::new();

    for i in 0..count {
        let (username, email) = ExampleCanisterTestDataGenerator::generate_user(i, prefix);

        let result: Result<User, String> =
            env.update_call("create_user", Encode!(&username, &email).unwrap())?;

        if let Ok(user) = result {
            user_ids.push(user.id);
        }
    }

    Ok(user_ids)
}

pub fn create_posts_batch(
    env: &TestEnvironment,
    user_ids: &[String],
    posts_per_user: usize,
    prefix: &str,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut post_ids = Vec::new();

    for (_user_idx, user_id) in user_ids.iter().enumerate() {
        for post_idx in 0..posts_per_user {
            let (title, content) =
                ExampleCanisterTestDataGenerator::generate_post(post_idx, user_id, prefix);

            let result: Result<Post, String> =
                env.update_call("create_post", Encode!(user_id, &title, &content).unwrap())?;

            if let Ok(post) = result {
                post_ids.push(post.id);
            }
        }
    }

    Ok(post_ids)
}
