# Issue #52: Implement get_incentives Function

## Summary
Implemented `get_incentives` function to query all active incentives for a specific waste type, filtered to show only active incentives and sorted by reward amount in descending order.

## Changes Made

### 1. Implemented get_incentives Function (stellar-contract/src/lib.rs)
```rust
pub fn get_incentives(env: Env, waste_type: WasteType) -> Vec<Incentive>
```
- Accepts waste_type parameter
- Returns vector of Incentive records
- Filters to include only active incentives
- Sorts by reward_points in descending order (highest rewards first)

### Implementation Details
```rust
pub fn get_incentives(env: Env, waste_type: WasteType) -> Vec<Incentive> {
    // Get all incentive IDs for this waste type
    let incentive_ids = Self::get_incentives_by_waste_type(env.clone(), waste_type);
    
    let mut active_incentives = Vec::new(&env);
    
    // Collect all active incentives
    for incentive_id in incentive_ids.iter() {
        if let Some(incentive) = Self::get_incentive_internal(&env, incentive_id) {
            // Filter: only include active incentives
            if incentive.active {
                active_incentives.push_back(incentive);
            }
        }
    }
    
    // Sort by reward_points in descending order (highest rewards first)
    // Using bubble sort since Soroban Vec doesn't have built-in sort
    let len = active_incentives.len();
    for i in 0..len {
        for j in 0..(len - i - 1) {
            let curr = active_incentives.get(j).unwrap();
            let next = active_incentives.get(j + 1).unwrap();
            
            if curr.reward_points < next.reward_points {
                // Swap elements
                let temp = curr.clone();
                active_incentives.set(j, next);
                active_incentives.set(j + 1, temp);
            }
        }
    }
    
    active_incentives
}
```

### Incentive Structure
```rust
pub struct Incentive {
    pub id: u64,
    pub rewarder: Address,
    pub waste_type: WasteType,
    pub reward_points: u64,
    pub total_budget: u64,
    pub remaining_budget: u64,
    pub active: bool,
    pub created_at: u64,
}
```

### 2. Added Comprehensive Tests (stellar-contract/tests/get_incentives_test.rs)

Created 16 test cases covering all scenarios:

#### Basic Functionality Tests
- ✅ `test_get_incentives_returns_active_only` - Filters out deactivated incentives
- ✅ `test_get_incentives_filters_by_waste_type` - Returns only matching waste type
- ✅ `test_get_incentives_sorted_by_reward_descending` - Sorted highest to lowest

#### Edge Cases Tests
- ✅ `test_get_incentives_empty_for_no_incentives` - Empty vector when no incentives
- ✅ `test_get_incentives_empty_when_all_deactivated` - Empty when all deactivated
- ✅ `test_get_incentives_single_incentive` - Handles single incentive

#### Sorting Tests
- ✅ `test_get_incentives_sorting_with_equal_rewards` - Handles equal reward amounts
- ✅ `test_get_incentives_already_sorted` - Maintains correct order

#### Multiple Waste Types Tests
- ✅ `test_get_incentives_independent_per_waste_type` - Independent per waste type

#### Data Integrity Tests
- ✅ `test_get_incentives_returns_complete_data` - All fields correct
- ✅ `test_get_incentives_reflects_budget_changes` - Reflects budget updates
- ✅ `test_get_incentives_excludes_auto_deactivated` - Excludes auto-deactivated

#### Comprehensive Coverage Tests
- ✅ `test_get_incentives_all_waste_types` - Works with all waste types
- ✅ `test_get_incentives_multiple_manufacturers` - Includes all manufacturers
- ✅ `test_get_incentives_no_side_effects` - Read-only operation
- ✅ `test_get_incentives_large_list` - Handles many incentives

## Acceptance Criteria Met

### ✅ Accept waste_type parameter
- Function accepts `waste_type: WasteType` parameter
- Works with all waste types (Paper, PetPlastic, Plastic, Metal, Glass)
- No authentication required (read-only)

### ✅ Return Vec<Incentive>
- Returns `Vec<Incentive>` containing incentive records
- Empty vector for waste types with no active incentives
- Type-safe return value

### ✅ Filter only active incentives
- Only includes incentives where `active == true`
- Excludes manually deactivated incentives
- Excludes auto-deactivated incentives (budget exhausted)
- Filters applied before sorting

### ✅ Sort by reward amount
- Sorted in descending order by `reward_points`
- Highest rewards appear first
- Stable sort for equal values
- Efficient bubble sort implementation

## Technical Details

### Function Signature
```rust
pub fn get_incentives(env: Env, waste_type: WasteType) -> Vec<Incentive>
```

### Algorithm
1. **Retrieve IDs**: Get all incentive IDs for the waste type
2. **Filter**: Collect only active incentives
3. **Sort**: Bubble sort by reward_points (descending)
4. **Return**: Sorted vector of active incentives

### Filtering Logic
```rust
if incentive.active {
    active_incentives.push_back(incentive);
}
```
- Checks `active` field
- Excludes deactivated incentives
- Includes only currently available incentives

### Sorting Logic
```rust
// Bubble sort - descending order
for i in 0..len {
    for j in 0..(len - i - 1) {
        if curr.reward_points < next.reward_points {
            // Swap to put higher reward first
            swap(j, j+1);
        }
    }
}
```
- Bubble sort algorithm
- Descending order (highest first)
- O(n²) time complexity
- Suitable for typical incentive counts

### Time Complexity
- **Retrieval**: O(1) - Single storage read for IDs
- **Filtering**: O(n) - Iterate through all incentives
- **Sorting**: O(n²) - Bubble sort
- **Overall**: O(n²) where n is number of incentives

### Space Complexity
- **O(n)** - Stores filtered incentives
- Additional O(1) for sorting (in-place swaps)

## Usage Examples

### Basic Usage
```rust
// Get all active incentives for Plastic
let incentives = client.get_incentives(&WasteType::Plastic);

println!("Found {} active incentives", incentives.len());
for incentive in incentives.iter() {
    println!("Reward: {} points/kg", incentive.reward_points);
}
```

### Display Best Incentives
```rust
let incentives = client.get_incentives(&WasteType::Metal);

if incentives.is_empty() {
    println!("No active incentives for Metal");
} else {
    let best = incentives.get(0).unwrap();
    println!("Best incentive: {} points/kg", best.reward_points);
    println!("Budget remaining: {}", best.remaining_budget);
}
```

### Compare Incentives
```rust
let plastic = client.get_incentives(&WasteType::Plastic);
let metal = client.get_incentives(&WasteType::Metal);

println!("Plastic incentives: {}", plastic.len());
println!("Metal incentives: {}", metal.len());

if !plastic.is_empty() && !metal.is_empty() {
    let best_plastic = plastic.get(0).unwrap().reward_points;
    let best_metal = metal.get(0).unwrap().reward_points;
    
    if best_plastic > best_metal {
        println!("Plastic offers better rewards");
    } else {
        println!("Metal offers better rewards");
    }
}
```

### Find Incentive by Manufacturer
```rust
let incentives = client.get_incentives(&WasteType::Paper);

for incentive in incentives.iter() {
    if incentive.rewarder == manufacturer_address {
        println!("Found your incentive: {} points/kg", incentive.reward_points);
    }
}
```

### Calculate Potential Rewards
```rust
let incentives = client.get_incentives(&WasteType::Glass);
let weight_kg = 10; // 10 kg of glass

println!("Potential rewards for {}kg of Glass:", weight_kg);
for incentive in incentives.iter() {
    let reward = weight_kg * incentive.reward_points;
    println!("  {} points from {}", reward, incentive.rewarder);
}
```

### Check Incentive Availability
```rust
let incentives = client.get_incentives(&WasteType::PetPlastic);

let available_count = incentives.len();
let total_budget: u64 = incentives.iter()
    .map(|i| i.remaining_budget)
    .sum();

println!("{} active incentives", available_count);
println!("Total budget available: {} points", total_budget);
```

### Display Incentive Leaderboard
```rust
let incentives = client.get_incentives(&WasteType::Metal);

println!("=== Metal Incentive Leaderboard ===");
for (i, incentive) in incentives.iter().enumerate() {
    println!("{}. {} points/kg (Budget: {})", 
        i + 1, 
        incentive.reward_points,
        incentive.remaining_budget
    );
}
```

## Integration with Existing System

### Workflow Integration
```rust
// 1. Manufacturer creates incentive
let incentive = client.create_incentive(
    &manufacturer,
    &WasteType::Plastic,
    &50,
    &10000
);

// 2. Collector checks available incentives
let incentives = client.get_incentives(&WasteType::Plastic);
assert_eq!(incentives.len(), 1);
assert_eq!(incentives.get(0).unwrap().reward_points, 50);

// 3. Collector submits material
let material = client.submit_material(
    &WasteType::Plastic,
    &5000,
    &collector,
    &desc
);

// 4. Recycler verifies material
client.verify_material(&material.id, &recycler);

// 5. Collector claims reward
client.claim_incentive_reward(
    &incentive.id,
    &material.id,
    &collector
);

// 6. Check updated incentive
let updated = client.get_incentives(&WasteType::Plastic);
let incentive = updated.get(0).unwrap();
assert_eq!(incentive.remaining_budget, 9750); // 10000 - 250
```

### Compatible Functions
- `create_incentive()` - Create new incentives
- `deactivate_incentive()` - Deactivate incentives
- `get_incentive_by_id()` - Get specific incentive
- `get_incentives_by_rewarder()` - Get by manufacturer
- `get_incentives_by_waste_type()` - Get IDs only
- `claim_incentive_reward()` - Claim rewards

## Performance Considerations

### Current Implementation
- **Filtering**: O(n) - Linear scan through incentives
- **Sorting**: O(n²) - Bubble sort algorithm
- **Suitable for**: Typical incentive counts (< 100 per waste type)

### Performance Characteristics
- Reads incentive IDs from storage
- Loads each incentive individually
- Filters in memory
- Sorts in memory
- Returns complete vector

### Optimization Opportunities
For large numbers of incentives:
1. **Maintain sorted index** - Store incentives pre-sorted
2. **Use merge sort** - O(n log n) sorting
3. **Pagination** - Return subsets of results
4. **Caching** - Cache sorted results

## Filtering Behavior

### Active Status
- **Included**: `active == true`
- **Excluded**: `active == false`
- **Excluded**: Auto-deactivated (budget exhausted)

### Waste Type Matching
- Exact match on `waste_type` field
- No partial matching
- Case-sensitive enum comparison

### Deactivation Scenarios
1. **Manual**: Manufacturer calls `deactivate_incentive()`
2. **Automatic**: Budget exhausted during reward claim
3. **Both excluded** from results

## Sorting Behavior

### Sort Order
- **Descending**: Highest rewards first
- **Field**: `reward_points`
- **Stable**: Equal values maintain relative order

### Sort Examples
```
Input:  [30, 70, 50, 90, 20]
Output: [90, 70, 50, 30, 20]

Input:  [50, 50, 70, 30]
Output: [70, 50, 50, 30]
```

### Why Descending?
- Users want to see best rewards first
- Maximizes collector earnings
- Encourages participation
- Standard for incentive displays

## Edge Cases Handled

### No Incentives
- Returns empty vector
- No errors or panics
- Safe to iterate

### All Deactivated
- Returns empty vector
- Filters out all inactive
- Consistent behavior

### Single Incentive
- Returns vector with one element
- No sorting needed
- Correct format

### Equal Rewards
- Maintains stable order
- All equal values included
- Predictable results

## Security Considerations

### No Authentication Required
- Read-only operation
- Public data access
- No authorization needed
- Safe for any caller

### Privacy Implications
- Reveals active incentives
- Shows reward amounts
- Public information by design
- Encourages transparency

### Data Integrity
- Returns immutable snapshot
- No modification possible
- Consistent results
- Thread-safe reads

## Future Enhancements

### Potential Additions
1. **Pagination**
   ```rust
   pub fn get_incentives_page(
       env: Env,
       waste_type: WasteType,
       offset: u64,
       limit: u64
   ) -> Vec<Incentive>;
   ```

2. **Filtering Options**
   ```rust
   pub fn get_incentives_filtered(
       env: Env,
       waste_type: WasteType,
       min_reward: u64,
       min_budget: u64
   ) -> Vec<Incentive>;
   ```

3. **Multiple Waste Types**
   ```rust
   pub fn get_incentives_multi(
       env: Env,
       waste_types: Vec<WasteType>
   ) -> Vec<Incentive>;
   ```

4. **Sort Options**
   ```rust
   pub fn get_incentives_sorted_by(
       env: Env,
       waste_type: WasteType,
       sort_by: SortField
   ) -> Vec<Incentive>;
   
   enum SortField {
       Reward,
       Budget,
       CreatedAt,
   }
   ```

5. **Statistics**
   ```rust
   pub fn get_incentive_stats(
       env: Env,
       waste_type: WasteType
   ) -> IncentiveStats;
   
   struct IncentiveStats {
       total_count: u64,
       active_count: u64,
       total_budget: u64,
       avg_reward: u64,
   }
   ```

## Testing Strategy

### Test Coverage
- **Basic functionality**: 3 tests
- **Edge cases**: 3 tests
- **Sorting**: 2 tests
- **Multiple waste types**: 1 test
- **Data integrity**: 3 tests
- **Comprehensive coverage**: 4 tests
- **Total**: 16 tests

### Test Scenarios
1. Returns only active incentives
2. Filters by waste type
3. Sorted by reward descending
4. Empty for no incentives
5. Empty when all deactivated
6. Single incentive
7. Equal reward amounts
8. Already sorted input
9. Independent per waste type
10. Complete data returned
11. Reflects budget changes
12. Excludes auto-deactivated
13. All waste types
14. Multiple manufacturers
15. No side effects
16. Large lists

### Edge Cases Tested
- No incentives
- All deactivated
- Single incentive
- Equal rewards
- Budget exhaustion
- Auto-deactivation
- Large lists (10+ items)

## Documentation

### Code Comments
- Function documentation with examples
- Parameter descriptions
- Return value documentation
- Sorting explanation
- Filtering logic

### Test Documentation
- Test names describe scenarios
- Comments explain complex logic
- Expected behavior clear
- Edge cases documented

## Deployment Notes

### No Breaking Changes
- Adds new public function
- Maintains existing functions
- Backward compatible
- No migration needed

### Usage Recommendations
- Use `get_incentives` for user-facing displays
- Use `get_incentives_by_waste_type` for IDs only
- Use `get_incentive_by_id` for specific incentive
- Choose based on data needs

## Comparison with Existing Functions

### Function Comparison
| Function | Returns | Filtering | Sorting | Use Case |
|----------|---------|-----------|---------|----------|
| `get_incentives` | Vec<Incentive> | Active only | By reward | User display |
| `get_incentives_by_waste_type` | Vec<u64> | None | None | Get IDs |
| `get_incentives_by_rewarder` | Vec<u64> | By manufacturer | None | Manufacturer view |
| `get_incentive_by_id` | Option<Incentive> | N/A | N/A | Single lookup |

## Conclusion

This implementation provides a complete, production-ready incentive query function with:
- **Active filtering**: Shows only available incentives
- **Waste type filtering**: Precise matching
- **Reward sorting**: Best incentives first
- **Comprehensive testing**: 16 test cases covering all scenarios
- **Clear documentation**: Usage examples and patterns
- **Performance**: Efficient for typical use cases

All acceptance criteria have been met:
✅ Accepts waste_type parameter
✅ Returns Vec<Incentive>
✅ Filters only active incentives
✅ Sorted by reward amount (descending)

The implementation is secure, efficient, and ready for production deployment.
