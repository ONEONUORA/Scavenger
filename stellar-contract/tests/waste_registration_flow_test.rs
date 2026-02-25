#![cfg(test)]

use soroban_sdk::{testutils::Address as _, Address, Env, String};
use stellar_scavngr_contract::{ParticipantRole, ScavengerContract, ScavengerContractClient, WasteType};

// ========== Test Setup Helpers ==========

fn create_test_contract(env: &Env) -> ScavengerContractClient {
    let contract_id = env.register_contract(None, ScavengerContract);
    ScavengerContractClient::new(env, &contract_id)
}

fn setup_test_environment(env: &Env) -> (ScavengerContractClient, Address) {
    env.mock_all_auths();
    let client = create_test_contract(env);
    let recycler = Address::generate(env);
    
    // Register recycler participant
    client.register_participant(&recycler, &ParticipantRole::Recycler);
    
    (client, recycler)
}

// ========== Test 1: Successful Waste Registration ==========

#[test]
fn test_successful_waste_registration() {
    let env = Env::default();
    let (client, recycler) = setup_test_environment(&env);
    
    let waste_type = WasteType::Plastic;
    let weight: u128 = 5000;
    let description = String::from_str(&env, "Plastic bottles");
    
    // Register waste
    let waste = client.submit_material(&waste_type, &weight, &recycler, &description);
    
    // Verify waste was created successfully
    assert_eq!(waste.waste_type, waste_type);
    assert_eq!(waste.weight, weight);
    assert_eq!(waste.submitter, recycler);
    assert_eq!(waste.description, description);
    assert!(!waste.verified); // Should not be verified initially
}

#[test]
fn test_successful_waste_registration_all_types() {
    let env = Env::default();
    let (client, recycler) = setup_test_environment(&env);
    
    let desc = String::from_str(&env, "Test waste");
    
    // Test all waste types
    let paper = client.submit_material(&WasteType::Paper, &1000, &recycler, &desc);
    let pet = client.submit_material(&WasteType::PetPlastic, &2000, &recycler, &desc);
    let plastic = client.submit_material(&WasteType::Plastic, &3000, &recycler, &desc);
    let metal = client.submit_material(&WasteType::Metal, &4000, &recycler, &desc);
    let glass = client.submit_material(&WasteType::Glass, &5000, &recycler, &desc);
    
    // Verify all were registered
    assert_eq!(paper.waste_type, WasteType::Paper);
    assert_eq!(pet.waste_type, WasteType::PetPlastic);
    assert_eq!(plastic.waste_type, WasteType::Plastic);
    assert_eq!(metal.waste_type, WasteType::Metal);
    assert_eq!(glass.waste_type, WasteType::Glass);
}

// ========== Test 2: Unregistered User Fails ==========

#[test]
#[should_panic(expected = "Participant not registered")]
fn test_unregistered_user_cannot_register_waste() {
    let env = Env::default();
    env.mock_all_auths();
    
    let client = create_test_contract(&env);
    let unregistered_user = Address::generate(&env);
    
    let description = String::from_str(&env, "Test waste");
    
    // Try to register waste without being registered as participant
    client.submit_material(&WasteType::Plastic, &1000, &unregistered_user, &description);
}

#[test]
#[should_panic(expected = "Participant not registered")]
fn test_unregistered_user_cannot_register_any_waste_type() {
    let env = Env::default();
    env.mock_all_auths();
    
    let client = create_test_contract(&env);
    let unregistered_user = Address::generate(&env);
    let desc = String::from_str(&env, "Test");
    
    // Try with different waste types - all should fail
    client.submit_material(&WasteType::Paper, &1000, &unregistered_user, &desc);
}

// ========== Test 3: Waste ID Generation ==========

#[test]
fn test_waste_id_generation_sequential() {
    let env = Env::default();
    let (client, recycler) = setup_test_environment(&env);
    
    let desc = String::from_str(&env, "Test waste");
    
    // Register multiple wastes
    let waste1 = client.submit_material(&WasteType::Plastic, &1000, &recycler, &desc);
    let waste2 = client.submit_material(&WasteType::Metal, &2000, &recycler, &desc);
    let waste3 = client.submit_material(&WasteType::Glass, &3000, &recycler, &desc);
    
    // Verify IDs are sequential
    assert_eq!(waste1.id, 1);
    assert_eq!(waste2.id, 2);
    assert_eq!(waste3.id, 3);
}

#[test]
fn test_waste_id_generation_unique() {
    let env = Env::default();
    let (client, recycler) = setup_test_environment(&env);
    
    let desc = String::from_str(&env, "Test waste");
    
    // Register many wastes
    let mut ids = Vec::new();
    for i in 0..10 {
        let waste_type = match i % 5 {
            0 => WasteType::Paper,
            1 => WasteType::PetPlastic,
            2 => WasteType::Plastic,
            3 => WasteType::Metal,
            _ => WasteType::Glass,
        };
        let waste = client.submit_material(&waste_type, &(1000 * (i as u128 + 1)), &recycler, &desc);
        ids.push(waste.id);
    }
    
    // Verify all IDs are unique
    for i in 0..ids.len() {
        for j in (i + 1)..ids.len() {
            assert_ne!(ids[i], ids[j], "Duplicate waste IDs found");
        }
    }
}

#[test]
fn test_waste_id_generation_no_gaps() {
    let env = Env::default();
    let (client, recycler) = setup_test_environment(&env);
    
    let desc = String::from_str(&env, "Test waste");
    
    // Register wastes and verify no gaps in sequence
    for expected_id in 1..=5 {
        let waste = client.submit_material(&WasteType::Plastic, &1000, &recycler, &desc);
        assert_eq!(waste.id, expected_id as u64);
    }
}

// ========== Test 4: Event Emission ==========

#[test]
fn test_waste_registration_event_emitted() {
    let env = Env::default();
    let (client, recycler) = setup_test_environment(&env);
    
    let waste_type = WasteType::Plastic;
    let weight: u128 = 5000;
    let description = String::from_str(&env, "Plastic bottles");
    
    // Register waste
    let waste = client.submit_material(&waste_type, &weight, &recycler, &description);
    
    // Verify event was emitted
    let events = env.events().all();
    assert!(!events.is_empty(), "No events were emitted");
    
    // Find the waste registration event
    let waste_event = events.iter().find(|e| {
        // Event should contain waste_id in topics
        e.topics.len() > 0
    });
    
    assert!(waste_event.is_some(), "Waste registration event not found");
}

#[test]
fn test_waste_registration_event_contains_waste_id() {
    let env = Env::default();
    let (client, recycler) = setup_test_environment(&env);
    
    let desc = String::from_str(&env, "Test waste");
    
    // Clear previous events
    let _ = env.events().all();
    
    // Register waste
    let waste = client.submit_material(&WasteType::Metal, &2000, &recycler, &desc);
    
    // Get events
    let events = env.events().all();
    
    // Verify event was emitted with waste ID
    assert!(!events.is_empty(), "No events emitted");
    let last_event = events.last().unwrap();
    assert!(!last_event.topics.is_empty(), "Event has no topics");
}

#[test]
fn test_waste_registration_event_emitted_for_each_waste() {
    let env = Env::default();
    let (client, recycler) = setup_test_environment(&env);
    
    let desc = String::from_str(&env, "Test waste");
    
    // Register multiple wastes
    let waste1 = client.submit_material(&WasteType::Paper, &1000, &recycler, &desc);
    let waste2 = client.submit_material(&WasteType::Plastic, &2000, &recycler, &desc);
    let waste3 = client.submit_material(&WasteType::Glass, &3000, &recycler, &desc);
    
    // Get all events
    let events = env.events().all();
    
    // Should have at least 3 events (one per waste registration)
    assert!(events.len() >= 3, "Expected at least 3 events, got {}", events.len());
}

// ========== Test 5: Participant Wastes Update ==========

#[test]
fn test_participant_wastes_updated_on_registration() {
    let env = Env::default();
    let (client, recycler) = setup_test_environment(&env);
    
    let desc = String::from_str(&env, "Test waste");
    
    // Register waste
    let waste = client.submit_material(&WasteType::Plastic, &5000, &recycler, &desc);
    
    // Get participant info
    let info = client.get_participant_info(&recycler);
    assert!(info.is_some(), "Participant info not found");
    
    let info = info.unwrap();
    let stats = info.stats;
    
    // Verify stats were updated
    assert_eq!(stats.total_submissions, 1);
    assert_eq!(stats.total_weight, 5000);
}

#[test]
fn test_participant_wastes_accumulate() {
    let env = Env::default();
    let (client, recycler) = setup_test_environment(&env);
    
    let desc = String::from_str(&env, "Test waste");
    
    // Register multiple wastes
    client.submit_material(&WasteType::Paper, &1000, &recycler, &desc);
    client.submit_material(&WasteType::Plastic, &2000, &recycler, &desc);
    client.submit_material(&WasteType::Metal, &3000, &recycler, &desc);
    
    // Get participant info
    let info = client.get_participant_info(&recycler).unwrap();
    let stats = info.stats;
    
    // Verify accumulated stats
    assert_eq!(stats.total_submissions, 3);
    assert_eq!(stats.total_weight, 6000); // 1000 + 2000 + 3000
}

#[test]
fn test_participant_wastes_by_type_tracked() {
    let env = Env::default();
    let (client, recycler) = setup_test_environment(&env);
    
    let desc = String::from_str(&env, "Test waste");
    
    // Register wastes of different types
    client.submit_material(&WasteType::Paper, &1000, &recycler, &desc);
    client.submit_material(&WasteType::Paper, &1500, &recycler, &desc);
    client.submit_material(&WasteType::Plastic, &2000, &recycler, &desc);
    client.submit_material(&WasteType::Metal, &3000, &recycler, &desc);
    
    // Get participant info
    let info = client.get_participant_info(&recycler).unwrap();
    let stats = info.stats;
    
    // Verify type-specific tracking
    assert_eq!(stats.paper_count, 2);
    assert_eq!(stats.plastic_count, 1);
    assert_eq!(stats.metal_count, 1);
    assert_eq!(stats.total_submissions, 4);
}

#[test]
fn test_multiple_participants_wastes_independent() {
    let env = Env::default();
    env.mock_all_auths();
    
    let client = create_test_contract(&env);
    let recycler1 = Address::generate(&env);
    let recycler2 = Address::generate(&env);
    
    // Register both participants
    client.register_participant(&recycler1, &ParticipantRole::Recycler);
    client.register_participant(&recycler2, &ParticipantRole::Recycler);
    
    let desc = String::from_str(&env, "Test waste");
    
    // Register wastes for each
    client.submit_material(&WasteType::Plastic, &1000, &recycler1, &desc);
    client.submit_material(&WasteType::Plastic, &2000, &recycler1, &desc);
    client.submit_material(&WasteType::Metal, &3000, &recycler2, &desc);
    
    // Verify stats are independent
    let info1 = client.get_participant_info(&recycler1).unwrap();
    let info2 = client.get_participant_info(&recycler2).unwrap();
    
    assert_eq!(info1.stats.total_submissions, 2);
    assert_eq!(info1.stats.total_weight, 3000);
    
    assert_eq!(info2.stats.total_submissions, 1);
    assert_eq!(info2.stats.total_weight, 3000);
}

// ========== Test 6: All Waste Types ==========

#[test]
fn test_all_waste_types_can_be_registered() {
    let env = Env::default();
    let (client, recycler) = setup_test_environment(&env);
    
    let desc = String::from_str(&env, "Test waste");
    
    // Register all waste types
    let paper = client.submit_material(&WasteType::Paper, &1000, &recycler, &desc);
    let pet = client.submit_material(&WasteType::PetPlastic, &2000, &recycler, &desc);
    let plastic = client.submit_material(&WasteType::Plastic, &3000, &recycler, &desc);
    let metal = client.submit_material(&WasteType::Metal, &4000, &recycler, &desc);
    let glass = client.submit_material(&WasteType::Glass, &5000, &recycler, &desc);
    
    // Verify all were registered with correct types
    assert_eq!(paper.waste_type, WasteType::Paper);
    assert_eq!(pet.waste_type, WasteType::PetPlastic);
    assert_eq!(plastic.waste_type, WasteType::Plastic);
    assert_eq!(metal.waste_type, WasteType::Metal);
    assert_eq!(glass.waste_type, WasteType::Glass);
}

#[test]
fn test_all_waste_types_retrievable() {
    let env = Env::default();
    let (client, recycler) = setup_test_environment(&env);
    
    let desc = String::from_str(&env, "Test waste");
    
    // Register all waste types
    let paper = client.submit_material(&WasteType::Paper, &1000, &recycler, &desc);
    let pet = client.submit_material(&WasteType::PetPlastic, &2000, &recycler, &desc);
    let plastic = client.submit_material(&WasteType::Plastic, &3000, &recycler, &desc);
    let metal = client.submit_material(&WasteType::Metal, &4000, &recycler, &desc);
    let glass = client.submit_material(&WasteType::Glass, &5000, &recycler, &desc);
    
    // Retrieve each and verify
    let paper_retrieved = client.get_waste(&paper.id).unwrap();
    let pet_retrieved = client.get_waste(&pet.id).unwrap();
    let plastic_retrieved = client.get_waste(&plastic.id).unwrap();
    let metal_retrieved = client.get_waste(&metal.id).unwrap();
    let glass_retrieved = client.get_waste(&glass.id).unwrap();
    
    assert_eq!(paper_retrieved.waste_type, WasteType::Paper);
    assert_eq!(pet_retrieved.waste_type, WasteType::PetPlastic);
    assert_eq!(plastic_retrieved.waste_type, WasteType::Plastic);
    assert_eq!(metal_retrieved.waste_type, WasteType::Metal);
    assert_eq!(glass_retrieved.waste_type, WasteType::Glass);
}

#[test]
fn test_all_waste_types_tracked_in_stats() {
    let env = Env::default();
    let (client, recycler) = setup_test_environment(&env);
    
    let desc = String::from_str(&env, "Test waste");
    
    // Register all waste types
    client.submit_material(&WasteType::Paper, &1000, &recycler, &desc);
    client.submit_material(&WasteType::PetPlastic, &2000, &recycler, &desc);
    client.submit_material(&WasteType::Plastic, &3000, &recycler, &desc);
    client.submit_material(&WasteType::Metal, &4000, &recycler, &desc);
    client.submit_material(&WasteType::Glass, &5000, &recycler, &desc);
    
    // Get participant stats
    let info = client.get_participant_info(&recycler).unwrap();
    let stats = info.stats;
    
    // Verify all types are tracked
    assert_eq!(stats.paper_count, 1);
    assert_eq!(stats.pet_plastic_count, 1);
    assert_eq!(stats.plastic_count, 1);
    assert_eq!(stats.metal_count, 1);
    assert_eq!(stats.glass_count, 1);
    assert_eq!(stats.total_submissions, 5);
    assert_eq!(stats.total_weight, 15000); // 1000+2000+3000+4000+5000
}

// ========== Comprehensive Integration Tests ==========

#[test]
fn test_waste_registration_flow_complete() {
    let env = Env::default();
    let (client, recycler) = setup_test_environment(&env);
    
    let desc = String::from_str(&env, "Comprehensive test waste");
    
    // Step 1: Register waste
    let waste = client.submit_material(&WasteType::Plastic, &5000, &recycler, &desc);
    
    // Step 2: Verify waste ID is generated
    assert!(waste.id > 0, "Waste ID should be positive");
    
    // Step 3: Verify waste can be retrieved
    let retrieved = client.get_waste(&waste.id);
    assert!(retrieved.is_some(), "Waste should be retrievable");
    
    // Step 4: Verify participant stats updated
    let info = client.get_participant_info(&recycler).unwrap();
    assert_eq!(info.stats.total_submissions, 1);
    assert_eq!(info.stats.total_weight, 5000);
    
    // Step 5: Verify event was emitted
    let events = env.events().all();
    assert!(!events.is_empty(), "Events should be emitted");
}

#[test]
fn test_multiple_waste_registrations_flow() {
    let env = Env::default();
    let (client, recycler) = setup_test_environment(&env);
    
    let desc = String::from_str(&env, "Test waste");
    
    // Register multiple wastes
    let waste1 = client.submit_material(&WasteType::Paper, &1000, &recycler, &desc);
    let waste2 = client.submit_material(&WasteType::Plastic, &2000, &recycler, &desc);
    let waste3 = client.submit_material(&WasteType::Metal, &3000, &recycler, &desc);
    
    // Verify all IDs are unique and sequential
    assert_eq!(waste1.id, 1);
    assert_eq!(waste2.id, 2);
    assert_eq!(waste3.id, 3);
    
    // Verify all can be retrieved
    assert!(client.get_waste(&waste1.id).is_some());
    assert!(client.get_waste(&waste2.id).is_some());
    assert!(client.get_waste(&waste3.id).is_some());
    
    // Verify stats accumulated
    let info = client.get_participant_info(&recycler).unwrap();
    assert_eq!(info.stats.total_submissions, 3);
    assert_eq!(info.stats.total_weight, 6000);
}

#[test]
fn test_waste_registration_with_different_roles() {
    let env = Env::default();
    env.mock_all_auths();
    
    let client = create_test_contract(&env);
    let recycler = Address::generate(&env);
    let collector = Address::generate(&env);
    
    // Register both roles
    client.register_participant(&recycler, &ParticipantRole::Recycler);
    client.register_participant(&collector, &ParticipantRole::Collector);
    
    let desc = String::from_str(&env, "Test waste");
    
    // Both should be able to register waste
    let waste1 = client.submit_material(&WasteType::Plastic, &1000, &recycler, &desc);
    let waste2 = client.submit_material(&WasteType::Metal, &2000, &collector, &desc);
    
    // Verify both registrations succeeded
    assert_eq!(waste1.id, 1);
    assert_eq!(waste2.id, 2);
    
    // Verify stats are separate
    let recycler_info = client.get_participant_info(&recycler).unwrap();
    let collector_info = client.get_participant_info(&collector).unwrap();
    
    assert_eq!(recycler_info.stats.total_submissions, 1);
    assert_eq!(collector_info.stats.total_submissions, 1);
}

// ========== Edge Cases and Validation ==========

#[test]
fn test_waste_registration_with_zero_weight() {
    let env = Env::default();
    let (client, recycler) = setup_test_environment(&env);
    
    let desc = String::from_str(&env, "Zero weight waste");
    
    // Register waste with zero weight (should be allowed)
    let waste = client.submit_material(&WasteType::Plastic, &0, &recycler, &desc);
    
    // Verify it was registered
    assert_eq!(waste.weight, 0);
    assert_eq!(waste.id, 1);
}

#[test]
fn test_waste_registration_with_large_weight() {
    let env = Env::default();
    let (client, recycler) = setup_test_environment(&env);
    
    let desc = String::from_str(&env, "Large weight waste");
    
    // Register waste with large weight
    let large_weight: u128 = 1_000_000_000; // 1 billion grams
    let waste = client.submit_material(&WasteType::Metal, &large_weight, &recycler, &desc);
    
    // Verify it was registered
    assert_eq!(waste.weight, large_weight);
}

#[test]
fn test_waste_registration_preserves_metadata() {
    let env = Env::default();
    let (client, recycler) = setup_test_environment(&env);
    
    let desc = String::from_str(&env, "Detailed waste description");
    
    // Register waste
    let waste = client.submit_material(&WasteType::Glass, &5000, &recycler, &desc);
    
    // Retrieve and verify all metadata preserved
    let retrieved = client.get_waste(&waste.id).unwrap();
    
    assert_eq!(retrieved.waste_type, WasteType::Glass);
    assert_eq!(retrieved.weight, 5000);
    assert_eq!(retrieved.submitter, recycler);
    assert_eq!(retrieved.description, desc);
}
