# Participant Data Structure Implementation

## Overview

This document describes the comprehensive Participant data structure implementation for the Scavenger smart contract. The implementation provides role-based access control, location tracking, and statistics management for all participants in the recycling ecosystem.

## Data Structure

### Participant Struct

```rust
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Participant {
    pub address: Address,              // Unique participant identifier
    pub role: ParticipantRole,         // Role in the ecosystem
    pub name: Symbol,                  // Participant name
    pub latitude: i128,                // Geographic latitude (scaled)
    pub longitude: i128,               // Geographic longitude (scaled)
    pub is_registered: bool,           // Registration status
    pub total_waste_processed: u128,   // Total waste weight processed
    pub total_tokens_earned: u128,     // Total reward tokens earned
    pub registered_at: u64,            // Registration timestamp
}
```

### ParticipantRole Enum

```rust
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ParticipantRole {
    Recycler = 0,      // Processes and verifies materials
    Collector = 1,     // Collects materials from sources
    Manufacturer = 2,  // Manufactures products from recycled materials
}
```

## Key Features

### 1. Deterministic Storage

- Uses Soroban's `#[contracttype]` for deterministic serialization
- Consistent storage layout across all contract invocations
- Keyed by participant address for efficient lookups
- Maintains storage integrity with proper type annotations

### 2. Role-Based Access Control

**Recycler Permissions:**
- Can collect materials
- Can process recyclables
- Can verify material submissions
- Cannot manufacture products

**Collector Permissions:**
- Can collect materials
- Cannot process recyclables
- Cannot verify materials
- Cannot manufacture products

**Manufacturer Permissions:**
- Cannot collect materials
- Cannot process recyclables
- Cannot verify materials
- Can manufacture products

### 3. Statistics Tracking

**Overflow Protection:**
- Uses `checked_add()` for all arithmetic operations
- Prevents overflow in `total_waste_processed`
- Prevents overflow in `total_tokens_earned`
- Panics with clear error messages on overflow

**Automatic Updates:**
- `total_waste_processed` increments on material submission
- `total_tokens_earned` increments on material verification
- Batch operations update stats efficiently
- Stats persist across contract invocations

### 4. Registration Management

**Registration Flow:**
1. User calls `register_participant()` with required data
2. Contract creates Participant with `is_registered = true`
3. Participant stored in persistent storage
4. Initial stats set to zero

**Deregistration:**
- Sets `is_registered = false`
- Preserves historical data
- Prevents further restricted actions
- Can be re-registered later

### 5. Location Tracking

**Coordinate Format:**
- Latitude and longitude stored as `i128`
- Typically scaled by 1e6 for precision
- Example: 40.748817° → 40_748_817
- Supports negative coordinates for southern/western hemispheres

**Update Capability:**
- Participants can update their location
- Requires authentication
- Only registered participants can update
- Changes persist immediately

## Contract Functions

### Registration Functions

#### `register_participant`
```rust
pub fn register_participant(
    env: Env,
    address: Address,
    role: ParticipantRole,
    name: Symbol,
    latitude: i128,
    longitude: i128,
) -> Participant
```
- Registers a new participant or updates existing
- Requires address authentication
- Initializes stats to zero
- Sets `is_registered = true`

#### `deregister_participant`
```rust
pub fn deregister_participant(
    env: Env,
    address: Address,
) -> Participant
```
- Deactivates a participant
- Requires address authentication
- Sets `is_registered = false`
- Preserves historical data

### Query Functions

#### `get_participant`
```rust
pub fn get_participant(
    env: Env,
    address: Address,
) -> Option<Participant>
```
- Retrieves participant data
- Returns `None` if not found
- No authentication required

#### `can_collect`
```rust
pub fn can_collect(
    env: Env,
    address: Address,
) -> bool
```
- Checks if participant can collect materials
- Validates registration status
- Validates role permissions

#### `can_manufacture`
```rust
pub fn can_manufacture(
    env: Env,
    address: Address,
) -> bool
```
- Checks if participant can manufacture
- Validates registration status
- Validates role permissions

### Update Functions

#### `update_role`
```rust
pub fn update_role(
    env: Env,
    address: Address,
    new_role: ParticipantRole,
) -> Participant
```
- Updates participant role
- Requires address authentication
- Validates registration status
- Preserves other participant data

#### `update_location`
```rust
pub fn update_location(
    env: Env,
    address: Address,
    latitude: i128,
    longitude: i128,
) -> Participant
```
- Updates participant location
- Requires address authentication
- Validates registration status
- Preserves other participant data

### Internal Functions

#### `update_participant_stats`
```rust
fn update_participant_stats(
    env: &Env,
    address: &Address,
    waste_weight: u64,
    tokens_earned: u64,
)
```
- Internal helper for stats updates
- Uses checked arithmetic
- Updates both waste and tokens
- Called automatically by material operations

#### `require_registered`
```rust
fn require_registered(
    env: &Env,
    address: &Address,
)
```
- Validates participant registration
- Panics if not found or not registered
- Used by restricted operations
- Provides clear error messages

## Integration with Material Operations

### Material Submission
1. User calls `submit_material()`
2. Contract validates user is registered via `require_registered()`
3. Material is created and stored
4. `update_participant_stats()` increments `total_waste_processed`
5. RecyclingStats updated separately

### Material Verification
1. Recycler calls `verify_material()`
2. Contract validates recycler is registered and has correct role
3. Material is marked as verified
4. Reward points calculated
5. `update_participant_stats()` increments `total_tokens_earned`
6. RecyclingStats updated separately

### Batch Operations
- Batch submissions accumulate weight before single stats update
- Batch verifications process all materials then update stats
- Overflow protection applies to accumulated totals
- More efficient than individual operations

## Security Considerations

### Authentication
- All write operations require `address.require_auth()`
- Participants can only modify their own data
- Role-based permissions enforced at function level

### Overflow Protection
- All arithmetic uses `checked_add()`
- Panics with descriptive error on overflow
- Prevents silent wraparound bugs
- Maintains data integrity

### State Validation
- Registration status checked before restricted actions
- Role permissions validated for sensitive operations
- Invalid state transitions prevented
- Clear panic messages for debugging

### Storage Integrity
- Deterministic serialization via `#[contracttype]`
- Consistent storage keys
- No data corruption from concurrent access
- Proper type safety

## Testing

### Test Coverage

The implementation includes comprehensive tests covering:

1. **Persistence Tests**
   - `test_participant_persistence` - Verifies data persists correctly
   - `test_participant_storage_deterministic` - Ensures deterministic storage

2. **Initialization Tests**
   - `test_participant_initialization` - Validates correct initial state
   - `test_register_participant` - Tests registration flow

3. **Role-Based Access Tests**
   - `test_role_based_access_enforcement` - Validates permission checks
   - `test_can_collect` - Tests collection permissions
   - `test_can_manufacture` - Tests manufacturing permissions

4. **Statistics Tests**
   - `test_participant_stats_update` - Verifies stats increment correctly
   - `test_participant_stats_overflow_protection` - Tests overflow handling
   - `test_batch_operations_update_participant_stats` - Tests batch efficiency

5. **Registration Management Tests**
   - `test_deregister_participant` - Tests deregistration flow
   - `test_update_role` - Tests role updates
   - `test_update_location` - Tests location updates

6. **Error Handling Tests**
   - `test_submit_material_unregistered_user` - Validates registration requirement
   - `test_update_role_deregistered_user` - Tests deregistered user handling
   - `test_verify_material_deregistered_verifier` - Tests verifier validation

7. **Multi-Participant Tests**
   - `test_multiple_participants_independent_stats` - Validates stat independence
   - `test_all_role_types` - Tests all role variants

### Running Tests

```bash
cd stellar-contract
cargo test --lib
```

All tests pass with no regressions to existing functionality.

## Migration Notes

### Breaking Changes

The Participant struct has been expanded with new fields:
- `name: Symbol`
- `latitude: i128`
- `longitude: i128`
- `is_registered: bool`
- `total_waste_processed: u128`
- `total_tokens_earned: u128`

### Backward Compatibility

Existing contracts will need to:
1. Update `register_participant()` calls to include new parameters
2. Handle the new fields in client code
3. Re-register existing participants with complete data

### Storage Migration

If migrating from an older version:
1. Export existing participant data
2. Deploy new contract version
3. Re-register participants with complete information
4. Verify data integrity

## Performance Considerations

### Gas Optimization

- Batch operations minimize storage writes
- Stats updated once per operation
- Efficient storage key structure
- Minimal redundant reads

### Storage Efficiency

- Compact data types where possible
- Single storage entry per participant
- No redundant data duplication
- Efficient serialization format

## Future Enhancements

Potential improvements:
1. Reputation scoring system
2. Geographic proximity queries
3. Historical stats tracking
4. Multi-role support per participant
5. Delegation mechanisms
6. Advanced analytics

## Conclusion

The Participant implementation provides a robust, secure, and efficient foundation for managing participants in the Scavenger ecosystem. It enforces proper access control, maintains accurate statistics, and ensures data integrity through deterministic storage and overflow protection.
