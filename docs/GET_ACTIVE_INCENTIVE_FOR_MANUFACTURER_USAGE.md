# Get Active Incentive for Manufacturer - Usage Guide

## Function Overview

`get_active_incentive_for_manufacturer` is a query function that retrieves the best active incentive for a specific manufacturer and waste type combination.

## Function Signature

```rust
pub fn get_active_incentive_for_manufacturer(
    env: Env,
    manufacturer: Address,
    waste_type: WasteType,
) -> Option<Incentive>
```

## Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `env` | `Env` | Soroban environment context |
| `manufacturer` | `Address` | The manufacturer's blockchain address |
| `waste_type` | `WasteType` | The type of waste (Paper, PetPlastic, Plastic, Metal, Glass) |

## Return Value

- **`Some(Incentive)`**: Returns the active incentive with the highest `reward_points` if found
- **`None`**: Returns None if no active incentive matches the criteria

## Usage Examples

### Example 1: Basic Usage

```rust
use stellar_scavngr_contract::{ScavengerContractClient, WasteType};

// Get the best active incentive for a manufacturer
let manufacturer_addr = Address::from_string("GABC123...");
let result = client.get_active_incentive_for_manufacturer(
    &manufacturer_addr,
    &WasteType::Plastic
);

match result {
    Some(incentive) => {
        println!("Found incentive with {} points per kg", incentive.reward_points);
        println!("Remaining budget: {}", incentive.remaining_budget);
    },
    None => {
        println!("No active incentive available");
    }
}
```

### Example 2: Reward Calculation

```rust
// Calculate potential reward before claiming
let manufacturer = Address::from_string("GABC123...");
let waste_type = WasteType::Metal;
let weight_grams = 5000; // 5 kg

if let Some(incentive) = client.get_active_incentive_for_manufacturer(
    &manufacturer,
    &waste_type
) {
    let weight_kg = weight_grams / 1000;
    let potential_reward = incentive.reward_points * weight_kg;
    
    if potential_reward <= incentive.remaining_budget {
        println!("Can claim {} points", potential_reward);
        // Proceed with claim
    } else {
        println!("Insufficient budget in incentive");
    }
} else {
    println!("No active incentive for this waste type");
}
```

### Example 3: Comparing Multiple Manufacturers

```rust
// Find the best manufacturer to work with for a specific waste type
let manufacturers = vec![manufacturer1, manufacturer2, manufacturer3];
let waste_type = WasteType::Paper;

let mut best_offer: Option<(Address, u64)> = None;

for manufacturer in manufacturers {
    if let Some(incentive) = client.get_active_incentive_for_manufacturer(
        &manufacturer,
        &waste_type
    ) {
        match best_offer {
            None => best_offer = Some((manufacturer.clone(), incentive.reward_points)),
            Some((_, current_best)) => {
                if incentive.reward_points > current_best {
                    best_offer = Some((manufacturer.clone(), incentive.reward_points));
                }
            }
        }
    }
}

if let Some((best_manufacturer, reward)) = best_offer {
    println!("Best offer: {} points from {:?}", reward, best_manufacturer);
}
```

### Example 4: Checking Availability Before Submission

```rust
// Check if there's an active incentive before submitting material
let manufacturer = Address::from_string("GABC123...");
let waste_type = WasteType::Glass;

match client.get_active_incentive_for_manufacturer(&manufacturer, &waste_type) {
    Some(incentive) if incentive.remaining_budget > 0 => {
        // Safe to submit material
        let material = client.submit_material(
            &waste_type,
            &weight,
            &collector,
            &description
        );
        println!("Material submitted with incentive ID: {}", incentive.id);
    },
    Some(_) => {
        println!("Incentive exists but budget exhausted");
    },
    None => {
        println!("No active incentive available");
    }
}
```

## Filtering Logic

The function applies three filters:

1. **Manufacturer Filter**: Only considers incentives created by the specified manufacturer
2. **Waste Type Filter**: Only considers incentives for the specified waste type
3. **Active Status Filter**: Only considers incentives where `active == true`

## Selection Criteria

When multiple incentives match all filters:
- Returns the incentive with the **highest `reward_points`** value
- If multiple incentives have equal reward_points, returns one of them (deterministic but unspecified which)

## Common Use Cases

### 1. Pre-Claim Validation
Check if an incentive exists before attempting to claim rewards:
```rust
if client.get_active_incentive_for_manufacturer(&mfr, &waste_type).is_some() {
    // Proceed with claim
}
```

### 2. UI Display
Show available incentives to users:
```rust
for waste_type in all_waste_types {
    if let Some(incentive) = client.get_active_incentive_for_manufacturer(&mfr, &waste_type) {
        display_incentive_card(waste_type, incentive);
    }
}
```

### 3. Automatic Incentive Selection
Automatically select the best incentive for reward distribution:
```rust
let best_incentive = client.get_active_incentive_for_manufacturer(
    &manufacturer,
    &material.waste_type
)?;

client.distribute_rewards(
    &material.id,
    &best_incentive.id,
    &manufacturer
);
```

## Performance Notes

- **Time Complexity**: O(n) where n is the number of incentives for the manufacturer
- **Storage Reads**: One read per incentive ID
- **Gas Cost**: Proportional to the number of incentives (typically low)
- **Read-Only**: No state modifications, safe to call repeatedly

## Error Handling

The function returns `None` in these cases:
- Manufacturer has no incentives
- All incentives are inactive
- No incentives match the waste type
- Manufacturer address is invalid (no panic, just returns None)

## Integration Points

### Works With
- `create_incentive`: Creates queryable incentives
- `deactivate_incentive`: Deactivated incentives are excluded
- `claim_incentive_reward`: Auto-deactivated incentives are excluded
- `get_incentives_by_rewarder`: Uses this internally
- `distribute_rewards`: Can use this for automatic incentive selection

### Does Not Affect
- Storage state (read-only)
- Incentive budgets
- Active status
- Other participants' data

## Testing

Comprehensive test coverage available in:
```
stellar-contract/tests/get_active_incentive_for_manufacturer_test.rs
```

Run tests with:
```bash
cargo test --test get_active_incentive_for_manufacturer_test
```

## Best Practices

1. **Always check for None**: Handle the case where no incentive is available
2. **Verify budget**: Check `remaining_budget` before claiming
3. **Cache results**: If calling multiple times with same parameters, cache the result
4. **Error messages**: Provide clear feedback when no incentive is found
5. **Fallback logic**: Have a plan for when no incentive is available

## Related Functions

- `get_incentive_by_id`: Get a specific incentive by ID
- `get_incentives`: Get all active incentives for a waste type (all manufacturers)
- `get_incentives_by_rewarder`: Get all incentive IDs for a manufacturer
- `get_incentives_by_waste_type`: Get all incentive IDs for a waste type
- `create_incentive`: Create a new incentive
- `deactivate_incentive`: Deactivate an incentive
