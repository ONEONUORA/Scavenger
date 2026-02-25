# Waste Confirmation Reset - Quick Reference

## Functions

### confirm_waste
```rust
confirm_waste(env: &Env, waste_id: u64, confirmer: Address)
```

### reset_waste_confirmation
```rust
reset_waste_confirmation(env: &Env, waste_id: u64, owner: Address)
```

## Parameters

### confirm_waste
- `waste_id`: ID of the waste to confirm
- `confirmer`: Address of the confirmer (authentication required)

### reset_waste_confirmation
- `waste_id`: ID of the waste to reset
- `owner`: Address of the waste owner (authentication required)

## Requirements

### confirm_waste
✅ Confirmer must be registered participant
✅ Waste must exist
✅ Waste must be active

### reset_waste_confirmation
✅ Caller must be the current waste owner
✅ Waste must exist
✅ Waste must be active

## Errors

### confirm_waste
- `"Waste not found"` - Invalid waste ID
- `"Waste is not active"` - Waste has been deactivated
- `"Confirmer not registered"` - Confirmer is not registered

### reset_waste_confirmation
- `"Waste not found"` - Invalid waste ID
- `"Only waste owner can reset confirmation"` - Caller is not owner
- `"Waste is not active"` - Waste has been deactivated

## Behavior

### confirm_waste
1. Validates waste exists and is active
2. Checks confirmer is registered
3. Sets `is_confirmed` to true
4. Records confirmer address
5. Stores updated material
6. Emits `WASTE_CONFIRMED` event

### reset_waste_confirmation
1. Validates waste exists and is active
2. Checks caller is current owner
3. Sets `is_confirmed` to false
4. Resets confirmer to submitter
5. Stores updated material
6. Emits `WASTE_CONFIRMATION_RESET` event

## Events Emitted

### WASTE_CONFIRMED (wst_conf)
```rust
- waste_id: u64
- confirmer: Address
```

### WASTE_CONFIRMATION_RESET (wst_rst)
```rust
- waste_id: u64
- owner: Address
```

## Material Struct Changes

### New Fields
```rust
pub struct Material {
    // ... existing fields ...
    pub is_confirmed: bool,  // NEW: defaults to false
    pub confirmer: Address,  // NEW: defaults to submitter
}
```

## Example Usage

### Basic Confirmation
```rust
// Confirm waste
contract.confirm_waste(&env, waste_id, &confirmer);

// Check confirmation
let material = contract.get_material(&env, waste_id).unwrap();
assert_eq!(material.is_confirmed, true);
assert_eq!(material.confirmer, confirmer);
```

### Reset Confirmation
```rust
// Reset confirmation
contract.reset_waste_confirmation(&env, waste_id, &owner);

// Verify reset
let material = contract.get_material(&env, waste_id).unwrap();
assert_eq!(material.is_confirmed, false);
assert_eq!(material.confirmer, material.submitter);
```

### Re-confirmation After Reset
```rust
// Initial confirmation
contract.confirm_waste(&env, waste_id, &confirmer1);

// Reset
contract.reset_waste_confirmation(&env, waste_id, &owner);

// Re-confirm with different confirmer
contract.confirm_waste(&env, waste_id, &confirmer2);

let material = contract.get_material(&env, waste_id).unwrap();
assert_eq!(material.is_confirmed, true);
assert_eq!(material.confirmer, confirmer2);
```

## Common Use Cases

### Confirm Waste Quality
```rust
// Inspector confirms waste meets standards
contract.confirm_waste(&env, waste_id, &inspector);
```

### Reset for Re-inspection
```rust
// Owner requests re-inspection
contract.reset_waste_confirmation(&env, waste_id, &owner);

// New inspector confirms
contract.confirm_waste(&env, waste_id, &new_inspector);
```

### Transfer with Fresh Confirmation
```rust
// Transfer waste
contract.transfer_waste(&env, waste_id, &old_owner, &new_owner);

// New owner resets for their own confirmation
contract.reset_waste_confirmation(&env, waste_id, &new_owner);

// New owner's inspector confirms
contract.confirm_waste(&env, waste_id, &their_inspector);
```

## Workflow Patterns

### Simple Flow
```
Submit → Confirm → Process
```

### Reset Flow
```
Submit → Confirm → Reset → Re-confirm → Process
```

### Multiple Cycles
```
Submit → Confirm → Reset → Confirm → Reset → Confirm → Process
```

## Testing
Run tests with:
```bash
cargo test reset_waste_confirmation
cargo test confirm_waste
```

## Important Notes

⚠️ **Breaking Change**: Material struct has been updated
- Requires contract redeployment
- Existing data may need migration

⚠️ **Owner Only**: Only current owner can reset
- Ownership is checked before reset
- Authentication is strictly enforced

✅ **Re-confirmation Allowed**: Waste can be confirmed multiple times
- Reset clears confirmation status
- Different participants can confirm after reset

## State Transitions

### Confirmation States
```
Unconfirmed (is_confirmed: false)
    ↓ confirm_waste
Confirmed (is_confirmed: true)
    ↓ reset_waste_confirmation
Unconfirmed (is_confirmed: false)
    ↓ confirm_waste
Confirmed (is_confirmed: true)
```

### Confirmer Address
```
Initial: confirmer = submitter
    ↓ confirm_waste
Updated: confirmer = actual_confirmer
    ↓ reset_waste_confirmation
Reset: confirmer = submitter
    ↓ confirm_waste
Updated: confirmer = new_confirmer
```

## Best Practices

1. **Confirm Before Critical Operations**
   - Use confirmation as quality gate
   - Verify waste before processing

2. **Track Confirmation Events**
   - Monitor WASTE_CONFIRMED events
   - Monitor WASTE_CONFIRMATION_RESET events
   - Maintain audit log

3. **Validate Confirmer Role**
   - Ensure confirmer has appropriate authority
   - Check confirmer credentials

4. **Document Reset Reasons**
   - Track why confirmation was reset
   - Maintain process documentation

## Troubleshooting

### Cannot Confirm Waste
- Check waste exists and is active
- Verify confirmer is registered
- Ensure proper authentication

### Cannot Reset Confirmation
- Verify caller is current owner
- Check waste is active
- Confirm ownership hasn't transferred

### Confirmation Not Persisting
- Verify storage is working
- Check for transaction failures
- Review event logs

## Migration Guide

For existing contracts:
1. Update Material struct definition
2. Redeploy contract
3. Set `is_confirmed: false` for existing materials
4. Set `confirmer: submitter` for existing materials
5. Update client code to handle new fields
6. Test confirmation workflow

## Related Functions

- `submit_material` - Create new waste material
- `transfer_waste` - Transfer waste ownership
- `deactivate_waste` - Deactivate waste (admin only)
- `get_material` - Query waste material

## Default Values

New materials are created with:
- `is_confirmed: false`
- `confirmer: submitter` (same as submitter address)

## Ownership Rules

- Only current owner can reset confirmation
- Ownership can change via `transfer_waste`
- New owner inherits confirmation status
- New owner can reset confirmation
