//! Refactored simple stress tests for ic-nosql using test utilities and parallel operations
//!
//! These tests evaluate the basic performance and reliability of ic-nosql
//! under moderate load with improved code organization and parallel processing.

use crate::{
    ic_nosql::ic_nosql_test_utils::{
        create_example_canister_env, create_posts_batch, create_users_batch,
        ExampleCanisterTestDataGenerator, User,
    },
    test_utils::{assert_success_rate, PerformanceMetrics},
};
use candid::Encode;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Instant;

#[test]
fn stress_test_create_many_users() -> Result<(), Box<dyn std::error::Error>> {
    let env = create_example_canister_env()?;
    let mut metrics = PerformanceMetrics::new();

    const NUM_USERS: usize = 100;
    println!("Creating {} users...", NUM_USERS);

    let start_time = Instant::now();

    for i in 0..NUM_USERS {
        let (username, email) = ExampleCanisterTestDataGenerator::generate_user(i, "stress_user");

        let (duration, result) = env.timed_update_call::<Result<User, String>>(
            "create_user",
            Encode!(&username, &email).unwrap(),
        );

        let success = result.is_ok() && result.unwrap().is_ok();
        metrics.record_operation(duration, success);

        if i % 10 == 0 {
            println!("Created {} users so far...", i);
        }
    }

    let total_duration = start_time.elapsed();
    println!("Total test execution time: {:?}", total_duration);

    metrics.print_summary("User Creation");

    // Verify success rate
    assert_success_rate(
        metrics.successful_operations,
        metrics.total_operations,
        0.8,
        "User creation",
    );

    Ok(())
}

#[test]
fn stress_test_create_posts_with_users() -> Result<(), Box<dyn std::error::Error>> {
    let env = create_example_canister_env()?;

    const NUM_USERS: usize = 10;
    const POSTS_PER_USER: usize = 5;

    println!(
        "Creating {} users with {} posts each...",
        NUM_USERS, POSTS_PER_USER
    );

    let start_time = Instant::now();

    // Create users first
    let user_ids = create_users_batch(&env, NUM_USERS, "post_user")?;

    // Create posts for each user
    let post_ids = create_posts_batch(&env, &user_ids, POSTS_PER_USER, "Test")?;

    let duration = start_time.elapsed();
    println!("Created {} posts in {:?}", post_ids.len(), duration);

    // Verify success rate
    let expected_posts = NUM_USERS * POSTS_PER_USER;
    assert_success_rate(post_ids.len(), expected_posts, 0.8, "Post creation");

    Ok(())
}

#[test]
fn stress_test_parallel_user_creation() -> Result<(), Box<dyn std::error::Error>> {
    let env = Arc::new(create_example_canister_env()?);

    const NUM_THREADS: usize = 4;
    const USERS_PER_THREAD: usize = 25;
    const TOTAL_USERS: usize = NUM_THREADS * USERS_PER_THREAD;

    println!(
        "Creating {} users in parallel using {} threads...",
        TOTAL_USERS, NUM_THREADS
    );

    let start_time = Instant::now();
    let results = Arc::new(Mutex::new(Vec::new()));
    let mut handles = Vec::new();

    for thread_id in 0..NUM_THREADS {
        let env_clone = env.clone();
        let results_clone = results.clone();

        let handle = thread::spawn(move || {
            let mut local_results = Vec::new();

            for i in 0..USERS_PER_THREAD {
                let user_index = thread_id * USERS_PER_THREAD + i;
                let (username, email) =
                    ExampleCanisterTestDataGenerator::generate_user(user_index, "parallel_user");

                let start = Instant::now();
                let result: Result<Result<User, String>, _> =
                    env_clone.update_call("create_user", Encode!(&username, &email).unwrap());
                let duration = start.elapsed();

                let success = result.is_ok() && result.unwrap().is_ok();
                local_results.push((duration, success));
            }

            let mut global_results = results_clone.lock().unwrap();
            global_results.extend(local_results);
        });

        handles.push(handle);
    }

    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }

    let total_duration = start_time.elapsed();
    let results = results.lock().unwrap();

    // Calculate metrics
    let mut metrics = PerformanceMetrics::new();
    for (duration, success) in results.iter() {
        metrics.record_operation(*duration, *success);
    }

    println!("Parallel user creation completed in {:?}", total_duration);
    metrics.print_summary("Parallel User Creation");

    // Verify success rate
    assert_success_rate(
        metrics.successful_operations,
        metrics.total_operations,
        0.8,
        "Parallel user creation",
    );

    Ok(())
}

#[test]
fn stress_test_parallel_mixed_operations() -> Result<(), Box<dyn std::error::Error>> {
    let env = Arc::new(create_example_canister_env()?);

    const NUM_THREADS: usize = 3;
    const OPERATIONS_PER_THREAD: usize = 15;

    println!(
        "Running parallel mixed operations with {} threads...",
        NUM_THREADS
    );

    let start_time = Instant::now();
    let results = Arc::new(Mutex::new(Vec::new()));
    let mut handles = Vec::new();

    // Create some initial users for read operations
    let initial_users = create_users_batch(&env, 5, "initial_user")?;

    for thread_id in 0..NUM_THREADS {
        let env_clone = env.clone();
        let results_clone = results.clone();
        let initial_users_clone = initial_users.clone();

        let handle = thread::spawn(move || {
            let mut local_results = Vec::new();

            for i in 0..OPERATIONS_PER_THREAD {
                let operation_index = thread_id * OPERATIONS_PER_THREAD + i;

                // Mix of write and read operations
                let (duration, success) = if i % 3 == 0 {
                    // Write operation: Create user
                    let (username, email) = ExampleCanisterTestDataGenerator::generate_user(
                        operation_index,
                        "mixed_user",
                    );
                    let start = Instant::now();
                    let result: Result<Result<User, String>, _> =
                        env_clone.update_call("create_user", Encode!(&username, &email).unwrap());
                    let duration = start.elapsed();
                    let success = result.is_ok() && result.unwrap().is_ok();
                    (duration, success)
                } else if i % 3 == 1 {
                    // Read operation: Get database stats
                    let start = Instant::now();
                    let result: Result<Vec<String>, _> =
                        env_clone.query_call("get_database_stats", Encode!().unwrap());
                    let duration = start.elapsed();
                    let success = result.is_ok() && !result.unwrap().is_empty();
                    (duration, success)
                } else {
                    // Read operation: Get existing user
                    if !initial_users_clone.is_empty() {
                        let user_id = &initial_users_clone[i % initial_users_clone.len()];
                        let start = Instant::now();
                        let result: Result<Result<User, String>, _> =
                            env_clone.query_call("get_user", Encode!(user_id).unwrap());
                        let duration = start.elapsed();
                        let success = result.is_ok() && result.unwrap().is_ok();
                        (duration, success)
                    } else {
                        // Fallback to stats if no users available
                        let start = Instant::now();
                        let result: Result<Vec<String>, _> =
                            env_clone.query_call("get_database_stats", Encode!().unwrap());
                        let duration = start.elapsed();
                        let success = result.is_ok();
                        (duration, success)
                    }
                };

                local_results.push((duration, success));
            }

            let mut global_results = results_clone.lock().unwrap();
            global_results.extend(local_results);
        });

        handles.push(handle);
    }

    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }

    let total_duration = start_time.elapsed();
    let results = results.lock().unwrap();

    // Calculate metrics
    let mut metrics = PerformanceMetrics::new();
    for (duration, success) in results.iter() {
        metrics.record_operation(*duration, *success);
    }

    println!(
        "Parallel mixed operations completed in {:?}",
        total_duration
    );
    metrics.print_summary("Parallel Mixed Operations");

    // Verify success rate
    assert_success_rate(
        metrics.successful_operations,
        metrics.total_operations,
        0.8,
        "Parallel mixed operations",
    );

    Ok(())
}

#[test]
fn stress_test_concurrent_operations() -> Result<(), Box<dyn std::error::Error>> {
    let env = create_example_canister_env()?;
    let mut write_metrics = PerformanceMetrics::new();
    let mut read_metrics = PerformanceMetrics::new();

    const NUM_ITERATIONS: usize = 20;

    println!(
        "Running {} concurrent read/write operations...",
        NUM_ITERATIONS
    );

    let start_time = Instant::now();

    for i in 0..NUM_ITERATIONS {
        // Write operation: Create user
        let (username, email) =
            ExampleCanisterTestDataGenerator::generate_user(i, "concurrent_user");

        let (duration, result) = env.timed_update_call::<Result<User, String>>(
            "create_user",
            Encode!(&username, &email).unwrap(),
        );

        let success = result.is_ok() && result.unwrap().is_ok();
        write_metrics.record_operation(duration, success);

        // Read operation: Get database stats
        let (duration, result) =
            env.timed_query_call::<Vec<String>>("get_database_stats", Encode!().unwrap());

        let success = result.is_ok() && !result.unwrap().is_empty();
        read_metrics.record_operation(duration, success);
    }

    let total_duration = start_time.elapsed();
    println!("Total test execution time: {:?}", total_duration);

    write_metrics.print_summary("Concurrent Write Operations");
    read_metrics.print_summary("Concurrent Read Operations");

    // Verify success rates
    assert_success_rate(
        write_metrics.successful_operations,
        write_metrics.total_operations,
        0.8,
        "Concurrent writes",
    );

    assert_success_rate(
        read_metrics.successful_operations,
        read_metrics.total_operations,
        0.9,
        "Concurrent reads",
    );

    Ok(())
}

#[test]
fn stress_test_pagination() -> Result<(), Box<dyn std::error::Error>> {
    let env = create_example_canister_env()?;

    const NUM_USERS: usize = 25;
    const PAGE_SIZE: usize = 5;

    println!("Setting up {} users for pagination test...", NUM_USERS);

    let start_time = Instant::now();

    // Create users
    let _user_ids = create_users_batch(&env, NUM_USERS, "paginated_user")?;

    println!("Testing pagination performance...");

    // Test pagination
    let total_pages = (NUM_USERS + PAGE_SIZE - 1) / PAGE_SIZE;
    let mut total_retrieved = 0;
    let mut page_metrics = PerformanceMetrics::new();

    for page in 1..=total_pages {
        let (duration, result) = env.timed_query_call::<Result<Vec<User>, String>>(
            "list_users",
            Encode!(&page, &PAGE_SIZE).unwrap(),
        );

        let success = result.is_ok() && result.as_ref().unwrap().is_ok();
        page_metrics.record_operation(duration, success);

        if let Ok(Ok(users)) = result {
            total_retrieved += users.len();
            println!("Page {}: Retrieved {} users", page, users.len());
        }
    }

    let total_duration = start_time.elapsed();
    println!("Total test execution time: {:?}", total_duration);

    page_metrics.print_summary("Pagination Performance");

    // Verify we retrieved users
    assert!(
        total_retrieved > 0,
        "No users were retrieved through pagination"
    );

    // Verify pagination success rate
    assert_success_rate(
        page_metrics.successful_operations,
        page_metrics.total_operations,
        0.9,
        "Pagination operations",
    );

    Ok(())
}

#[test]
fn stress_test_canister_upgrade() -> Result<(), Box<dyn std::error::Error>> {
    let env = create_example_canister_env()?;

    const NUM_USERS: usize = 5;

    println!("Setting up {} users before upgrade...", NUM_USERS);

    let start_time = Instant::now();

    // Create users
    let user_ids = create_users_batch(&env, NUM_USERS, "upgrade_user")?;

    println!("Performing canister upgrade...");

    // Perform upgrade
    let upgrade_start = Instant::now();
    env.upgrade_canister()?;
    let upgrade_duration = upgrade_start.elapsed();

    println!("Upgrade completed in {:?}", upgrade_duration);

    println!("Verifying data integrity after upgrade...");

    // Verify users still exist
    let verified_count = env.verify_entities_exist::<User>(&user_ids, "get_user")?;

    let total_duration = start_time.elapsed();
    println!("Total test execution time: {:?}", total_duration);

    println!(
        "Verified {} out of {} users after upgrade",
        verified_count,
        user_ids.len()
    );

    // Verify data persisted
    assert!(verified_count > 0, "No users were found after upgrade");

    // Verify most data persisted
    assert_success_rate(
        verified_count,
        user_ids.len(),
        0.8,
        "Data persistence after upgrade",
    );

    Ok(())
}
