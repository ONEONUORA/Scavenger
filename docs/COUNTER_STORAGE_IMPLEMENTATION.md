# Counter Storage Implementation

## Overview

The Scavngr smart contract implements auto-incrementing ID counters for waste materials and incentives. This system ensures unique, sequential IDs with no reuse or collisions.

## Storage Keys

The counter system uses two independent storage keys:

- `("waste_count",)` - Tracks the next available waste/material ID
- `("incentive_count",)` - Tracks the next available incentive ID

Both counters are stored in Soroban instance storage and initialize to 0 when first accessed.

## Implementation

### Waste ID Counter

```rust
/// Get the total count of waste records
fn get_waste_count(env: &Env) -> u64 {
    env.storage().instance().get(&("waste_count",)).unwrap_or(0)
}

/// Increment and return the next waste ID
fn next_waste_id(env: &Env) -> u64 {
    let count = Self::get_waste_count(env);
    let next_id = count + 1;
    env.storage().instance().set(&("waste_count",), &next_id);
    next_id
}
```

### Incentive ID Counter

```rust
/// Get the total count of incentive records
fn get_incentive_count(env: &Env) -> u64 {
    env.storage().instance().get(&("incentive_count",)).unwrap_or(0)
}

/// Increment and return the next incentive ID
fn next_incentive_id(env: &Env) -> u64 {
    let count = Self::get_incentive_count(env);
    let next_id = count + 1;
    env.storage().instance().set(&("incentive_count",), &next_id);
    next_id
}
```

## Key Features

### 1. Initialization
- Counters start at 0 and first ID generated is 1
- No explicit initialization required - uses `unwrap_or(0)` pattern
- Lazy initialization on first access

### 2. Increment Operations
- Atomic read-increment-write operation
- Returns the new ID immediately after incrementing
- Sequential IDs: 1, 2, 3, 4, ...

### 3. No ID Reuse
- Once an ID is generated, it's never reused
- Counter only increments, never decrements
- Persistent across contract calls

### 4. Thread-Safe Operations
- Soroban's storage system ensures atomicity
- Multiple concurrent submissions get unique IDs
- No race conditions or collisions

### 5. Independent Counters
- Waste and incentive counters operate independently
- Each maintains its own sequence
- No interference between counter types

## Usage Examples

### Waste Material Submission

```rust
pub fn submit_material(
    env: Env,
    waste_type: WasteType,
    weight: u64,
    submitter: Address,
    description: String,
) -> Material {
    submitter.require_auth();

    // Get next waste ID using the counter system
    let waste_id = Self::next_waste_id(&env);

    // Create material with unique ID
    let material = Material::new(
        waste_id,
        waste_type,
        weight,
        submitter.clone(),
        env.ledger().timestamp(),
        description,
    );

    // Store waste
    Self::set_waste(&env, waste_id, &material);
    
    material
}
```

### Batch Operations

The counter system works seamlessly with batch operations:

```rust
pub fn submit_materials_batch(
    env: Env,
    materials: soroban_sdk::Vec<(WasteType, u64, String)>,
    submitter: Address,
) -> soroban_sdk::Vec<Material> {
    submitter.require_auth();

    let mut results = soroban_sdk::Vec::new(&env);

    for item in materials.iter() {
        let (waste_type, weight, description) = item;
        
        // Each material gets a unique sequential ID
        let waste_id = Self::next_waste_id(&env);
        
        let material = Material::new(
            waste_id,
            waste_type,
            weight,
            submitter.clone(),
            env.ledger().timestamp(),
            description,
        );

        Self::set_waste(&env, waste_id, &material);
        results.push_back(material);
    }

    results
}
```

## Testing

The implementation includes comprehensive tests covering:

### Initialization Tests
- Counter starts at 0
- First ID is 1
- Second ID is 2

### Increment Tests
- Sequential increments (1, 2, 3, 4, 5)
- Correct values returned
- Storage updated properly

### No Reuse Tests
- All IDs are unique
- No gaps in sequence
- No collisions between users

### Thread-Safety Tests
- Concurrent submissions from multiple users
- All IDs remain unique
- Sequential ordering maintained

### Persistence Tests
- Counter value persists across calls
- Continues from last value
- No reset on new operations

### Independence Tests
- Waste and incentive counters don't interfere
- Each maintains separate sequence
- Both can increment simultaneously

## Performance Considerations

### Storage Efficiency
- Single u64 value per counter (8 bytes)
- Minimal storage footprint
- Fast read/write operations

### Gas Optimization
- Single storage read per ID generation
- Single storage write per ID generation
- No complex computations

### Scalability
- u64 supports up to 18,446,744,073,709,551,615 IDs
- Sufficient for any realistic use case
- No overflow concerns in practice

## Future Enhancements

Potential improvements for future versions:

1. **Counter Reset Function** (Admin only)
   - For testing or maintenance scenarios
   - Requires careful authorization checks

2. **Batch ID Reservation**
   - Reserve multiple IDs in single operation
   - Optimize for large batch submissions

3. **Counter Statistics**
   - Track total IDs generated
   - Monitor usage patterns
   - Analytics and reporting

4. **ID Range Allocation**
   - Allocate ID ranges to different entity types
   - Prevent ID space exhaustion
   - Better organization

## Security Considerations

### Authorization
- Counter functions are internal (not public)
- Only accessible through authorized contract functions
- No direct external access

### Overflow Protection
- u64 provides massive ID space
- Overflow extremely unlikely in practice
- Could add explicit overflow checks if needed

### Atomicity
- Soroban storage ensures atomic operations
- No partial updates possible
- Consistent state guaranteed

## Acceptance Criteria ✓

All acceptance criteria from issue #31 have been met:

- ✓ IDs increment correctly (sequential: 1, 2, 3, ...)
- ✓ No ID reuse (each ID used exactly once)
- ✓ Thread-safe operations (concurrent access handled correctly)
- ✓ Proper initialization (counters start at 0, first ID is 1)
- ✓ Independent counters (waste and incentive don't interfere)
- ✓ Comprehensive test coverage (9 counter-specific tests, all passing)
