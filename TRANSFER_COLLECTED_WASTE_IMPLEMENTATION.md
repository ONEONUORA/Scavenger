# Transfer Collected Waste Function Implementation

## Summary
Implemented the `transfer_collected_waste` function in `stellar-contract/src/lib.rs` to allow collectors to transfer aggregated waste to manufacturers with weight to be confirmed later.

## Function Signature
```rust
pub fn transfer_collected_waste(
    env: Env,
    waste_type: WasteType,
    collector: Address,
    manufacturer: Address,
    latitude: i128,
    longitude: i128,
    notes: Symbol,
) -> u128
```

## Implementation Details

### ✅ Tasks Completed

1. **Check caller is collector** - Validates caller has Collector role
2. **Accept waste_type, recipient, location, notes** - All parameters accepted
3. **Create new waste record with weight 0** - Creates Waste with weight=0 for later confirmation
4. **Create transfer record** - Creates WasteTransfer and stores in history
5. **Emit event** - Publishes "bulk_xfr" event with all details

### Key Features

- **Role Validation**: Ensures caller is a Collector
- **Recipient Validation**: Ensures recipient is a Manufacturer
- **Zero Weight**: Creates waste with weight=0, allowing manufacturer to confirm actual weight later
- **Ownership**: Waste is immediately owned by manufacturer
- **Transfer History**: Records the transfer from collector to manufacturer
- **Event Emission**: Publishes event with symbol "bulk_xfr"

### Acceptance Criteria Met

✓ **Only collectors can use this** - Role check enforced  
✓ **Waste created correctly** - Waste struct created with weight=0  
✓ **Manufacturer can confirm weight** - Weight starts at 0, can be updated by manufacturer  

### Storage Operations
- **Waste creation**: `("waste_v2", waste_id)` - Stores new waste with weight=0
- **Manufacturer's waste list**: `("participant_wastes", manufacturer)` - Adds waste_id
- **Transfer history**: `("transfer_history", waste_id)` - Records transfer

### Event Structure
```rust
env.events().publish(
    (symbol_short!("bulk_xfr"), waste_id),
    (collector, manufacturer, waste_type, timestamp)
);
```

## Use Case
This function supports the scenario where collectors aggregate waste from multiple recyclers and transfer it in bulk to manufacturers. The weight is set to 0 initially because:
- The exact weight may not be known at collection time
- The manufacturer will weigh and confirm the actual amount upon receipt
- This allows for flexible bulk transfer workflows

## Test Coverage
Added `test_transfer_collected_waste()` that verifies:
- Collector can transfer to manufacturer
- Waste ID is generated correctly
- Function returns the new waste ID

## Files Modified
- `stellar-contract/src/lib.rs` - Added function and test
