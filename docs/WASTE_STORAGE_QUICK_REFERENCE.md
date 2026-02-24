# Waste Storage - Quick Reference

## Data Structures

### Waste Struct
```rust
pub struct Waste {
    pub id: u64,
    pub waste_type: WasteType,
    pub weight: u64,
    pub submitter: Address,
    pub submitted_at: u64,
    pub status: WasteStatus,
    pub location: String,
}
```

### WasteStatus Enum
```rust
pub enum WasteStatus {
    Pending = 0,
    Processing = 1,
    Processed = 2,
    Rejected = 3,
}
```

## Quick Start

### Create Waste
```rust
let waste = Waste::new(
    1,                              // id
    WasteType::Plastic,            // waste_type
    5000,                          // weight (grams)
    submitter,                     // submitter address
    env.ledger().timestamp(),      // submitted_at
    String::from_str(&env, "Downtown"), // location
);
```

### Store Waste
```rust
env.storage().instance().set(&("waste", waste.id), &waste);
```

### Retrieve Waste
```rust
let waste: Waste = env.storage().instance().get(&("waste", id)).unwrap();
```

### Update Status
```rust
let mut waste: Waste = env.storage().instance().get(&("waste", id)).unwrap();
if waste.update_status(WasteStatus::Processed) {
    env.storage().instance().set(&("waste", id), &waste);
}
```

## Methods

### Waste Methods
```rust
// Create new waste
Waste::new(id, waste_type, weight, submitter, submitted_at, location) -> Waste

// Update status (returns false if status is final)
waste.update_status(new_status) -> bool

// Check minimum weight (100g)
waste.meets_minimum_weight() -> bool

// Check if processable
waste.is_processable() -> bool

// Validate all fields
waste.validate() -> Result<(), &'static str>
```

### WasteStatus Methods
```rust
// Validate u32 value
WasteStatus::is_valid(value) -> bool

// Convert from u32
WasteStatus::from_u32(value) -> Option<WasteStatus>

// Convert to u32
status.to_u32() -> u32

// Get string representation
status.as_str() -> &'static str

// Check if modifiable
status.is_modifiable() -> bool

// Check if final
status.is_final() -> bool
```

## Status Transitions

### Valid Transitions
```
Pending → Processing → Processed
Pending → Rejected
Processing → Processed
Processing → Rejected
```

### Invalid Transitions
```
Processed → * (final)
Rejected → * (final)
```

## Validation Rules

### Weight
- Must be > 0
- Must be >= 100g (minimum)
- Maximum: u64::MAX

### Status
- Pending: Initial status
- Processing: Can be modified
- Processed: Final, cannot be modified
- Rejected: Final, cannot be modified

## Storage Patterns

### Single Item
```rust
// Store
env.storage().instance().set(&("waste", id), &waste);

// Retrieve
let waste: Waste = env.storage().instance().get(&("waste", id)).unwrap();

// Check existence
let exists: bool = env.storage().instance().has(&("waste", id));
```

### Batch Operations
```rust
// Store multiple
for i in 1..=10 {
    let waste = Waste::new(i, waste_type, weight, submitter.clone(), timestamp, location.clone());
    env.storage().instance().set(&("waste", i), &waste);
}

// Retrieve multiple
let mut wastes = Vec::new();
for i in 1..=10 {
    if let Some(waste) = env.storage().instance().get::<_, Waste>(&("waste", i)) {
        wastes.push(waste);
    }
}
```

### Different Key Types
```rust
// By ID
env.storage().instance().set(&("waste", id), &waste);

// By submitter and ID
env.storage().instance().set(&("waste_by_submitter", submitter, id), &waste);

// By status and ID
env.storage().instance().set(&("waste_by_status", status, id), &waste);
```

## Common Patterns

### Create and Validate
```rust
let waste = Waste::new(id, waste_type, weight, submitter, timestamp, location);

match waste.validate() {
    Ok(_) => {
        env.storage().instance().set(&("waste", id), &waste);
    },
    Err(e) => {
        panic!("Validation failed: {}", e);
    }
}
```

### Process Workflow
```rust
// 1. Retrieve waste
let mut waste: Waste = env.storage().instance().get(&("waste", id)).unwrap();

// 2. Check if processable
if !waste.is_processable() {
    panic!("Waste is not processable");
}

// 3. Update to Processing
waste.update_status(WasteStatus::Processing);
env.storage().instance().set(&("waste", id), &waste);

// 4. Process...

// 5. Update to Processed
waste.update_status(WasteStatus::Processed);
env.storage().instance().set(&("waste", id), &waste);
```

### Safe Status Update
```rust
let mut waste: Waste = env.storage().instance().get(&("waste", id)).unwrap();

if waste.update_status(new_status) {
    env.storage().instance().set(&("waste", id), &waste);
    // Status updated successfully
} else {
    // Status is final, cannot update
    panic!("Cannot update final status");
}
```

### Query by Status
```rust
// Get all pending waste
let mut pending_wastes = Vec::new();
for i in 1..=total_count {
    if let Some(waste) = env.storage().instance().get::<_, Waste>(&("waste", i)) {
        if waste.status == WasteStatus::Pending {
            pending_wastes.push(waste);
        }
    }
}
```

## Error Handling

### Validation Errors
```rust
"Weight must be greater than zero"
"Weight must be at least 100g"
```

### Status Update Errors
```rust
// Returns false if status is final
if !waste.update_status(new_status) {
    // Handle error
}
```

### Storage Errors
```rust
// Handle missing waste
match env.storage().instance().get::<_, Waste>(&("waste", id)) {
    Some(waste) => {
        // Process waste
    },
    None => {
        panic!("Waste not found");
    }
}
```

## Testing

### Run Tests
```bash
cd stellar-contract
cargo test --lib waste_tests
cargo test --lib waste_status_tests
```

### Test Coverage
- ✅ Creation and storage
- ✅ Round-trip serialization
- ✅ Status updates
- ✅ Validation
- ✅ Boundary values
- ✅ Edge cases

## Storage Footprint

### Size Estimate
```
id:           8 bytes
waste_type:   4 bytes
weight:       8 bytes
submitter:   32 bytes
submitted_at: 8 bytes
status:       4 bytes
location:     Variable (string length + overhead)
---
Total:       ~64 bytes + location length
```

### Optimization Tips
1. Use short location strings
2. Batch storage operations
3. Use efficient key structures
4. Cache frequently accessed items

## Status Reference

| Status | Value | Modifiable | Final | Description |
|--------|-------|------------|-------|-------------|
| Pending | 0 | ✅ | ❌ | Submitted, awaiting processing |
| Processing | 1 | ✅ | ❌ | Currently being processed |
| Processed | 2 | ❌ | ✅ | Successfully processed |
| Rejected | 3 | ❌ | ✅ | Rejected (invalid/contaminated) |

## WasteType Reference

| Type | Value | Description |
|------|-------|-------------|
| Paper | 0 | Paper waste |
| PetPlastic | 1 | PET plastic |
| Plastic | 2 | General plastic |
| Metal | 3 | Metal waste |
| Glass | 4 | Glass waste |

## Best Practices

1. **Always Validate**: Call `validate()` before storing
2. **Check Status**: Verify status before updates
3. **Handle Errors**: Use proper error handling for storage operations
4. **Use Consistent Keys**: Maintain consistent key patterns
5. **Batch When Possible**: Batch operations for efficiency
6. **Document Keys**: Document your key structure
7. **Test Thoroughly**: Test all edge cases

## Documentation Links

- **Full Implementation Guide:** `docs/WASTE_STORAGE_IMPLEMENTATION.md`
- **Changes Summary:** `docs/WASTE_STORAGE_CHANGES_SUMMARY.md`
- **Completion Report:** `WASTE_STORAGE_IMPLEMENTATION_COMPLETE.md`
