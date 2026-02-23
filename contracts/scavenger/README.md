# Scavenger Contract - Constructor Implementation

This contract implements a secure initialization pattern with default configuration values and re-initialization protection.

## Constructor Specification

### Function Signature
```rust
pub fn __constructor(env: &Env, admin: Address, token_address: Address)
```

### Parameters
- `admin` - Address that will have administrative privileges
- `token_address` - Address of the scavenger token contract

### Initialization Behavior

1. **Accepts token address parameter** - Required parameter for token contract integration
2. **Initializes counter to 1** - Sets the counter storage to 1 for tracking purposes
3. **Sets default percentages**:
   - Collector percentage: 5%
   - Owner percentage: 50%
4. **Sets admin address** - Stores the admin with authentication
5. **Validation** - Prevents re-initialization with panic

### Default Configuration

| Parameter | Default Value | Description |
|-----------|--------------|-------------|
| Collector Percentage | 5% | Percentage allocated to collectors |
| Owner Percentage | 50% | Percentage allocated to owners |
| Counter | 1 | Initial counter value |
| Total Earned | 0 | Initial tokens earned |

## Storage Initialization

The constructor initializes the following storage keys:

- `ADMIN` - Admin address
- `TOKEN` - Token contract address
- `COL_PCT` - Collector percentage (5%)
- `OWN_PCT` - Owner percentage (50%)
- `COUNTER` - Counter value (1)
- `EARNED` - Total tokens earned (0)
- `INIT` - Initialization flag (true)

## Security Features

### Re-initialization Protection
The contract uses an `INITIALIZED` flag to prevent re-initialization:

```rust
if Storage::is_initialized(env) {
    panic!("Contract already initialized");
}
```

This ensures the constructor can only be called once, protecting against:
- Accidental re-initialization
- Malicious attempts to reset contract state
- Admin takeover attacks

### Admin Authentication
The constructor requires authentication from the admin address:

```rust
admin.require_auth();
```

## Usage Example

```rust
use soroban_sdk::{Address, Env};

// Deploy and initialize contract
let admin = Address::from_string("GADMIN...");
let token = Address::from_string("GTOKEN...");

client.__constructor(&admin, &token);

// Verify initialization
assert_eq!(client.get_admin(), admin);
assert_eq!(client.get_token_address(), token);
assert_eq!(client.get_collector_percentage(), 5);
assert_eq!(client.get_owner_percentage(), 50);
assert_eq!(client.get_counter(), 1);
assert_eq!(client.is_initialized(), true);
```

## Post-Initialization Configuration

After initialization, the admin can:

1. **Set charity address** (optional):
```rust
client.set_charity_address(&admin, &charity_address);
```

2. **Update percentages** (with validation):
```rust
client.update_collector_percentage(&admin, 10);
client.update_owner_percentage(&admin, 40);
```

## Testing

The contract includes comprehensive tests covering:

### Initialization Tests
- ✅ Contract initializes with correct values
- ✅ All storage is set correctly
- ✅ Counter starts at 1
- ✅ Default percentages are correct (5% and 50%)

### Security Tests
- ✅ Cannot re-initialize contract
- ✅ Initialization requires admin authentication

### Validation Tests
- ✅ Percentage updates validate correctly
- ✅ Total percentages cannot exceed 100%

### Optional Configuration Tests
- ✅ Charity address is optional (None initially)
- ✅ Admin can set charity address after initialization

Run tests with:
```bash
cargo test -p scavenger
```

## Acceptance Criteria

✅ **Contract initializes once** - Re-initialization is prevented with panic

✅ **All storage is set correctly** - Admin, token address, percentages, counter, and total earned are all initialized

✅ **Cannot re-initialize** - Initialization flag prevents multiple constructor calls

## Implementation Details

### Storage Layer
The storage module provides type-safe access to all contract data with:
- Initialization flag checking
- Optional value handling
- Default value support

### Contract Layer
The contract implements:
- Single-call initialization pattern
- Admin authentication
- Default configuration values
- Post-initialization configuration methods

### Validation
- Admin authentication on initialization
- Re-initialization prevention
- Percentage validation on updates (sum ≤ 100%)

## Future Enhancements

Potential additions for future iterations:
- Upgradeable initialization for contract upgrades
- Multi-sig admin initialization
- Configurable default percentages
- Initialization events/logging
