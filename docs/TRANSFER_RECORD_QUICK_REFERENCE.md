# TransferRecord Quick Reference

## Quick Start

### Import
```rust
use crate::{TransferRecord, TransferItemType, TransferStatus};
```

### Create a Transfer Record
```rust
let record = TransferRecord::new(
    1,                              // id
    sender_address,                 // from
    recipient_address,              // to
    TransferItemType::Material,     // item_type
    42,                             // item_id
    1000,                           // amount
    env.ledger().timestamp(),       // timestamp
    String::from_str(&env, "Note"), // note
);
```

### Store Single Record
```rust
env.storage().instance().set(&("transfer", record_id), &record);
```

### Retrieve Single Record
```rust
let record: TransferRecord = env.storage().instance()
    .get(&("transfer", record_id))
    .unwrap();
```

### Store Transfer History
```rust
let mut history = Vec::new(&env);
history.push_back(record1);
history.push_back(record2);
env.storage().instance().set(&("history", address), &history);
```

### Append to History
```rust
let mut history: Vec<TransferRecord> = env.storage().instance()
    .get(&("history", address))
    .unwrap_or_else(|| Vec::new(&env));
history.push_back(new_record);
env.storage().instance().set(&("history", address), &history);
```

## TransferItemType

### Variants
- `Material` - Material/Waste transfer
- `Token` - Token transfer
- `Incentive` - Incentive transfer
- `Ownership` - Ownership transfer

### Methods
```rust
TransferItemType::is_valid(0)           // true
TransferItemType::from_u32(1)           // Some(Token)
TransferItemType::Material.to_u32()     // 0
TransferItemType::Token.as_str()        // "TOKEN"
```

## TransferStatus

### Variants
- `Pending` - Transfer is pending
- `InProgress` - Transfer is in progress
- `Completed` - Transfer completed successfully
- `Failed` - Transfer failed
- `Cancelled` - Transfer was cancelled

### Methods
```rust
TransferStatus::is_valid(2)             // true
TransferStatus::from_u32(2)             // Some(Completed)
TransferStatus::Completed.to_u32()      // 2
TransferStatus::Pending.as_str()        // "PENDING"
TransferStatus::Completed.is_final()    // true
TransferStatus::Pending.is_active()     // true
```

## TransferRecord Methods

### Update Status
```rust
if record.update_status(TransferStatus::Completed) {
    // Status updated
} else {
    // Status is final, cannot update
}
```

### Validate
```rust
match record.validate() {
    Ok(()) => // Valid
    Err(msg) => // Invalid: msg contains error
}
```

### Check Completion
```rust
if record.is_complete() {
    // Transfer is completed
}
```

### Check Modifiable
```rust
if record.is_modifiable() {
    // Can update status
}
```

## Common Patterns

### Create and Store
```rust
let record = TransferRecord::new(id, from, to, item_type, item_id, amount, timestamp, note);
record.validate()?;
env.storage().instance().set(&("transfer", id), &record);
```

### Update Status Workflow
```rust
let mut record: TransferRecord = env.storage().instance()
    .get(&("transfer", id))
    .expect("Transfer not found");

if record.update_status(TransferStatus::InProgress) {
    env.storage().instance().set(&("transfer", id), &record);
}
```

### Batch Retrieval
```rust
let mut results = Vec::new(&env);
for id in transfer_ids.iter() {
    if let Some(record) = env.storage().instance().get(&("transfer", id)) {
        results.push_back(record);
    }
}
```

### Filter by Status
```rust
let history: Vec<TransferRecord> = env.storage().instance()
    .get(&("history", address))
    .unwrap_or_else(|| Vec::new(&env));

let mut completed = Vec::new(&env);
for record in history.iter() {
    if record.is_complete() {
        completed.push_back(record);
    }
}
```

## Validation Rules

- Amount must be > 0
- Sender and recipient must be different
- Final statuses (Completed, Failed, Cancelled) cannot be changed

## Storage Keys

### Single Record
```rust
("transfer", record_id: u64)
```

### Transfer History
```rust
("history", address: Address)
("transfers", address: Address)
```

### By Item Type
```rust
("transfers", item_type: TransferItemType, item_id: u64)
```

## Error Messages

- `"Amount must be greater than zero"` - Invalid amount
- `"Sender and recipient cannot be the same"` - Same address validation

## Best Practices

1. Always validate before storing
2. Use vector storage for history
3. Check `is_modifiable()` before updates
4. Use batch operations for efficiency
5. Store with consistent key patterns
6. Handle `None` cases gracefully

## Testing

```rust
#[test]
fn test_transfer_workflow() {
    let env = Env::default();
    let from = Address::generate(&env);
    let to = Address::generate(&env);
    let note = String::from_str(&env, "Test");
    
    let mut record = TransferRecord::new(
        1, from, to, TransferItemType::Material,
        10, 1000, 0, note
    );
    
    assert!(record.validate().is_ok());
    assert!(record.update_status(TransferStatus::Completed));
    assert!(record.is_complete());
    assert!(!record.is_modifiable());
}
```
