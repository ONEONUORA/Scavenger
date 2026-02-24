# Waste Registered Event - Usage Guide

## Overview

The `waste_registered` event (symbol: "recycled") is emitted whenever waste is successfully registered in the system through the `recycle_waste` function.

## Event Structure

### Event Topics
```rust
(Symbol("recycled"), waste_id: u128)
```

### Event Data
```rust
(
    waste_type: WasteType,
    weight: u128,
    recycler: Address,
    latitude: i128,
    longitude: i128
)
```

## Field Descriptions

| Field | Type | Description |
|-------|------|-------------|
| `waste_id` | `u128` | Unique identifier for the registered waste (in topics) |
| `waste_type` | `WasteType` | Type of waste (Paper, PetPlastic, Plastic, Metal, Glass) |
| `weight` | `u128` | Weight of the waste in grams |
| `recycler` | `Address` | Address of the participant who registered the waste |
| `latitude` | `i128` | Latitude coordinate scaled by 1e6 (e.g., 40.5° = 40_500_000) |
| `longitude` | `i128` | Longitude coordinate scaled by 1e6 (e.g., -74.0° = -74_000_000) |

## When is the Event Emitted?

The event is emitted in the `recycle_waste` function after:
1. ✅ Participant authentication is verified
2. ✅ Participant registration is confirmed
3. ✅ Waste ID is generated
4. ✅ Waste object is created and stored
5. ✅ Waste is added to participant's waste list

## Example: Listening for Events in Tests

```rust
use soroban_sdk::{symbol_short, testutils::Events, Env, IntoVal};

#[test]
fn test_listen_for_waste_registered() {
    let env = Env::default();
    // ... setup contract and client ...
    
    // Register waste
    let waste_id = client.recycle_waste(
        &WasteType::Plastic,
        &2500,
        &recycler,
        &40_500_000,
        &-74_000_000,
    );
    
    // Get events
    let events = env.events().all();
    let event = events.last().unwrap();
    
    // Extract event data
    let (waste_type, weight, recycler_addr, lat, lon): 
        (WasteType, u128, Address, i128, i128) = 
        event.data.try_into_val(&env).unwrap();
    
    // Use the data
    println!("Waste {} registered: {} grams of {:?}", 
        waste_id, weight, waste_type);
}
```

## Example: Frontend Integration

When integrating with a frontend application, you can listen for these events to:

1. **Update UI in Real-Time**: Show newly registered waste items
2. **Track Waste Flow**: Monitor waste registration across the network
3. **Generate Analytics**: Aggregate data about waste types and locations
4. **Notify Users**: Alert users when waste is registered in their area

```javascript
// Pseudo-code for frontend event listening
contract.on('recycled', (wasteId, data) => {
    const { waste_type, weight, recycler, latitude, longitude } = data;
    
    // Update map with new waste location
    map.addMarker({
        lat: latitude / 1_000_000,
        lng: longitude / 1_000_000,
        type: waste_type,
        weight: weight / 1000 // Convert to kg
    });
    
    // Update statistics
    updateStats(waste_type, weight);
});
```

## Coordinate Scaling

Coordinates are scaled by 1,000,000 (1e6) to maintain precision while using integer types:

```rust
// Converting from decimal degrees to scaled integer
let latitude_degrees = 40.7128;  // New York City
let latitude_scaled = (latitude_degrees * 1_000_000.0) as i128;  // 40_712_800

// Converting back to decimal degrees
let latitude_degrees = latitude_scaled as f64 / 1_000_000.0;  // 40.7128
```

### Valid Coordinate Ranges

- **Latitude**: -90,000,000 to 90,000,000 (-90° to 90°)
- **Longitude**: -180,000,000 to 180,000,000 (-180° to 180°)

## Common Use Cases

### 1. Waste Registration Tracking
Monitor all waste registrations in the system:
```rust
// Filter events by symbol
let waste_events: Vec<_> = env.events().all()
    .iter()
    .filter(|e| {
        let symbol: Symbol = e.topics.get(0).unwrap().try_into_val(&env).unwrap();
        symbol == symbol_short!("recycled")
    })
    .collect();
```

### 2. Participant Activity Monitoring
Track waste registrations by a specific participant:
```rust
let participant_wastes: Vec<_> = env.events().all()
    .iter()
    .filter(|e| {
        let data: (WasteType, u128, Address, i128, i128) = 
            e.data.try_into_val(&env).unwrap();
        data.2 == target_participant
    })
    .collect();
```

### 3. Geographic Analysis
Find waste registrations in a specific area:
```rust
let area_wastes: Vec<_> = env.events().all()
    .iter()
    .filter(|e| {
        let data: (WasteType, u128, Address, i128, i128) = 
            e.data.try_into_val(&env).unwrap();
        let (_, _, _, lat, lon) = data;
        lat >= min_lat && lat <= max_lat && 
        lon >= min_lon && lon <= max_lon
    })
    .collect();
```

### 4. Waste Type Statistics
Aggregate data by waste type:
```rust
let mut stats = HashMap::new();
for event in env.events().all().iter() {
    let (waste_type, weight, _, _, _): (WasteType, u128, Address, i128, i128) = 
        event.data.try_into_val(&env).unwrap();
    *stats.entry(waste_type).or_insert(0) += weight;
}
```

## Testing Best Practices

1. **Always verify event emission**: Check that events are emitted for every waste registration
2. **Validate all fields**: Ensure each field contains the expected value
3. **Test edge cases**: Test with boundary coordinates and various waste types
4. **Test multiple registrations**: Verify events are emitted correctly for multiple wastes
5. **Check event ordering**: Ensure events are emitted in the correct sequence

## Related Functions

- `recycle_waste()`: Registers waste and emits this event
- `get_waste()`: Retrieves waste information by ID
- `get_participant_wastes()`: Gets all wastes for a participant

## Notes

- The event symbol "recycled" is consistent with the action being performed
- All numeric values use appropriate integer types for blockchain efficiency
- The event is emitted only after successful waste registration
- Event data is immutable once emitted
