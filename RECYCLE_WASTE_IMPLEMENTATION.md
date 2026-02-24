# Recycle Waste Function Implementation

## Summary
Implemented the `recycle_waste` function in `stellar-contract/src/lib.rs` to allow registered recyclers to register new waste with location data.

## Function Signature
```rust
pub fn recycle_waste(
    env: Env,
    waste_type: WasteType,
    weight: u128,
    recycler: Address,
    latitude: i128,
    longitude: i128,
) -> u128
```

## Implementation Details

### ✅ Tasks Completed

1. **Check caller is registered** - Validates that the recycler is a registered participant
2. **Accept parameters** - Accepts waste_type, weight, latitude, longitude
3. **Generate unique waste ID** - Uses `next_waste_id()` to generate sequential IDs
4. **Create Waste struct** - Creates a new Waste instance with all required fields
5. **Store in wastes map** - Stores waste in instance storage with key `("waste_v2", waste_id)`
6. **Add to participant_wastes** - Maintains a list of waste IDs per participant
7. **Emit WasteRecycled event** - Publishes event with symbol "recycled" and all details
8. **Return waste ID** - Returns the unique u128 waste ID

### Key Features

- **Authentication**: Requires caller authentication via `recycler.require_auth()`
- **Registration Check**: Panics if participant is not registered
- **Unique IDs**: Uses internal counter to ensure unique waste IDs
- **Event Emission**: Publishes event with waste details for off-chain tracking
- **Storage**: Uses separate storage key ("waste_v2") to avoid conflicts with existing Material storage

### Event Structure
```rust
env.events().publish(
    (symbol_short!("recycled"), waste_id),
    (waste_type, weight, recycler, latitude, longitude, timestamp)
);
```

## Test Coverage
Added `test_recycle_waste()` that verifies:
- Participant registration requirement
- Waste ID generation
- Function returns correct waste ID

## Files Modified
- `stellar-contract/src/lib.rs` - Added function and test
- Exported `Waste` type in public API
