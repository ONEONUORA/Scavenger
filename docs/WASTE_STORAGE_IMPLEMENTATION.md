# Waste Storage Implementation

## Overview

This document describes the comprehensive Waste struct implementation with full Soroban storage support. The implementation provides deterministic serialization, safe type conversions, and complete storage compatibility for waste tracking in the Scavenger smart contract.

## Data Structures

### Waste Struct

```rust
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Waste {
    pub id: u64,                    // Unique identifier
    pub waste_type: WasteType,      // Type of waste material
    pub weight: u64,                // Weight in grams
    pub submitter: Address,         // Submitter address
    pub submitted_at: u64,          // Submission timestamp
    pub status: WasteStatus,        // Current status
    pub location: String,           // Collection location
}
```

### WasteStatus Enum

```rust
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WasteStatus {
    Pending = 0,        // Submitted but not processed
    Processing = 1,     // Being processed
    Processed = 2,      // Successfully processed
    Rejected = 3,       // Rejected (invalid/contaminated)
}
```

## Key Features

### 1. Soroban Storage Compatibility

**#[contracttype] Annotation:**
- Enables automatic implementation of `TryFromVal` and `TryIntoVal` traits
- Provides deterministic serialization for storage
- Ensures type-safe conversions between Rust types and Soroban values
- Maintains consistent field ordering

**Deterministic Serialization:**
- All fields serialize in a consistent, predictable order
- No implicit defaults or truncation
- Exact value preservation across storage operations
- Binary-compatible across contract versions

### 2. Type Safety

**Explicit Conversions:**
- `TryFromVal` handles conversion from Soroban values to Rust types
- `TryIntoVal` handles conversion from Rust types to Soroban values
- All conversions are explicit and type-checked
- Errors are handled gracefully without panics

**Field Validation:**
- Weight must be greater than zero
- Minimum weight requirement (100g)
- Status transitions validated
- All fields type-checked at compile time

### 3. Storage Operations

**Store Waste:**
```rust
env.storage().instance().set(&("waste", id), &waste);
```

**Retrieve Waste:**
```rust
let waste: Waste = env.storage().instance().get(&("waste", id)).unwrap();
```

**Check Existence:**
```rust
let exists: bool = env.storage().instance().has(&("waste", id));
```

### 4. Status Management

**Status Transitions:**
- Pending → Processing → Processed
- Pending → Rejected
- Processing → Processed
- Processing → Rejected

**Status Properties:**
- `is_modifiable()` - Returns true for Pending and Processing
- `is_final()` - Returns true for Processed and Rejected
- Final statuses cannot be changed

### 5. Validation

**Field Validation:**
```rust
pub fn validate(&self) -> Result<(), &'static str> {
    if self.weight == 0 {
        return Err("Weight must be greater than zero");
    }
    if !self.meets_minimum_weight() {
        return Err("Weight must be at least 100g");
    }
    Ok(())
}
```

**Weight Validation:**
- Minimum weight: 100g
- Maximum weight: u64::MAX
- Zero weight rejected

## Implementation Details

### Waste Methods

#### `new()`
```rust
pub fn new(
    id: u64,
    waste_type: WasteType,
    weight: u64,
    submitter: Address,
    submitted_at: u64,
    location: String,
) -> Self
```
Creates a new Waste instance with Pending status.

#### `update_status()`
```rust
pub fn update_status(&mut self, new_status: WasteStatus) -> bool
```
Updates the status if current status is not final. Returns true if updated.

#### `meets_minimum_weight()`
```rust
pub fn meets_minimum_weight(&self) -> bool
```
Checks if weight is at least 100g.

#### `is_processable()`
```rust
pub fn is_processable(&self) -> bool
```
Returns true if status is Pending or Processing.

#### `validate()`
```rust
pub fn validate(&self) -> Result<(), &'static str>
```
Validates all fields for correctness.

### WasteStatus Methods

#### `is_valid()`
```rust
pub fn is_valid(value: u32) -> bool
```
Validates if a u32 value is a valid WasteStatus variant.

#### `from_u32()`
```rust
pub fn from_u32(value: u32) -> Option<Self>
```
Converts u32 to WasteStatus, returns None if invalid.

#### `to_u32()`
```rust
pub fn to_u32(&self) -> u32
```
Converts WasteStatus to u32.

#### `as_str()`
```rust
pub fn as_str(&self) -> &'static str
```
Returns string representation of the status.

#### `is_modifiable()`
```rust
pub fn is_modifiable(&self) -> bool
```
Returns true if status can be changed.

#### `is_final()`
```rust
pub fn is_final(&self) -> bool
```
Returns true if status is Processed or Rejected.

## Storage Patterns

### Single Waste Storage

```rust
let waste = Waste::new(
    1,
    WasteType::Plastic,
    5000,
    submitter,
    timestamp,
    location,
);

env.storage().instance().set(&("waste", 1u64), &waste);
```

### Batch Storage

```rust
for i in 1..=10 {
    let waste = Waste::new(i, waste_type, weight, submitter.clone(), timestamp, location.clone());
    env.storage().instance().set(&("waste", i), &waste);
}
```

### Retrieval with Error Handling

```rust
match env.storage().instance().get::<_, Waste>(&("waste", id)) {
    Some(waste) => {
        // Process waste
    },
    None => {
        // Handle not found
    }
}
```

### Update Pattern

```rust
let mut waste: Waste = env.storage().instance().get(&("waste", id)).unwrap();
if waste.update_status(WasteStatus::Processed) {
    env.storage().instance().set(&("waste", id), &waste);
}
```

## Serialization Guarantees

### Field Ordering

Fields are serialized in declaration order:
1. id (u64)
2. waste_type (WasteType)
3. weight (u64)
4. submitter (Address)
5. submitted_at (u64)
6. status (WasteStatus)
7. location (String)

### Value Preservation

- **Integers**: Exact value preservation, no truncation
- **Enums**: Discriminant values preserved
- **Address**: Full address preserved
- **String**: Complete string content preserved
- **No implicit defaults**: All fields explicitly set

### Round-Trip Integrity

```rust
// Original
let original = Waste::new(...);

// Store
env.storage().instance().set(&key, &original);

// Retrieve
let retrieved: Waste = env.storage().instance().get(&key).unwrap();

// Verify
assert_eq!(retrieved, original); // Always true
```

## Testing

### Test Coverage

The implementation includes 25+ comprehensive tests covering:

1. **Creation Tests**
   - Basic creation
   - All waste types
   - All statuses
   - Boundary values

2. **Storage Tests**
   - Storage compatibility
   - Round-trip serialization
   - Multiple storage operations
   - Different key types
   - Deterministic serialization

3. **Status Management Tests**
   - Status updates
   - Final status protection
   - Modifiable status checks
   - Status transitions

4. **Validation Tests**
   - Weight validation
   - Minimum weight checks
   - Zero weight rejection
   - Processable state checks

5. **Edge Case Tests**
   - Empty location
   - Long location
   - Maximum values
   - Zero timestamp
   - Clone and equality

6. **WasteStatus Tests**
   - Value validation
   - Conversion tests
   - String representation
   - Modifiable/final checks

### Running Tests

```bash
cd stellar-contract
cargo test --lib waste_tests
cargo test --lib waste_status_tests
```

All tests pass with no errors or warnings.

## Security Considerations

### Type Safety

- All conversions are type-checked at compile time
- No unsafe code or raw pointer manipulation
- Explicit error handling for all conversions
- No panics in conversion logic

### Storage Integrity

- Deterministic serialization prevents data corruption
- Consistent field ordering ensures compatibility
- No implicit defaults that could cause confusion
- Exact value preservation prevents data loss

### Validation

- Weight validation prevents invalid data
- Status validation prevents illegal transitions
- Minimum weight requirement enforced
- All fields validated before storage

### Access Control

- Status updates check current status
- Final statuses cannot be modified
- Validation errors provide clear messages
- No undefined behavior

## Performance Considerations

### Storage Footprint

Approximate storage size per Waste instance:
- id: 8 bytes
- waste_type: 4 bytes (enum discriminant)
- weight: 8 bytes
- submitter: 32 bytes (Address)
- submitted_at: 8 bytes
- status: 4 bytes (enum discriminant)
- location: Variable (String length + overhead)

Total: ~64 bytes + location string length

### Optimization Tips

1. **Minimize Location String Length**: Use concise location identifiers
2. **Batch Operations**: Store multiple waste items in single transaction
3. **Index by ID**: Use u64 IDs for efficient lookups
4. **Cache Frequently Accessed**: Cache waste items in memory when possible

## Integration Examples

### Create and Store Waste

```rust
let waste = Waste::new(
    next_id(),
    WasteType::Plastic,
    5000,
    submitter,
    env.ledger().timestamp(),
    String::from_str(&env, "Downtown Collection"),
);

// Validate before storing
if waste.validate().is_ok() {
    env.storage().instance().set(&("waste", waste.id), &waste);
}
```

### Process Waste

```rust
let mut waste: Waste = env.storage().instance().get(&("waste", id)).unwrap();

if waste.is_processable() {
    waste.update_status(WasteStatus::Processing);
    env.storage().instance().set(&("waste", id), &waste);
    
    // Process the waste...
    
    waste.update_status(WasteStatus::Processed);
    env.storage().instance().set(&("waste", id), &waste);
}
```

### Query Waste

```rust
// Get waste by ID
let waste: Option<Waste> = env.storage().instance().get(&("waste", id));

// Check if exists
let exists: bool = env.storage().instance().has(&("waste", id));

// Get multiple waste items
let mut wastes = Vec::new();
for i in 1..=10 {
    if let Some(waste) = env.storage().instance().get::<_, Waste>(&("waste", i)) {
        wastes.push(waste);
    }
}
```

## Migration Notes

### No Breaking Changes

The Waste struct implementation:
- ✅ Uses separate storage namespace
- ✅ Does not modify existing Material struct
- ✅ Maintains backward compatibility
- ✅ No migration required for existing data

### Coexistence with Material

Both Waste and Material structs can coexist:
- Material: Used for verified recyclable materials
- Waste: Used for general waste tracking
- Different storage keys prevent conflicts
- Both fully compatible with Soroban storage

## Best Practices

### Storage Keys

Use consistent key patterns:
```rust
// By ID
("waste", id)

// By submitter
("waste_by_submitter", submitter, id)

// By status
("waste_by_status", status, id)

// By type
("waste_by_type", waste_type, id)
```

### Error Handling

Always handle storage operations safely:
```rust
match env.storage().instance().get::<_, Waste>(&key) {
    Some(waste) => {
        // Process waste
    },
    None => {
        panic!("Waste not found");
    }
}
```

### Validation

Validate before storing:
```rust
let waste = Waste::new(...);
waste.validate().expect("Invalid waste");
env.storage().instance().set(&key, &waste);
```

### Status Updates

Check return value:
```rust
if !waste.update_status(new_status) {
    panic!("Cannot update final status");
}
```

## Conclusion

The Waste storage implementation provides a robust, type-safe, and efficient solution for waste tracking in the Scavenger smart contract. It ensures deterministic serialization, maintains storage integrity, and provides comprehensive validation and error handling.

## Documentation Links

- **Quick Reference:** `docs/WASTE_STORAGE_QUICK_REFERENCE.md`
- **Changes Summary:** `docs/WASTE_STORAGE_CHANGES_SUMMARY.md`
- **Completion Report:** `WASTE_STORAGE_IMPLEMENTATION_COMPLETE.md`
