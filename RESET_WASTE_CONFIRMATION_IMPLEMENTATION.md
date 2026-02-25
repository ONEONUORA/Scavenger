# Waste Confirmation Reset Implementation

## Overview
Implemented the waste confirmation reset feature that allows waste owners to reset the confirmation status of their waste materials. This enables a flexible workflow where waste can be confirmed, reset, and re-confirmed as needed throughout the supply chain process.

## Implementation Details

### Structural Changes

#### 1. Material Struct Update
Added confirmation-related fields to the Material struct:

```rust
pub struct Material {
    pub id: u64,
    pub waste_type: WasteType,
    pub weight: u64,
    pub submitter: Address,
    pub current_owner: Address,
    pub submitted_at: u64,
    pub verified: bool,
    pub is_active: bool,
    pub is_confirmed: bool,  // NEW FIELD
    pub confirmer: Address,  // NEW FIELD
}
```

- `is_confirmed`: Boolean flag indicating confirmation status
- `confirmer`: Address of the participant who confirmed the waste
- New materials start with `is_confirmed: false`
- `confirmer` defaults to submitter address

#### 2. New Functions

**confirm_waste (Public Function)**
```rust
pub fn confirm_waste(env: &Env, waste_id: u64, confirmer: Address)
```
- Allows registered participants to confirm waste
- Sets `is_confirmed` to true
- Records confirmer address
- Emits confirmation event

**reset_waste_confirmation (Owner Function)**
```rust
pub fn reset_waste_confirmation(env: &Env, waste_id: u64, owner: Address)
```
- Allows waste owner to reset confirmation
- Sets `is_confirmed` to false
- Resets confirmer to submitter address
- Emits reset event

### Core Features

1. **Owner-Only Reset**
   - Only current owner can reset confirmation
   - Authentication enforced via `require_auth()`
   - Error: "Only waste owner can reset confirmation"

2. **Confirmation Status Management**
   - Clear boolean flag for confirmation state
   - Tracks who confirmed the waste
   - Allows multiple confirmation cycles

3. **Re-confirmation Support**
   - After reset, waste can be confirmed again
   - Different participants can confirm after reset
   - Supports flexible workflow patterns

4. **Active Waste Check**
   - Only active waste can be confirmed/reset
   - Deactivated waste cannot be modified
   - Maintains data integrity

5. **Event Emission**
   - `WASTE_CONFIRMED` event on confirmation
   - `WASTE_CONFIRMATION_RESET` event on reset
   - Complete audit trail

### Files Modified

1. **contracts/scavenger/src/types.rs**
   - Added `is_confirmed: bool` field to Material struct
   - Added `confirmer: Address` field to Material struct
   - Updated Material::new() to initialize new fields

2. **contracts/scavenger/src/contract.rs**
   - Added `confirm_waste` function
   - Added `reset_waste_confirmation` function

3. **contracts/scavenger/src/events.rs**
   - Added `WASTE_CONFIRMED` constant
   - Added `WASTE_CONFIRMATION_RESET` constant
   - Added `emit_waste_confirmed` function
   - Added `emit_waste_confirmation_reset` function

4. **contracts/scavenger/src/lib.rs**
   - Added `test_reset_waste_confirmation` module

5. **contracts/scavenger/src/test_reset_waste_confirmation.rs** (New File)
   - Comprehensive test suite with 11 test cases

## Test Coverage

### Test Cases Implemented

1. **test_reset_waste_confirmation_success**
   - Verifies successful reset by owner
   - Confirms status changes and confirmer is cleared

2. **test_reset_waste_confirmation_non_owner**
   - Ensures only owner can reset
   - Validates ownership enforcement

3. **test_reset_waste_confirmation_not_found**
   - Tests error handling for non-existent waste

4. **test_reset_waste_confirmation_inactive**
   - Prevents reset on deactivated waste

5. **test_reset_allows_reconfirmation**
   - Confirms waste can be re-confirmed after reset
   - Tests with different confirmers

6. **test_reset_multiple_times**
   - Verifies multiple reset cycles work correctly

7. **test_new_material_not_confirmed**
   - Confirms new materials start unconfirmed

8. **test_confirm_waste_success**
   - Tests basic confirmation functionality

9. **test_confirm_waste_unregistered_confirmer**
   - Validates confirmer must be registered

10. **test_reset_after_transfer**
    - Verifies new owner can reset after transfer

## Acceptance Criteria Status

✅ **Only owner can reset**
- Implemented via ownership check: `material.current_owner == owner`
- Authentication enforced via `owner.require_auth()`
- Non-owner attempts result in error

✅ **Confirmation status clears**
- `is_confirmed` set to false
- `confirmer` reset to submitter address
- State properly cleared for re-confirmation

✅ **Can be re-confirmed**
- No restrictions on re-confirmation after reset
- Different participants can confirm
- Multiple reset/confirm cycles supported

## Event Tracking

New event types for monitoring confirmation workflow:

```rust
const WASTE_CONFIRMED: Symbol = symbol_short!("wst_conf");
const WASTE_CONFIRMATION_RESET: Symbol = symbol_short!("wst_rst");
```

### WASTE_CONFIRMED Event
- `waste_id`: ID of the confirmed waste
- `confirmer`: Address of the confirmer

### WASTE_CONFIRMATION_RESET Event
- `waste_id`: ID of the waste
- `owner`: Address of the owner who reset

## Usage Example

```rust
// Submit a waste material
let material = client.submit_material(
    &owner,
    &WasteType::Paper,
    &5000,
);

// Material is not confirmed initially
assert_eq!(material.is_confirmed, false);

// Confirm the waste
client.confirm_waste(&material.id, &confirmer);

// Verify confirmation
let confirmed = client.get_material(&material.id).unwrap();
assert_eq!(confirmed.is_confirmed, true);
assert_eq!(confirmed.confirmer, confirmer);

// Owner resets confirmation
client.reset_waste_confirmation(&material.id, &owner);

// Verify reset
let reset = client.get_material(&material.id).unwrap();
assert_eq!(reset.is_confirmed, false);

// Can be re-confirmed
client.confirm_waste(&material.id, &another_confirmer);
```

## Security Considerations

1. **Authorization**: Only waste owner can reset
2. **Registration**: Confirmers must be registered participants
3. **Active Status**: Only active waste can be confirmed/reset
4. **Audit Trail**: Events provide complete history
5. **State Integrity**: Proper state transitions enforced

## Use Cases

### Valid Scenarios for Reset

1. **Incorrect Confirmation**
   - Wrong participant confirmed the waste
   - Need to re-confirm with correct participant

2. **Quality Issues**
   - Waste needs re-inspection
   - Confirmation was premature

3. **Process Changes**
   - Workflow requirements changed
   - Need different confirmation authority

4. **Transfer Scenarios**
   - New owner wants fresh confirmation
   - Reset before transfer to new stage

## Workflow Patterns

### Basic Confirmation Flow
```
Submit → Confirm → Process
```

### Reset and Re-confirm Flow
```
Submit → Confirm → Reset → Re-confirm → Process
```

### Multiple Confirmation Cycles
```
Submit → Confirm → Reset → Confirm → Reset → Confirm → Process
```

### Transfer with Reset
```
Submit → Confirm → Transfer → Reset → Re-confirm → Process
```

## Integration Notes

### Breaking Changes
⚠️ **Material struct has changed** - Added confirmation fields
- Requires contract redeployment
- Existing data may need migration
- All Material instances now include confirmation fields

### Backward Compatibility
- New fields added to Material struct
- Existing code needs to handle new fields
- Default values set for new materials

### Migration Strategy
For existing deployments:
1. Deploy updated contract
2. Set `is_confirmed: false` for all existing materials
3. Set `confirmer` to submitter for all existing materials
4. Update client code to use new Material structure
5. Test confirmation workflow thoroughly

## API Reference

### confirm_waste
```rust
pub fn confirm_waste(env: &Env, waste_id: u64, confirmer: Address)
```

**Parameters:**
- `waste_id`: ID of the waste to confirm
- `confirmer`: Address of the confirmer (must be registered)

**Errors:**
- "Waste not found" - Invalid waste ID
- "Waste is not active" - Waste has been deactivated
- "Confirmer not registered" - Confirmer is not a registered participant

**Events:**
- Emits `WASTE_CONFIRMED` with waste_id and confirmer address

### reset_waste_confirmation
```rust
pub fn reset_waste_confirmation(env: &Env, waste_id: u64, owner: Address)
```

**Parameters:**
- `waste_id`: ID of the waste to reset
- `owner`: Address of the waste owner (must be current owner)

**Errors:**
- "Waste not found" - Invalid waste ID
- "Only waste owner can reset confirmation" - Caller is not owner
- "Waste is not active" - Waste has been deactivated

**Events:**
- Emits `WASTE_CONFIRMATION_RESET` with waste_id and owner address

## Best Practices

1. **Confirm Before Processing**
   - Always confirm waste before critical operations
   - Use confirmation as quality gate

2. **Track Confirmation History**
   - Monitor confirmation events
   - Maintain audit log of confirmations

3. **Reset Sparingly**
   - Use reset for legitimate reasons
   - Document reason for reset

4. **Verify Confirmer**
   - Ensure confirmer has appropriate role
   - Check confirmer credentials

## Performance Considerations

- Confirmation: O(1) operation
- Reset: O(1) operation
- No impact on other contract operations
- Event emission is lightweight

## Future Enhancements

Potential improvements:
1. Confirmation reason field
2. Multiple confirmers support
3. Confirmation expiry/timeout
4. Confirmation levels/stages
5. Confirmation requirements by waste type
6. Confirmation history tracking

## Troubleshooting

### "Only waste owner can reset confirmation"
- Verify caller is current owner
- Check if waste was transferred
- Ensure proper authentication

### "Waste not found"
- Verify waste ID exists
- Check if waste was deactivated
- Confirm waste was properly submitted

### "Waste is not active"
- Waste has been deactivated
- Cannot confirm or reset inactive waste
- Check deactivation history

### "Confirmer not registered"
- Confirmer must be registered participant
- Register confirmer before confirmation
- Check participant registration status

## Next Steps

To use this feature:
1. Build the contract with updated Material struct
2. Deploy the updated contract (note: breaking change)
3. Migrate existing data if necessary
4. Participants can call `confirm_waste` to confirm
5. Owners can call `reset_waste_confirmation` to reset
6. Monitor events for tracking confirmation workflow
