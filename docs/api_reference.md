# ATP API Reference

This document provides a detailed reference for all ATP endpoints.

## Request/Response Types

All endpoints now use structured request and response types for better maintainability and type safety.

## Account Management

### create_account
```candid
create_account: (request: CreateAccountRequest) -> (variant { Ok: CreateAccountResponse; Err: text; });
```
Creates a new account with the specified signature algorithm, curve, and approved address. The caller becomes the owner of the account.

Request:
- `algorithm`: The signature algorithm to use (ECDSA or Schnorr)
- `curve`: The curve to use (secp256k1 or ed25519)
- `approved_address`: The principal that is approved to transfer the account

Response:
- `CreateAccountResponse` containing `AccountReply` with account details on success
- Error message on failure

### unlock_account
```candid
unlock_account: (request: UnlockAccountRequest) -> (variant { Ok: UnlockAccountResponse; Err: text; });
```
Unlocks a locked account. Only the approved address can call this method.

Request:
- `account_id`: ID of the account to unlock

Response:
- `UnlockAccountResponse` containing `AccountReply` with updated account details on success
- Error message on failure

### transfer_account
```candid
transfer_account: (request: TransferAccountRequest) -> (variant { Ok: TransferAccountResponse; Err: text; });
```
Transfers account ownership to another principal. Only the approved address can call this method, and the account must be in the Locked state.

Request:
- `account_id`: ID of the account to transfer
- `to`: Principal ID of the new owner

Response:
- `TransferAccountResponse` containing `AccountReply` with updated account details on success
- Error message on failure

### activate_account
```candid
activate_account: (request: ActivateAccountRequest) -> (variant { Ok: ActivateAccountResponse; Err: text; });
```
Activates an unlocked account. Only the owner can call this method, and the account must be in the Unlocked state.

Request:
- `account_id`: ID of the account to activate

Response:
- `ActivateAccountResponse` containing `AccountReply` with updated account details on success
- Error message on failure

### get_account
```candid
get_account: (request: GetAccountRequest) -> (variant { Ok: GetAccountResponse; Err: text; }) query;
```
Retrieves account details. Anyone can call this method.

Request:
- `account_id`: ID of the account to retrieve

Response:
- `GetAccountResponse` containing `AccountReply` with account details on success
- Error message on failure

## Signing Operations

### sign
```candid
sign: (request: SignRequest) -> (variant { Ok: SignResponse; Err: text; });
```
Signs a message with the account's private key. Only the owner can call this method, and the account must be in the Active state.

Request:
- `account_id`: ID of the account to use for signing
- `message_hex`: Hex-encoded message to sign

Response:
- `SignResponse` containing hex-encoded signature on success
- Error message on failure

### sign_eip1559_transaction
```candid
sign_eip1559_transaction: (request: SignEip1559TransactionRequest) -> (variant { Ok: SignEip1559TransactionResponse; Err: text; });
```
Signs an EIP-1559 Ethereum transaction. Only the owner can call this method, and the account must be in the Active state with ECDSA/secp256k1.

Request:
- `account_id`: ID of the account to use for signing
- `tx_request`: Transaction request details

Response:
- `SignEip1559TransactionResponse` containing hex-encoded signed transaction on success
- Error message on failure

### get_eth_address
```candid
get_eth_address: (request: GetEthAddressRequest) -> (variant { Ok: GetEthAddressResponse; Err: text; }) query;
```
Retrieves the Ethereum address derived from the account's public key. Anyone can call this method.

Request:
- `account_id`: ID of the account

Response:
- `GetEthAddressResponse` containing hex-encoded Ethereum address on success
- Error message on failure
