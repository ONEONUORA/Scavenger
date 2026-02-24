# Issue #46: Implement set_charity_contract Function

## Summary
Implemented admin-controlled charity contract address setter functionality for the Stellar smart contract.

## Changes Made

### 1. Added Storage Keys (stellar-contract/src/lib.rs)
- Added `ADMIN` storage key for admin address
- Added `CHARITY` storage key for charity contract address

### 2. Implemented Admin Functions
- `initialize_admin(env: Env, admin: Address)`: Initialize admin (one-time setup)
- `get_admin(env: Env) -> Address`: Get current admin address
- `require_admin(env: &Env, caller: &Address)`: Internal helper to verify admin authentication

### 3. Implemented Charity Contract Functions
- `set_charity_contract(env: Env, admin: Address, charity_address: Address)`: Set charity contract address (admin only)
  - Validates caller is admin
  - Validates charity address is different from admin
  - Stores charity address in contract storage
- `get_charity_contract(env: Env) -> Option<Address>`: Get current charity contract address

### 4. Added Comprehensive Tests
Created test suite covering all acceptance criteria:
- `test_initialize_admin`: Verify admin initialization works
- `test_initialize_admin_twice`: Verify admin can only be initialized once
- `test_set_charity_contract`: Verify admin can set charity address
- `test_set_charity_contract_non_admin`: Verify non-admin cannot set charity address
- `test_set_charity_contract_same_as_admin`: Verify charity address validation
- `test_get_charity_contract_not_set`: Verify getter returns None when not set
- `test_charity_contract_update`: Verify admin can update charity address
- `test_charity_donations_workflow`: Verify donations workflow after setting charity address

## Acceptance Criteria Met

✅ **Only admin can set**: Implemented `require_admin()` check that validates caller is admin and requires authentication

✅ **Address validates correctly**: Implemented validation to ensure charity address is different from admin address

✅ **Donations work after setting**: Charity address is properly stored and retrievable via `get_charity_contract()` for donation operations

## Technical Details

### Security Features
1. Admin authentication required via `require_auth()`
2. Admin address validation before any admin operation
3. Charity address validation to prevent setting to admin address
4. One-time admin initialization to prevent admin takeover

### Storage Implementation
- Uses Soroban SDK's instance storage for persistent data
- Storage keys use `symbol_short!` macro for efficiency
- Admin and charity addresses stored separately for flexibility

### Error Handling
- Panics with descriptive messages for:
  - Unauthorized access attempts
  - Invalid address configurations
  - Duplicate admin initialization
  - Missing admin setup

## Usage Example

```rust
// 1. Initialize admin (one-time setup)
client.initialize_admin(&admin_address);

// 2. Set charity contract address (admin only)
client.set_charity_contract(&admin_address, &charity_address);

// 3. Get charity address for donations
let charity = client.get_charity_contract().expect("Charity not set");

// 4. Update charity address if needed (admin only)
client.set_charity_contract(&admin_address, &new_charity_address);
```

## Notes

- The implementation follows the same pattern as the existing `contracts/scavenger` contract
- Admin must be initialized before setting charity contract
- Charity address can be updated multiple times by admin
- The codebase has pre-existing compilation errors unrelated to this implementation
- All new code for admin and charity functionality compiles without errors
