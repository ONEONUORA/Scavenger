# Waste Deactivation - Quick Reference

## Function
```rust
deactivate_waste(env: &Env, admin: Address, waste_id: u64)
```

## Parameters
- `admin`: Contract admin address (authentication required)
- `waste_id`: ID of the waste material to deactivate

## Returns
None (void function)

## Requirements
✅ Caller must be the contract admin
✅ Waste must exist
✅ Waste must not be already deactivated

## Errors
- `"Only admin can perform this action"` - Caller is not admin
- `"Waste not found"` - Invalid waste ID
- `"Waste already deactivated"` - Waste is already inactive

## Behavior
1. Validates admin authentication
2. Checks waste exists
3. Verifies waste is currently active
4. Sets `is_active` to false
5. Stores updated material
6. Emits `WASTE_DEACTIVATED` event

## Key Features
- **Permanent**: Cannot be reactivated once deactivated
- **Admin-Only**: Only contract admin can deactivate
- **Query Filtering**: Deactivated waste returns None from get_material
- **Event Tracking**: Emits event for audit trail

## Event Emitted
```rust
WASTE_DEACTIVATED (wst_deact)
- waste_id: u64
- admin: Address
```

## Related Functions

### get_material
```rust
pub fn get_material(env: &Env, material_id: u64) -> Option<Material>
```
- Returns `Some(Material)` only if material is active
- Returns `None` for deactivated materials
- Use this for querying materials

## Example Usage

### Deactivate a Waste Record
```rust
// Admin deactivates waste #123
contract.deactivate_waste(&env, &admin, 123);

// Waste is no longer queryable
let result = contract.get_material(&env, 123);
assert!(result.is_none());
```

### Check if Material is Active
```rust
// Query material
let material = contract.get_material(&env, material_id);

if material.is_some() {
    println!("Material is active");
} else {
    println!("Material not found or deactivated");
}
```

### Batch Deactivation
```rust
// Deactivate multiple waste records
let waste_ids = vec![101, 102, 103, 104];

for waste_id in waste_ids {
    contract.deactivate_waste(&env, &admin, waste_id);
}
```

## Common Use Cases

### Remove Fraudulent Entry
```rust
// Admin discovers fraudulent waste submission
contract.deactivate_waste(&env, &admin, fraudulent_waste_id);
```

### Correct Data Entry Error
```rust
// Remove duplicate or incorrect entry
contract.deactivate_waste(&env, &admin, duplicate_waste_id);
```

### Policy Violation
```rust
// Remove waste that violates system policies
contract.deactivate_waste(&env, &admin, violating_waste_id);
```

## Material Struct Changes

### New Field
```rust
pub struct Material {
    pub id: u64,
    pub waste_type: WasteType,
    pub weight: u64,
    pub submitter: Address,
    pub current_owner: Address,
    pub submitted_at: u64,
    pub verified: bool,
    pub is_active: bool,  // NEW: defaults to true
}
```

### Default Behavior
- New materials: `is_active = true`
- After deactivation: `is_active = false`
- No reactivation possible

## Testing
Run tests with:
```bash
cargo test deactivate_waste
```

## Important Notes

⚠️ **Breaking Change**: Material struct has been updated
- Requires contract redeployment
- Existing data may need migration

⚠️ **Permanent Action**: Deactivation cannot be undone
- No reactivation function exists
- Use with caution

⚠️ **Admin Only**: Only contract admin can deactivate
- Regular users cannot deactivate waste
- Authentication is strictly enforced

## Query Behavior

### Active Material
```rust
let material = contract.get_material(&env, active_id);
// Returns: Some(Material { is_active: true, ... })
```

### Deactivated Material
```rust
let material = contract.get_material(&env, deactivated_id);
// Returns: None
```

### Non-Existent Material
```rust
let material = contract.get_material(&env, 999999);
// Returns: None
```

## Best Practices

1. **Verify Before Deactivating**
   - Double-check waste ID
   - Confirm deactivation is necessary
   - Document reason for deactivation

2. **Monitor Events**
   - Track WASTE_DEACTIVATED events
   - Maintain audit log
   - Review deactivation patterns

3. **Use Sparingly**
   - Deactivation is permanent
   - Consider alternatives first
   - Reserve for exceptional cases

4. **Batch Operations**
   - Group related deactivations
   - Process in single transaction if possible
   - Minimize gas costs

## Troubleshooting

### "Only admin can perform this action"
- Verify caller is contract admin
- Check admin address is correct
- Ensure proper authentication

### "Waste not found"
- Verify waste ID exists
- Check if waste was already deactivated
- Confirm waste was properly submitted

### "Waste already deactivated"
- Waste has already been deactivated
- Cannot deactivate twice
- Check deactivation history

## Migration Guide

For existing contracts:
1. Update Material struct definition
2. Redeploy contract
3. Set `is_active: true` for existing materials
4. Update client code to handle new field
5. Test thoroughly before production use
