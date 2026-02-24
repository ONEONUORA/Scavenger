# Incentive Storage System Implementation

## Overview
The incentive storage system manages reward incentives created by manufacturers to encourage recycling of specific waste types. It provides three storage maps for efficient querying and management.

## Storage Architecture

### 1. Incentive Map (ID -> Incentive)
- **Key**: `("incentive", incentive_id: u64)`
- **Value**: `Incentive` struct
- **Purpose**: Direct lookup of incentive by ID
- **Usage**: Primary storage for all incentive data

### 2. Rewarder Incentives Map (Address -> Vec<u64>)
- **Key**: `("rewarder_incentives", manufacturer_address: Address)`
- **Value**: `Vec<u64>` (list of incentive IDs)
- **Purpose**: Track all incentives created by a specific manufacturer
- **Usage**: Query all incentives from a manufacturer

### 3. General Incentives Map (WasteType -> Vec<u64>)
- **Key**: `("general_incentives", waste_type: WasteType)`
- **Value**: `Vec<u64>` (list of incentive IDs)
- **Purpose**: Track all active incentives for a specific waste type
- **Usage**: Find available incentives for recyclers submitting specific waste types

## Data Structure

```rust
#[contracttype]
pub struct Incentive {
    pub id: u64,
    pub rewarder: Address,        // Manufacturer offering the incentive
    pub waste_type: WasteType,    // Target waste type
    pub reward_points: u64,       // Points per kg
    pub total_budget: u64,        // Total points allocated
    pub remaining_budget: u64,    // Points still available
    pub active: bool,             // Whether incentive is active
    pub created_at: u64,          // Timestamp
}
```

## Core Functions

### Storage Helpers
- `set_incentive(env, id, incentive)` - Store incentive by ID
- `get_incentive(env, id)` - Retrieve incentive by ID
- `incentive_exists(env, id)` - Check if incentive exists

### Query Functions
- `get_incentive_by_id(env, id)` - Public getter for incentive
- `get_incentives_by_rewarder(env, address)` - Get all incentives from a manufacturer
- `get_incentives_by_waste_type(env, waste_type)` - Get all incentives for a waste type

### Management Functions
- `create_incentive(env, rewarder, waste_type, reward_points, total_budget)` - Create new incentive
- `deactivate_incentive(env, id, rewarder)` - Deactivate an incentive (only by creator)
- `claim_incentive_reward(env, incentive_id, material_id, claimer)` - Claim reward for verified material

## Implementation Details

### Multiple Incentives Per Manufacturer
- Manufacturers can create unlimited incentives
- Each incentive gets a unique ID from the counter system
- All IDs are tracked in the rewarder's incentive list

### Deactivation
- Only the creator can deactivate their incentive
- Deactivation sets `active = false`
- Deactivated incentives remain in storage for history
- Deactivated incentives are excluded from active queries

### Budget Management
- `total_budget` is set at creation and never changes
- `remaining_budget` decreases as rewards are claimed
- Incentive automatically becomes inactive when budget is exhausted
- Claims fail if insufficient budget remains

## Usage Examples

### Creating an Incentive
```rust
let incentive = client.create_incentive(
    &manufacturer,
    &WasteType::PetPlastic,
    &50,  // 50 points per kg
    &10000  // 10,000 total points
);
```

### Querying Incentives
```rust
// Get all incentives from a manufacturer
let manufacturer_incentives = client.get_incentives_by_rewarder(&manufacturer);

// Get all incentives for PET plastic
let pet_incentives = client.get_incentives_by_waste_type(&WasteType::PetPlastic);
```

### Deactivating an Incentive
```rust
client.deactivate_incentive(&incentive_id, &manufacturer);
```

## Testing Requirements

### Basic Operations
- Create incentive with valid parameters
- Retrieve incentive by ID
- Check incentive existence

### Multiple Incentives
- Manufacturer creates multiple incentives
- Different manufacturers create incentives for same waste type
- Query returns all relevant incentives

### Deactivation
- Only creator can deactivate
- Non-creator cannot deactivate
- Deactivated incentives excluded from active queries

### Budget Management
- Claiming rewards decreases remaining budget
- Cannot claim more than remaining budget
- Incentive becomes inactive when budget exhausted

## Integration with Existing Systems

### Counter System
- Uses `next_incentive_id()` for unique IDs
- Independent from waste ID counter
- Sequential and collision-free

### Material System
- Incentives apply to verified materials only
- Reward calculation based on material weight
- Integration with stats tracking for bonus points
