# Participants Storage Implementation

## Overview

The participants storage system provides a robust map-based storage solution for managing participant records in the Scavngr smart contract. It ensures data integrity, prevents duplicate registrations, and provides efficient access to participant information.

## Storage Architecture

### Storage Key Structure
```rust
Key: (Address,) → Participant
```

Participants are stored using their address as the key, enabling O(1) lookup performance.

## Core Functions

### Registration Functions

#### `register_participant`
```rust
pub fn register_participant(
    env: Env,
    address: Address,
    role: ParticipantRole,
) -> Participant
```

Registers a new participant with the specified role.

**Features:**
- Requires authentication from the registering address
- Prevents duplicate registrations
- Records registration timestamp
- Returns the created Participant struct

**Panics:**
- If participant is already registered

**Example:**
```rust
let participant = client.register_participant(
    &user_address,
    &ParticipantRole::Recycler
);
```

#### `is_participant_registered`
```rust
pub fn is_participant_registered(env: Env, address: Address) -> bool
```

Checks if an address is registered as a participant.

**Returns:** `true` if registered, `false` otherwise

### Storage Helper Functions

#### `set_participant` (Internal)
```rust
fn set_participant(env: &Env, address: &Address, participant: &Participant)
```

Internal helper function for storing participant records efficiently.

**Features:**
- Encapsulates storage key logic
- Used by registration and update functions
- Ensures consistent storage patterns

### Retrieval Functions

#### `get_participant`
```rust
pub fn get_participant(env: Env, address: Address) -> Option<Participant>
```

Retrieves participant information by address.

**Returns:** `Some(Participant)` if found, `None` otherwise

### Update Functions

#### `update_role`
```rust
pub fn update_role(
    env: Env,
    address: Address,
    new_role: ParticipantRole
) -> Participant
```

Updates a participant's role while preserving other data.

**Features:**
- Requires authentication from the participant
- Preserves registration timestamp
- Validates participant exists before update

**Panics:**
- If participant not found

## Data Integrity

### Duplicate Prevention
The system prevents duplicate registrations by checking existence before creating new records:

```rust
if Self::is_participant_registered(env.clone(), address.clone()) {
    panic!("Participant already registered");
}
```

### Data Preservation
Updates preserve critical data like registration timestamps:

```rust
// Original registration_at is maintained
participant.role = new_role;
Self::set_participant(&env, &address, &participant);
```

## Usage Examples

### Basic Registration Flow
```rust
// Register a recycler
let recycler = client.register_participant(
    &recycler_address,
    &ParticipantRole::Recycler
);

// Check registration status
let is_registered = client.is_participant_registered(&recycler_address);
assert!(is_registered);

// Retrieve participant info
let participant = client.get_participant(&recycler_address).unwrap();
assert_eq!(participant.role, ParticipantRole::Recycler);
```

### Role Update Flow
```rust
// Update role from Recycler to Collector
let updated = client.update_role(
    &user_address,
    &ParticipantRole::Collector
);

assert_eq!(updated.role, ParticipantRole::Collector);
// registration_at remains unchanged
```

### Validation Flow
```rust
// Check if user can collect materials
let can_collect = client.can_collect(&user_address);

// Check if user can manufacture
let can_manufacture = client.can_manufacture(&user_address);
```

## Performance Characteristics

- **Registration:** O(1) - Single storage write
- **Lookup:** O(1) - Direct key access
- **Update:** O(1) - Single storage read + write
- **Existence Check:** O(1) - Storage has() operation

## Security Considerations

1. **Authentication Required:** All write operations require address authentication
2. **Duplicate Prevention:** System prevents multiple registrations for same address
3. **Data Validation:** Role updates validate participant existence
4. **Immutable History:** Registration timestamps cannot be modified

## Testing

The implementation includes comprehensive tests:

- `test_register_participant` - Basic registration
- `test_register_participant_duplicate` - Duplicate prevention
- `test_is_participant_registered` - Registration checks
- `test_get_participant` - Data retrieval
- `test_update_role` - Role updates
- `test_all_role_types` - All role variants

## Integration with Other Systems

### Material Submission
Participants must be registered to submit materials:
```rust
// Participant registration is checked during transfers
if !Self::is_participant_registered(env.clone(), address.clone()) {
    panic!("Participant not registered");
}
```

### Waste Verification
Only recyclers can verify materials:
```rust
let participant = Self::get_participant(env.clone(), verifier.clone())
    .expect("Verifier not registered");

if !participant.role.can_process_recyclables() {
    panic!("Only recyclers can verify materials");
}
```

### Waste Transfers
Both sender and receiver must be registered:
```rust
if !Self::is_participant_registered(env.clone(), from.clone()) {
    panic!("Sender not registered");
}
if !Self::is_participant_registered(env.clone(), to.clone()) {
    panic!("Receiver not registered");
}
```

## Future Enhancements

Potential improvements for future versions:

1. **Batch Operations:** Register multiple participants in one transaction
2. **Role History:** Track role changes over time
3. **Participant Metadata:** Add optional profile information
4. **Deactivation:** Soft-delete functionality for inactive participants
5. **Indexing:** Efficient queries by role type
