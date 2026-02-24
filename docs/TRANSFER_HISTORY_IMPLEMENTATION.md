# Waste Transfer History Implementation

## Overview

The waste transfer history system provides an immutable, chronologically-ordered record of all waste ownership transfers in the Scavngr ecosystem. This creates a transparent audit trail for waste materials as they move through the recycling supply chain.

## Storage Architecture

### Storage Key Structure
```rust
Key: ("transfers", waste_id) → Vec<WasteTransfer>
```

Each waste item has its own transfer history stored as a vector, ensuring:
- Chronological ordering (append-only)
- Immutability (history cannot be modified)
- Efficient retrieval by waste ID

### WasteTransfer Structure
```rust
pub struct WasteTransfer {
    pub waste_id: u64,
    pub from: Address,
    pub to: Address,
    pub transferred_at: u64,
    pub note: String,
}
```

## Core Functions

### Transfer Recording

#### `record_transfer` (Internal)
```rust
fn record_transfer(
    env: &Env,
    waste_id: u64,
    from: Address,
    to: Address,
    note: String,
)
```

Internal function that appends a new transfer to the history.

**Features:**
- Automatically captures timestamp
- Appends to existing history (preserves chronological order)
- Creates new history vector if none exists

**Implementation:**
```rust
let mut history: Vec<WasteTransfer> = env
    .storage()
    .instance()
    .get(&key)
    .unwrap_or(Vec::new(env));

let transfer = WasteTransfer::new(
    waste_id,
    from,
    to,
    env.ledger().timestamp(),
    note,
);

history.push_back(transfer);
env.storage().instance().set(&key, &history);
```

### Waste Transfer

#### `transfer_waste`
```rust
pub fn transfer_waste(
    env: Env,
    waste_id: u64,
    from: Address,
    to: Address,
    note: String,
) -> Material
```

Transfers waste ownership and records the transfer in history.

**Validations:**
1. Sender authentication required
2. Both sender and receiver must be registered participants
3. Sender must be the current owner of the waste
4. Waste must exist

**Process:**
1. Validate participants
2. Retrieve and validate waste ownership
3. Update waste ownership
4. Record transfer in history
5. Return updated material

**Panics:**
- "Sender not registered"
- "Receiver not registered"
- "Only waste owner can transfer"
- "Waste not found"

**Example:**
```rust
let note = String::from_str(&env, "Transferring to processing facility");
let transferred = client.transfer_waste(
    &waste_id,
    &current_owner,
    &new_owner,
    &note
);
```

### History Retrieval

#### `get_transfer_history`
```rust
pub fn get_transfer_history(env: Env, waste_id: u64) -> Vec<WasteTransfer>
```

Retrieves the complete transfer history for a waste item.

**Returns:** 
- Vector of WasteTransfer records in chronological order
- Empty vector if no transfers have occurred

**Example:**
```rust
let history = client.get_transfer_history(&waste_id);
for transfer in history.iter() {
    // Process each transfer
}
```

### Query Functions

#### `get_transfers_from`
```rust
pub fn get_transfers_from(env: Env, address: Address) -> Vec<(u64, Vec<WasteTransfer>)>
```

Gets all transfers where the address was the sender.

**Note:** Current implementation returns empty vector. Production version would require additional indexing for efficient queries.

#### `get_transfers_to`
```rust
pub fn get_transfers_to(env: Env, address: Address) -> Vec<(u64, Vec<WasteTransfer>)>
```

Gets all transfers where the address was the receiver.

**Note:** Current implementation returns empty vector. Production version would require additional indexing for efficient queries.

## Immutability Guarantees

### Append-Only Design
The history is append-only, ensuring:
- Past transfers cannot be modified
- Transfer order is preserved
- Complete audit trail is maintained

### No Delete Operations
There are no functions to delete or modify transfer history, providing:
- Tamper-proof records
- Regulatory compliance
- Trust in the system

## Chronological Ordering

### Automatic Timestamp
Each transfer automatically captures the ledger timestamp:
```rust
env.ledger().timestamp()
```

### Sequential Appending
Transfers are appended to the vector in order:
```rust
history.push_back(transfer);
```

This ensures chronological order is maintained naturally.

## Usage Examples

### Single Transfer
```rust
// Register participants
client.register_participant(&recycler, &ParticipantRole::Recycler);
client.register_participant(&collector, &ParticipantRole::Collector);

// Submit waste
let waste = client.submit_material(
    &WasteType::Metal,
    &5000,
    &recycler,
    &String::from_str(&env, "Metal scraps")
);

// Transfer to collector
let note = String::from_str(&env, "Sending to collection center");
client.transfer_waste(&waste.id, &recycler, &collector, &note);

// View history
let history = client.get_transfer_history(&waste.id);
assert_eq!(history.len(), 1);
```

### Multi-Hop Transfer Chain
```rust
// Register supply chain participants
client.register_participant(&recycler, &ParticipantRole::Recycler);
client.register_participant(&collector, &ParticipantRole::Collector);
client.register_participant(&manufacturer, &ParticipantRole::Manufacturer);

// Submit waste
let waste = client.submit_material(&WasteType::Plastic, &10000, &recycler, &desc);

// Transfer 1: Recycler → Collector
client.transfer_waste(
    &waste.id,
    &recycler,
    &collector,
    &String::from_str(&env, "Initial collection")
);

// Transfer 2: Collector → Manufacturer
client.transfer_waste(
    &waste.id,
    &collector,
    &manufacturer,
    &String::from_str(&env, "Ready for processing")
);

// View complete chain
let history = client.get_transfer_history(&waste.id);
assert_eq!(history.len(), 2);

// Verify chronological order
let t1 = history.get(0).unwrap();
let t2 = history.get(1).unwrap();
assert!(t2.transferred_at >= t1.transferred_at);
```

### Audit Trail Verification
```rust
let history = client.get_transfer_history(&waste_id);

// Verify complete chain of custody
for (i, transfer) in history.iter().enumerate() {
    println!("Transfer {}: {} → {} at {}",
        i + 1,
        transfer.from,
        transfer.to,
        transfer.transferred_at
    );
    
    // Verify each participant in chain
    assert!(client.is_participant_registered(&transfer.from));
    assert!(client.is_participant_registered(&transfer.to));
}
```

## Performance Characteristics

- **Record Transfer:** O(n) where n = history length (read + append + write)
- **Get History:** O(1) - Direct key access
- **Transfer Waste:** O(n) - Includes history recording
- **Query by Participant:** O(m*n) - Requires iteration (not indexed)

## Gas Optimization

### Efficient Storage
- Single storage key per waste item
- Vector storage is gas-efficient for sequential access
- No redundant data stored

### Batch Considerations
For high-volume scenarios, consider:
- Limiting history length
- Archiving old transfers
- Off-chain indexing for queries

## Security Considerations

1. **Authentication:** Only waste owner can initiate transfers
2. **Validation:** Both parties must be registered participants
3. **Ownership Verification:** System validates current ownership before transfer
4. **Immutability:** History cannot be tampered with after recording
5. **Transparency:** All transfers are publicly auditable

## Testing

Comprehensive test coverage includes:

- `test_transfer_waste` - Basic transfer functionality
- `test_transfer_waste_unregistered_sender` - Sender validation
- `test_transfer_waste_unregistered_receiver` - Receiver validation
- `test_transfer_waste_not_owner` - Ownership verification
- `test_transfer_history_chronological` - Order preservation
- `test_transfer_history_immutable` - Immutability guarantee
- `test_empty_transfer_history` - Empty history handling
- `test_transfer_history_different_wastes` - History isolation

## Integration Points

### Material Tracking
Transfer history integrates with the material tracking system:
```rust
// Material ownership is updated
material.submitter = to.clone();
Self::set_waste(&env, waste_id, &material);

// Transfer is recorded
Self::record_transfer(&env, waste_id, from, to, note);
```

### Statistics System
Future integration could track:
- Number of transfers per participant
- Average time between transfers
- Most active transfer routes

### Verification System
Transfer history can be used to:
- Verify supply chain integrity
- Validate material provenance
- Audit recycling processes

## Future Enhancements

### Indexing
Add participant-based indexes for efficient queries:
```rust
// Index: ("transfers_from", address) → Vec<u64> (waste_ids)
// Index: ("transfers_to", address) → Vec<u64> (waste_ids)
```

### Batch Transfers
Support transferring multiple wastes in one transaction:
```rust
pub fn transfer_wastes_batch(
    env: Env,
    waste_ids: Vec<u64>,
    from: Address,
    to: Address,
    note: String,
) -> Vec<Material>
```

### Transfer Metadata
Enhance WasteTransfer with additional fields:
- Transfer reason/category
- Location information
- Quality assessment
- Processing notes

### History Archival
For long-lived wastes with many transfers:
- Archive old transfers to separate storage
- Maintain summary statistics
- Provide paginated history access

### Event Emission
Emit events for off-chain indexing:
```rust
env.events().publish(("transfer", waste_id), (from, to, timestamp));
```

## Compliance & Auditing

The transfer history system supports:

- **Regulatory Compliance:** Complete audit trail for waste tracking
- **Supply Chain Transparency:** Visible chain of custody
- **Quality Assurance:** Verification of proper handling
- **Dispute Resolution:** Historical record for conflict resolution
- **Environmental Reporting:** Data for sustainability metrics
