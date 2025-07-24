use crate::atp::atp_test_utils::*;
use crate::test_utils::TestDataGenerator;
use atp::domain::models::account::AccountState;
use atp::domain::models::signer::{Curve, SignatureAlgorithm};

#[test]
fn test_dex_to_user_complete_flow() -> Result<(), Box<dyn std::error::Error>> {
    let env = create_atp_canister_env()?;

    // Define principals for realistic scenario
    let dex_principal = TestDataGenerator::generate_test_principal("dex");
    let user_principal = TestDataGenerator::generate_test_principal("user");
    let admin_principal = TestDataGenerator::generate_test_principal("admin");

    println!("DEX Principal: {}", dex_principal.to_string());
    println!("User Principal: {}", user_principal.to_string());
    println!("Admin Principal: {}", admin_principal.to_string());

    // Step 1: Admin creates account for user, with DEX as approved address
    let account = create_test_account(
        &env,
        SignatureAlgorithm::Ecdsa,
        Curve::Secp256k1,
        dex_principal,   // DEX is approved to transfer
        admin_principal, // Admin creates the account
    )?;

    // Verify initial state
    assert_eq!(account.account_state, AccountState::Locked);
    assert_eq!(account.owner, admin_principal.to_string());
    assert_eq!(account.approved_address, dex_principal.to_string());

    // Step 2: DEX transfers account to user (should succeed as approved address)
    let transferred_account = transfer_account(
        &env,
        &account.id,
        user_principal,
        dex_principal, // DEX calls transfer as approved address
    )?;

    // Verify transfer state
    assert_eq!(transferred_account.account_state, AccountState::Unlocked);
    assert_eq!(transferred_account.owner, user_principal.to_string());
    assert_eq!(transferred_account.approved_address, ""); // Cleared after transfer

    // Step 3: User activates their account (should succeed as owner)
    let active_account = activate_account(
        &env,
        &account.id,
        user_principal, // User activates as owner
    )?;

    // Verify final state
    assert_eq!(active_account.account_state, AccountState::Active);
    assert_eq!(active_account.owner, user_principal.to_string());

    // Step 4: User can sign messages (should succeed as owner of active account)
    let test_message = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
    let signature = sign_message(
        &env,
        &account.id,
        test_message,
        user_principal, // User signs as owner
    )?;

    assert!(!signature.is_empty());

    Ok(())
}
