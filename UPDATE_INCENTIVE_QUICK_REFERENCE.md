# Update Incentive - Quick Reference

## Function
```rust
update_incentive(env: &Env, incentive_id: u64, new_reward_points: u64, new_total_budget: u64) -> Incentive
```

## Parameters
- `incentive_id`: ID of the incentive to update
- `new_reward_points`: New reward points per kilogram (must be > 0)
- `new_total_budget`: New total budget allocation (must be > 0)

## Returns
Updated `Incentive` struct with new values

## Requirements
✅ Caller must be the incentive creator (manufacturer)
✅ Incentive must be active
✅ Reward points must be greater than zero
✅ Total budget must be greater than zero

## Errors
- `"Incentive not found"` - Invalid incentive ID
- `"Incentive is not active"` - Cannot update inactive incentive
- `"Reward must be greater than zero"` - Invalid reward value
- `"Total budget must be greater than zero"` - Invalid budget value
- Authentication error - Caller is not the incentive owner

## Behavior
1. Validates incentive exists and is active
2. Authenticates the caller as the incentive owner
3. Validates new reward points and budget values
4. Calculates used budget and adjusts remaining budget
5. Updates incentive in storage
6. Emits `INCENTIVE_UPDATED` event
7. Returns updated incentive

## Budget Logic
- **Used Budget** = `total_budget - remaining_budget`
- **New Remaining** = `new_total_budget - used_budget`
- If `new_total_budget <= used_budget`, incentive is deactivated

## Event Emitted
```rust
INCENTIVE_UPDATED (inc_upd)
- incentive_id: u64
- rewarder: Address
- new_reward_points: u64
- new_total_budget: u64
```

## Example Usage
```rust
// Update incentive #1 with new values
let updated = contract.update_incentive(
    &env,
    1,      // incentive_id
    150,    // new_reward_points
    8000,   // new_total_budget
);

println!("Updated reward: {}", updated.reward_points);
println!("Updated budget: {}", updated.total_budget);
println!("Remaining: {}", updated.remaining_budget);
```

## Common Use Cases

### Increase Rewards
```rust
// Increase reward from 100 to 200 points/kg
contract.update_incentive(&env, incentive_id, 200, total_budget);
```

### Add More Budget
```rust
// Increase budget from 5000 to 10000
contract.update_incentive(&env, incentive_id, reward_points, 10000);
```

### Adjust Both
```rust
// Update both reward and budget
contract.update_incentive(&env, incentive_id, 150, 7500);
```

## Testing
Run tests with:
```bash
cargo test update_incentive
```

## Related Functions
- `create_incentive` - Create new incentive
- `get_incentive_by_id` - Retrieve incentive details
- `get_active_incentive_for_manufacturer` - Get best active incentive
- `distribute_rewards` - Use incentive for rewards
