# Scavenger Contract - Participant Registration

This contract implements participant registration for the Scavenger recycling system, allowing users to register as recyclers, collectors, or manufacturers.

## Features

### Participant Registration

The `register_participant` function allows users to register with the following information:
- Role (Recycler, Collector, or Manufacturer)
- Name
- Geographic location (latitude, longitude)
- Automatic timestamp of registration

### Roles

Three participant roles are supported:

1. **Recycler** - Users who collect and recycle materials
2. **Collector** - Users who collect materials from recyclers
3. **Manufacturer** - Users who manufacture products from recycled materials

### Data Structure

```rust
pub struct Participant {
    pub address: Address,
    pub role: Role,
    pub name: String,
    pub latitude: i64,
    pub longitude: i64,
    pub registered_at: u64,
}
```

## Functions

### Registration

```rust
pub fn register_participant(
    env: Env,
    participant_address: Address,
    role: Role,
    name: String,
    latitude: i64,
    longitude: i64,
)
```

Registers a new participant in the system.

**Parameters:**
- `participant_address` - Address of the participant (requires authentication)
- `role` - Role enum (Recycler, Collector, or Manufacturer)
- `name` - Display name of the participant
- `latitude` - Geographic latitude (can be negative)
- `longitude` - Geographic longitude (can be negative)

**Requirements:**
- Participant must authenticate (require_auth)
- Address can only register once
- All parameters are required

**Emits:** `ParticipantRegistered` event

### Query Functions

```rust
pub fn get_participant(env: Env, address: Address) -> Option<Participant>
```
Returns participant information if registered, None otherwise.

```rust
pub fn is_registered(env: Env, address: Address) -> bool
```
Checks if an address is registered.

## Events

### ParticipantRegistered

Emitted when a participant successfully registers.

**Event Data:**
- Topic: `("reg", participant_address)`
- Data: `(role, name, latitude, longitude)`

## Usage Example

```rust
use soroban_sdk::{Address, Env, String};

// Register as a recycler
let participant = Address::from_string("GPART...");
let name = String::from_str(&env, "John Recycler");

client.register_participant(
    &participant,
    &Role::Recycler,
    &name,
    &40_7128,  // New York latitude * 10000
    &-74_0060  // New York longitude * 10000
);

// Check if registered
if client.is_registered(&participant) {
    let info = client.get_participant(&participant).unwrap();
    // Use participant info
}
```

## Storage

Participants are stored in persistent storage using the pattern:
- Key: `(PARTICIPANT, address)`
- Value: `Participant` struct

This allows efficient lookup by address and ensures data persists across contract upgrades.

## Testing

The contract includes comprehensive tests covering:

### Registration Tests
- ✅ Register as Recycler
- ✅ Register as Collector
- ✅ Register as Manufacturer
- ✅ All roles work correctly
- ✅ Multiple participants can register

### Validation Tests
- ✅ Cannot register twice (panic on duplicate)
- ✅ Unregistered addresses return None
- ✅ Registration requires authentication

### Data Tests
- ✅ Participant data stored correctly
- ✅ Timestamp recorded at registration
- ✅ Negative coordinates supported
- ✅ All fields persist correctly

### Event Tests
- ✅ Event emitted on registration
- ✅ Event contains correct data

Run tests with:
```bash
cargo test -p scavenger
```

## Acceptance Criteria

✅ **Users can register once** - Duplicate registration prevented with panic

✅ **All roles work correctly** - Recycler, Collector, and Manufacturer roles all function properly

✅ **Event emits properly** - ParticipantRegistered event emitted with correct data

## Security Features

1. **Authentication Required** - Only the participant address can register itself
2. **Single Registration** - Each address can only register once
3. **Immutable Registration** - Once registered, participant data cannot be changed (future enhancement)
4. **Persistent Storage** - Data survives contract upgrades

## Geographic Coordinates

Coordinates are stored as `i64` values to support:
- Positive and negative values (for all global locations)
- High precision (multiply by 10000 for 4 decimal places)
- Example: 40.7128° N = 407128

## Future Enhancements

Potential additions for future iterations:
- Update participant information
- Deregister/unregister functionality
- Role-based permissions
- Participant verification/approval system
- Search participants by role
- Geographic proximity queries
- Reputation/rating system
