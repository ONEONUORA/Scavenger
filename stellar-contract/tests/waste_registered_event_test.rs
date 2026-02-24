use soroban_sdk::{
    symbol_short, testutils::{Address as _, Events}, Address, Env, IntoVal, Symbol, Vec,
};
use stellar_scavngr_contract::{ParticipantRole, ScavengerContract, ScavengerContractClient, WasteType};

#[test]
fn test_waste_registered_event_emitted() {
    let env = Env::default();
    let contract_id = env.register_contract(None, ScavengerContract);
    let client = ScavengerContractClient::new(&env, &contract_id);

    let recycler = Address::generate(&env);
    env.mock_all_auths();

    // Register participant
    client.register_participant(&recycler, &ParticipantRole::Recycler);

    // Recycle waste
    let waste_type = WasteType::Plastic;
    let weight: u128 = 2500;
    let latitude: i128 = 40_500_000;
    let longitude: i128 = -74_000_000;

    let waste_id = client.recycle_waste(
        &waste_type,
        &weight,
        &recycler,
        &latitude,
        &longitude,
    );

    // Verify event was emitted
    let events = env.events().all();
    let event = events.last().unwrap();

    // Check event topics
    let expected_topics: Vec<soroban_sdk::Val> = (
        symbol_short!("recycled"),
        waste_id,
    ).into_val(&env);
    
    assert_eq!(event.topics, expected_topics);

    // Check event data contains all required fields
    let event_data: (WasteType, u128, Address, i128, i128) = event.data.try_into_val(&env).unwrap();
    assert_eq!(event_data.0, waste_type);
    assert_eq!(event_data.1, weight);
    assert_eq!(event_data.2, recycler);
    assert_eq!(event_data.3, latitude);
    assert_eq!(event_data.4, longitude);
}

#[test]
fn test_waste_registered_event_fields() {
    let env = Env::default();
    let contract_id = env.register_contract(None, ScavengerContract);
    let client = ScavengerContractClient::new(&env, &contract_id);

    let recycler = Address::generate(&env);
    env.mock_all_auths();

    client.register_participant(&recycler, &ParticipantRole::Recycler);

    // Test with different waste types and values
    let test_cases = vec![
        (WasteType::Paper, 1000u128, 51_500_000i128, 0i128),
        (WasteType::Metal, 5000u128, -33_900_000i128, 151_200_000i128),
        (WasteType::Glass, 3500u128, 35_700_000i128, 139_700_000i128),
    ];

    for (waste_type, weight, lat, lon) in test_cases {
        let waste_id = client.recycle_waste(
            &waste_type,
            &weight,
            &recycler,
            &lat,
            &lon,
        );

        // Get the last event
        let events = env.events().all();
        let event = events.last().unwrap();

        // Verify waste_id in topics
        let topics: Vec<soroban_sdk::Val> = (
            symbol_short!("recycled"),
            waste_id,
        ).into_val(&env);
        assert_eq!(event.topics, topics);

        // Verify all fields in event data
        let event_data: (WasteType, u128, Address, i128, i128) = event.data.try_into_val(&env).unwrap();
        assert_eq!(event_data.0, waste_type, "Waste type mismatch");
        assert_eq!(event_data.1, weight, "Weight mismatch");
        assert_eq!(event_data.2, recycler, "Recycler address mismatch");
        assert_eq!(event_data.3, lat, "Latitude mismatch");
        assert_eq!(event_data.4, lon, "Longitude mismatch");
    }
}

#[test]
fn test_waste_registered_event_multiple_wastes() {
    let env = Env::default();
    let contract_id = env.register_contract(None, ScavengerContract);
    let client = ScavengerContractClient::new(&env, &contract_id);

    let recycler1 = Address::generate(&env);
    let recycler2 = Address::generate(&env);
    env.mock_all_auths();

    client.register_participant(&recycler1, &ParticipantRole::Recycler);
    client.register_participant(&recycler2, &ParticipantRole::Recycler);

    // Register multiple wastes
    let waste_id1 = client.recycle_waste(
        &WasteType::Plastic,
        &2000,
        &recycler1,
        &40_000_000,
        &-74_000_000,
    );

    let waste_id2 = client.recycle_waste(
        &WasteType::Metal,
        &3000,
        &recycler2,
        &41_000_000,
        &-73_000_000,
    );

    // Verify both events were emitted
    let all_events = env.events().all();
    assert!(all_events.len() >= 2, "Expected at least 2 events");

    // Check the last two events correspond to our waste registrations
    let events_vec: Vec<_> = all_events.iter().collect();
    let event1 = &events_vec[events_vec.len() - 2];
    let event2 = &events_vec[events_vec.len() - 1];

    // Verify first waste event
    let topics1: Vec<soroban_sdk::Val> = (
        symbol_short!("recycled"),
        waste_id1,
    ).into_val(&env);
    assert_eq!(event1.topics, topics1);

    let data1: (WasteType, u128, Address, i128, i128) = event1.data.try_into_val(&env).unwrap();
    assert_eq!(data1.0, WasteType::Plastic);
    assert_eq!(data1.2, recycler1);

    // Verify second waste event
    let topics2: Vec<soroban_sdk::Val> = (
        symbol_short!("recycled"),
        waste_id2,
    ).into_val(&env);
    assert_eq!(event2.topics, topics2);

    let data2: (WasteType, u128, Address, i128, i128) = event2.data.try_into_val(&env).unwrap();
    assert_eq!(data2.0, WasteType::Metal);
    assert_eq!(data2.2, recycler2);
}

#[test]
fn test_waste_registered_event_with_boundary_coordinates() {
    let env = Env::default();
    let contract_id = env.register_contract(None, ScavengerContract);
    let client = ScavengerContractClient::new(&env, &contract_id);

    let recycler = Address::generate(&env);
    env.mock_all_auths();

    client.register_participant(&recycler, &ParticipantRole::Recycler);

    // Test with boundary coordinates
    let max_lat: i128 = 90_000_000;
    let max_lon: i128 = 180_000_000;
    let min_lat: i128 = -90_000_000;
    let min_lon: i128 = -180_000_000;

    let boundary_tests = vec![
        (max_lat, max_lon),
        (min_lat, min_lon),
        (0, 0),
        (max_lat, min_lon),
        (min_lat, max_lon),
    ];

    for (lat, lon) in boundary_tests {
        let waste_id = client.recycle_waste(
            &WasteType::PetPlastic,
            &1500,
            &recycler,
            &lat,
            &lon,
        );

        let events = env.events().all();
        let event = events.last().unwrap();

        let event_data: (WasteType, u128, Address, i128, i128) = event.data.try_into_val(&env).unwrap();
        assert_eq!(event_data.3, lat, "Latitude should match");
        assert_eq!(event_data.4, lon, "Longitude should match");
    }
}

#[test]
fn test_waste_registered_event_symbol() {
    let env = Env::default();
    let contract_id = env.register_contract(None, ScavengerContract);
    let client = ScavengerContractClient::new(&env, &contract_id);

    let recycler = Address::generate(&env);
    env.mock_all_auths();

    client.register_participant(&recycler, &ParticipantRole::Recycler);

    client.recycle_waste(
        &WasteType::Paper,
        &1000,
        &recycler,
        &40_000_000,
        &-74_000_000,
    );

    let events = env.events().all();
    let event = events.last().unwrap();

    // Extract the symbol from topics
    let symbol: Symbol = event.topics.get(0).unwrap().try_into_val(&env).unwrap();
    assert_eq!(symbol, symbol_short!("recycled"));
}
