# ATP API Reference

This document provides a detailed reference for all ATP endpoints.

## Account Management

### create_account
```candid
create_account: (algorithm: SignatureAlgorithm, curve: Curve, approved_address: principal) -> (variant { Ok: AccountReply; Err: text; });
```
Creates a new account with the specified signature algorithm, curve, and approved address. The caller becomes the owner of the account.

Parameters:
- `algorithm`: The signature algorithm to use (ECDSA or Schnorr)
- `curve`: The curve to use (secp256k1 or ed25519)
- `approved_address`: The principal that is approved to transfer the account

Returns:
- `AccountReply` with account details on success
- Error message on failure

### unlock_account
```candid
unlock_account: (account_id: text) -> (variant { Ok: AccountReply; Err: text; });
```
Unlocks a locked account. Only the approved address can call this method.

Parameters:
- `account_id`: ID of the account to unlock

Returns:
- `AccountReply` with updated account details on success
- Error message on failure

### transfer_account
```candid
transfer_account: (account_id: text, to: principal) -> (variant { Ok: AccountReply; Err: text; });
```
Transfers account ownership to another principal. Only the approved address can call this method, and the account must be in the Locked state.

Parameters:
- `account_id`: ID of the account to transfer
- `to`: Principal ID of the new owner

Returns:
- `AccountReply` with updated account details on success
- Error message on failure

### activate_account
```candid
activate_account: (account_id: text) -> (variant { Ok: AccountReply; Err: text; });
```
Activates an unlocked account. Only the owner can call this method, and the account must be in the Unlocked state.

Parameters:
- `account_id`: ID of the account to activate

Returns:
- `AccountReply` with updated account details on success
- Error message on failure

### get_account
```candid
get_account: (account_id: text) -> (variant { Ok: AccountReply; Err: text; }) query;
```
Retrieves account details. Anyone can call this method.

Parameters:
- `account_id`: ID of the account to retrieve

Returns:
- `AccountReply` with account details on success
- Error message on failure

## Signing Operations

### sign
```candid
sign: (account_id: text, message_hex: text) -> (variant { Ok: text; Err: text; });
```
Signs a message with the account's private key. Only the owner can call this method, and the account must be in the Active state.

Parameters:
- `account_id`: ID of the account to use for signing
- `message_hex`: Hex-encoded message to sign

Returns:
- Hex-encoded signature on success
- Error message on failure

### sign_eip1559_transaction
```candid
sign_eip1559_transaction: (account_id: text, tx_request: Eip1559TransactionRequestDTO) -> (variant { Ok: text; Err: text; });
```
Signs an EIP-1559 Ethereum transaction. Only the owner can call this method, and the account must be in the Active state with ECDSA/secp256k1.

Parameters:
- `account_id`: ID of the account to use for signing
- `tx_request`: Transaction request details

Returns:
- Hex-encoded signed transaction on success
- Error message on failure

### get_eth_address
```candid
get_eth_address: (account_id: text) -> (variant { Ok: text; Err: text; }) query;
```
Retrieves the Ethereum address derived from the account's public key. Anyone can call this method.

Parameters:
- `account_id`: ID of the account

Returns:
- Hex-encoded Ethereum address on success
- Error message on failure
