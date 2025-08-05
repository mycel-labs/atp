use crate::atp::atp_test_utils::*;
use crate::test_utils::TestDataGenerator;
use atp_caip::curve::Curve;
use ic_atp::domain::models::account::AccountState;
use ic_atp::domain::models::signer::SignatureAlgorithm;

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
    assert_eq!(account.account.account_state, AccountState::Locked);
    assert_eq!(account.account.owner, admin_principal.to_string());
    assert_eq!(account.account.approved_address, dex_principal.to_string());

    // Verify eth address generation
    assert!(!account.account.public_key_hex.is_empty());
    let eip1559_address = generate_address(&env, &account.account.id, "eip155:1")?;
    assert!(eip1559_address.address.starts_with("0x"));

    // Verify solana address generation, shoud be error with invalid curve
    assert!(!account.account.public_key_hex.is_empty());
    let eip1559_address = generate_address(&env, &account.account.id, "solana:mainnet");
    assert!(eip1559_address.is_err());

    // Step 2: DEX transfers account to user (should succeed as approved address)
    let transferred_account = transfer_account(
        &env,
        &account.account.id,
        user_principal,
        dex_principal, // DEX calls transfer as approved address
    )?;

    // Verify transfer state
    assert_eq!(
        transferred_account.account.account_state,
        AccountState::Unlocked
    );
    assert_eq!(
        transferred_account.account.owner,
        user_principal.to_string()
    );
    assert_eq!(transferred_account.account.approved_address, ""); // Cleared after transfer

    // Step 3: User activates their account (should succeed as owner)
    let active_account = activate_account(
        &env,
        &account.account.id,
        user_principal, // User activates as owner
    )?;

    // Verify final state
    assert_eq!(active_account.account.account_state, AccountState::Active);
    assert_eq!(active_account.account.owner, user_principal.to_string());

    // Step 4: User can sign messages (should succeed as owner of active account)
    let test_message = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
    let signature_response = sign_message(
        &env,
        &account.account.id,
        test_message,
        user_principal, // User signs as owner
    )?;

    assert!(!signature_response.signature.is_empty());

    Ok(())
}
