# Participant Implementation - Quick Reference

## Data Structure

```rust
pub struct Participant {
    pub address: Address,              // Participant's blockchain address
    pub role: ParticipantRole,         // Recycler, Collector, or Manufacturer
    pub name: Symbol,                  // Display name
    pub latitude: i128,                // Geographic latitude (scaled by 1e6)
    pub longitude: i128,               // Geographic longitude (scaled by 1e6)
    pub is_registered: bool,           // Active registration status
    pub total_waste_processed: u128,   // Total grams of waste processed
    pub total_tokens_earned: u128,     // Total reward tokens earned
    pub registered_at: u64,            // Unix timestamp of registration
}
```

## Roles & Permissions

| Role | Collect | Process | Verify | Manufacture |
|------|---------|---------|--------|-------------|
| Recycler | ✅ | ✅ | ✅ | ❌ |
| Collector | ✅ | ❌ | ❌ | ❌ |
| Manufacturer | ❌ | ❌ | ❌ | ✅ |

## Public Functions

### Registration
```rust
// Register new participant
register_participant(env, address, role, name, latitude, longitude) -> Participant

// Deactivate participant
deregister_participant(env, address) -> Participant

// Update participant role
update_role(env, address, new_role) -> Participant

// Update participant location
update_location(env, address, latitude, longitude) -> Participant
```

### Queries
```rust
// Get participant data
get_participant(env, address) -> Option<Participant>

// Check collection permission
can_collect(env, address) -> bool

// Check manufacturing permission
can_manufacture(env, address) -> bool
```

## Usage Examples

### Register a Recycler
```rust
let name = Symbol::new(&env, "Alice");
let participant = client.register_participant(
    &user_address,
    &ParticipantRole::Recycler,
    &name,
    &40_748_817,  // NYC latitude * 1e6
    &-73_985_428, // NYC longitude * 1e6
);
```

### Submit Material (Auto-updates stats)
```rust
// Requires registered participant
let material = client.submit_material(
    &WasteType::Metal,
    &5000,  // 5kg in grams
    &submitter_address,
    &description,
);
// Automatically increments total_waste_processed by 5000
```

### Verify Material (Auto-updates tokens)
```rust
// Requires registered recycler
let verified = client.verify_material(&material_id, &recycler_address);
// Automatically increments submitter's total_tokens_earned
```

### Check Permissions
```rust
if client.can_collect(&address) {
    // User can collect materials
}

if client.can_manufacture(&address) {
    // User can manufacture products
}
```

## Coordinate Format

Coordinates are stored as `i128` scaled by 1,000,000:

```rust
// Convert decimal degrees to storage format
let latitude = (40.748817 * 1_000_000.0) as i128;   // 40_748_817
let longitude = (-73.985428 * 1_000_000.0) as i128; // -73_985_428

// Convert storage format to decimal degrees
let lat_degrees = latitude as f64 / 1_000_000.0;    // 40.748817
let lon_degrees = longitude as f64 / 1_000_000.0;   // -73.985428
```

## Statistics Tracking

### Automatic Updates

| Action | Updates |
|--------|---------|
| `submit_material()` | `total_waste_processed += weight` |
| `submit_materials_batch()` | `total_waste_processed += sum(weights)` |
| `verify_material()` | `total_tokens_earned += reward_points` |
| `verify_materials_batch()` | `total_tokens_earned += sum(rewards)` |

### Reward Calculation

```rust
// Points = (weight_in_kg) * multiplier * 10
// Multipliers:
// - Paper: 1
// - PetPlastic: 3
// - Plastic: 2
// - Metal: 5
// - Glass: 2

// Example: 5kg of metal
// Points = 5 * 5 * 10 = 250
```

## Error Handling

### Common Errors

```rust
// Participant not found
panic!("Participant not found")

// Not registered
panic!("Participant is not registered")

// Wrong role
panic!("Only recyclers can verify materials")

// Overflow
panic!("Overflow in total_waste_processed")
panic!("Overflow in total_tokens_earned")
```

## Security Features

### Overflow Protection
```rust
// All arithmetic uses checked_add()
total_waste_processed = total_waste_processed
    .checked_add(weight)
    .expect("Overflow in total_waste_processed");
```

### Registration Validation
```rust
// All restricted actions validate registration
fn require_registered(env: &Env, address: &Address) {
    // Panics if not found or not registered
}
```

### Authentication
```rust
// All write operations require authentication
address.require_auth();
```

## Testing

### Run All Tests
```bash
cd stellar-contract
cargo test --lib
```

### Run Specific Test
```bash
cargo test test_participant_persistence
```

### Verify Implementation
```bash
./scripts/verify-participant-implementation.sh
```

## Migration Checklist

- [ ] Export existing participant data
- [ ] Deploy new contract version
- [ ] Update client code to include new parameters
- [ ] Re-register all participants
- [ ] Verify data integrity
- [ ] Update documentation
- [ ] Regenerate test snapshots

## Common Patterns

### Register and Submit
```rust
// 1. Register participant
let name = Symbol::new(&env, "Bob");
client.register_participant(&user, &ParticipantRole::Collector, &name, &0, &0);

// 2. Submit material
let desc = String::from_str(&env, "Plastic bottles");
let material = client.submit_material(&WasteType::Plastic, &2000, &user, &desc);

// 3. Check stats
let participant = client.get_participant(&user).unwrap();
assert_eq!(participant.total_waste_processed, 2000);
```

### Verify and Earn Tokens
```rust
// 1. Register recycler
let name = Symbol::new(&env, "Recycler");
client.register_participant(&recycler, &ParticipantRole::Recycler, &name, &0, &0);

// 2. Verify material
client.verify_material(&material_id, &recycler);

// 3. Check submitter's tokens
let submitter = client.get_participant(&submitter_address).unwrap();
// tokens earned based on material weight and type
```

### Batch Operations
```rust
// Batch submit
let mut materials = Vec::new(&env);
materials.push_back((WasteType::Paper, 1000, desc1));
materials.push_back((WasteType::Plastic, 2000, desc2));
let results = client.submit_materials_batch(&materials, &user);

// Batch verify
let mut ids = Vec::new(&env);
ids.push_back(1);
ids.push_back(2);
let verified = client.verify_materials_batch(&ids, &recycler);
```

## Performance Tips

1. **Use Batch Operations** - More efficient than individual calls
2. **Minimize Storage Reads** - Cache participant data when possible
3. **Validate Once** - Registration checked at function entry
4. **Efficient Serialization** - Compact data types reduce gas costs

## Documentation Links

- **Full Implementation Guide:** `docs/PARTICIPANT_IMPLEMENTATION.md`
- **Changes Summary:** `docs/PARTICIPANT_CHANGES_SUMMARY.md`
- **Completion Report:** `PARTICIPANT_IMPLEMENTATION_COMPLETE.md`

## Support

For issues or questions:
1. Check the full documentation
2. Review test cases for examples
3. Run verification script
4. Check error messages for debugging hints
