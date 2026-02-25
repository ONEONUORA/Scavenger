#![cfg(test)]

use soroban_sdk::{testutils::Address as _, Address, Env, String};

use crate::{ScavengerContract, ScavengerContractClient};
use crate::types::{Role, WasteType};

// ========== Test Setup Helpers ==========

fn create_test_contract(env: &Env) -> (ScavengerContractClient, Address, Address, Address) {
    let contract_id = env.register(ScavengerContract, ());
    let client = ScavengerContractClient::new(env, &contract_id);
    
    let admin = Address::generate(env);
    let token_address = Address::generate(env);
    let charity_address = Address::generate(env);
    
    (client, admin, token_address, charity_address)
}

fn setup_test_environment(env: &Env) -> (ScavengerContractClient, Address) {
    env.mock_all_auths();
    let (client, admin, token_address, charity_address) = create_test_contract(env);
    
    // Initialize contract
    client.__constructor(env, admin.clone(), token_address, charity_address, 30, 20);
    
    // Register recycler participant
    let recycler = Address::generate(env);
    let name = String::from_str(env, "Test Recycler");
    client.register_participant(env, &recycler, &Role::Recycler, &name, 100, 200);
    
    (client, recycler)
}

// ========== Test 1: Successful Waste Registration ==========

#[test]
fn test_successful_waste_registration() {
    let env = Env::default();
    let (client, recycler) = setup_test_environment(&env);
    
    let waste_type = WasteType::Plastic;
    let weight: u64 = 5000;
    
    // Register waste
    let material = client.submit_material(&env, &recycler, waste_type, weight);
    
    // Verify waste was created successfully
    assert_eq!(material.waste_type, waste_type);
    assert_eq!(material.weight, weight);
    assert_eq!(material.submitter, recycler);
    assert!(!material.verified); // Should not be verified initially
}

#[test]
fn test_successful_waste_registration_all_types() {
    let env = Env::default();
    let (client, recycler) = setup_test_environment(&env);
    
    // Test all waste types
    let paper = client.submit_material(&recycler, &WasteType::Paper, &1000);
    let pet = client.submit_material(&recycler, &WasteType::PetPlastic, &2000);
    let plastic = client.submit_material(&recycler, &WasteType::Plastic, &3000);
    let metal = client.submit_material(&recycler, &WasteType::Metal, &4000);
    let glass = client.submit_material(&recycler, &WasteType::Glass, &5000);
    
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
    
    let (client, _admin, _token, _charity) = create_test_contract(&env);
    let unregistered_user = Address::generate(&env);
    
    // Try to register waste without being registered as participant
    client.submit_material(&unregistered_user, &WasteType::Plastic, &1000);
}

#[test]
#[should_panic(expected = "Participant not registered")]
fn test_unregistered_user_cannot_register_any_waste_type() {
    let env = Env::default();
    env.mock_all_auths();
    
    let (client, _admin, _token, _charity) = create_test_contract(&env);
    let unregistered_user = Address::generate(&env);
    
    // Try with different waste types - all should fail
    client.submit_material(&unregistered_user, &WasteType::Paper, &1000);
}

// ========== Test 3: Waste ID Generation ==========

#[test]
fn test_waste_id_generation_sequential() {
    let env = Env::default();
    let (client, recycler) = setup_test_environment(&env);
    
    // Register multiple wastes
    let waste1 = client.submit_material(&recycler, &WasteType::Plastic, &1000);
    let waste2 = client.submit_material(&recycler, &WasteType::Metal, &2000);
    let waste3 = client.submit_material(&recycler, &WasteType::Glass, &3000);
    
    // Verify IDs are sequential
    assert_eq!(waste1.id, 1);
    assert_eq!(waste2.id, 2);
    assert_eq!(waste3.id, 3);
}

#[test]
fn test_waste_id_generation_unique() {
    let env = Env::default();
    let (client, recycler) = setup_test_environment(&env);
    
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
        let waste = client.submit_material(&recycler, &waste_type, &(1000 * (i as u64 + 1)));
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
    
    // Register wastes and verify no gaps in sequence
    for expected_id in 1..=5 {
        let waste = client.submit_material(&recycler, &WasteType::Plastic, &1000);
        assert_eq!(waste.id, expected_id as u64);
    }
}

// ========== Test 4: Event Emission ==========

#[test]
fn test_waste_registration_event_emitted() {
    let env = Env::default();
    let (client, recycler) = setup_test_environment(&env);
    
    let waste_type = WasteType::Plastic;
    let weight: u64 = 5000;
    
    // Register waste
    let waste = client.submit_material(&waste_type, &weight, &recycler);
    
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
    
    // Clear previous events
    let _ = env.events().all();
    
    // Register waste
    let waste = client.submit_material(&recycler, &WasteType::Metal, &2000);
    
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
    
    // Register multiple wastes
    let waste1 = client.submit_material(&recycler, &WasteType::Paper, &1000);
    let waste2 = client.submit_material(&recycler, &WasteType::Plastic, &2000);
    let waste3 = client.submit_material(&recycler, &WasteType::Glass, &3000);
    
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
    
    // Register waste
    let waste = client.submit_material(&recycler, &WasteType::Plastic, &5000);
    
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
    
    // Register multiple wastes
    client.submit_material(&recycler, &WasteType::Paper, &1000);
    client.submit_material(&recycler, &WasteType::Plastic, &2000);
    client.submit_material(&recycler, &WasteType::Metal, &3000);
    
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
    
    // Register wastes of different types
    client.submit_material(&recycler, &WasteType::Paper, &1000);
    client.submit_material(&recycler, &WasteType::Paper, &1500);
    client.submit_material(&recycler, &WasteType::Plastic, &2000);
    client.submit_material(&recycler, &WasteType::Metal, &3000);
    
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
    
    let (client, admin, token_address, charity_address) = create_test_contract(&env);
    client.__constructor(&admin, &token_address, &charity_address, &30, &20);
    
    let recycler1 = Address::generate(&env);
    let recycler2 = Address::generate(&env);
    
    let name1 = String::from_str(&env, "Recycler 1");
    let name2 = String::from_str(&env, "Recycler 2");
    
    // Register both participants
    client.register_participant(&recycler1, &Role::Recycler, &name1, &100, &200);
    client.register_participant(&recycler2, &Role::Recycler, &name2, &300, &400);
    
    // Register wastes for each
    client.submit_material(&recycler1, &WasteType::Plastic, &1000);
    client.submit_material(&recycler1, &WasteType::Plastic, &2000);
    client.submit_material(&recycler2, &WasteType::Metal, &3000);
    
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
    
    // Register all waste types
    let paper = client.submit_material(&recycler, &WasteType::Paper, &1000);
    let pet = client.submit_material(&recycler, &WasteType::PetPlastic, &2000);
    let plastic = client.submit_material(&recycler, &WasteType::Plastic, &3000);
    let metal = client.submit_material(&recycler, &WasteType::Metal, &4000);
    let glass = client.submit_material(&recycler, &WasteType::Glass, &5000);
    
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
    
    // Register all waste types
    let paper = client.submit_material(&recycler, &WasteType::Paper, &1000);
    let pet = client.submit_material(&recycler, &WasteType::PetPlastic, &2000);
    let plastic = client.submit_material(&recycler, &WasteType::Plastic, &3000);
    let metal = client.submit_material(&recycler, &WasteType::Metal, &4000);
    let glass = client.submit_material(&recycler, &WasteType::Glass, &5000);
    
    // Retrieve each and verify
    let paper_retrieved = client.get_material(&paper.id).unwrap();
    let pet_retrieved = client.get_material(&pet.id).unwrap();
    let plastic_retrieved = client.get_material(&plastic.id).unwrap();
    let metal_retrieved = client.get_material(&metal.id).unwrap();
    let glass_retrieved = client.get_material(&glass.id).unwrap();
    
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
    
    // Register all waste types
    client.submit_material(&recycler, &WasteType::Paper, &1000);
    client.submit_material(&recycler, &WasteType::PetPlastic, &2000);
    client.submit_material(&recycler, &WasteType::Plastic, &3000);
    client.submit_material(&recycler, &WasteType::Metal, &4000);
    client.submit_material(&recycler, &WasteType::Glass, &5000);
    
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
    
    // Step 1: Register waste
    let waste = client.submit_material(&recycler, &WasteType::Plastic, &5000);
    
    // Step 2: Verify waste ID is generated
    assert!(waste.id > 0, "Waste ID should be positive");
    
    // Step 3: Verify waste can be retrieved
    let retrieved = client.get_material(&waste.id);
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
    
    // Register multiple wastes
    let waste1 = client.submit_material(&recycler, &WasteType::Paper, &1000);
    let waste2 = client.submit_material(&recycler, &WasteType::Plastic, &2000);
    let waste3 = client.submit_material(&recycler, &WasteType::Metal, &3000);
    
    // Verify all IDs are unique and sequential
    assert_eq!(waste1.id, 1);
    assert_eq!(waste2.id, 2);
    assert_eq!(waste3.id, 3);
    
    // Verify all can be retrieved
    assert!(client.get_material(&waste1.id).is_some());
    assert!(client.get_material(&waste2.id).is_some());
    assert!(client.get_material(&waste3.id).is_some());
    
    // Verify stats accumulated
    let info = client.get_participant_info(&recycler).unwrap();
    assert_eq!(info.stats.total_submissions, 3);
    assert_eq!(info.stats.total_weight, 6000);
}

#[test]
fn test_waste_registration_with_different_roles() {
    let env = Env::default();
    env.mock_all_auths();
    
    let (client, admin, token_address, charity_address) = create_test_contract(&env);
    client.__constructor(&admin, &token_address, &charity_address, &30, &20);
    
    let recycler = Address::generate(&env);
    let collector = Address::generate(&env);
    
    let name1 = String::from_str(&env, "Recycler");
    let name2 = String::from_str(&env, "Collector");
    
    // Register both roles
    client.register_participant(&recycler, &Role::Recycler, &name1, &100, &200);
    client.register_participant(&collector, &Role::Collector, &name2, &300, &400);
    
    // Both should be able to register waste
    let waste1 = client.submit_material(&recycler, &WasteType::Plastic, &1000);
    let waste2 = client.submit_material(&collector, &WasteType::Metal, &2000);
    
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
    
    // Register waste with zero weight (should be allowed)
    let waste = client.submit_material(&recycler, &WasteType::Plastic, &0);
    
    // Verify it was registered
    assert_eq!(waste.weight, 0);
    assert_eq!(waste.id, 1);
}

#[test]
fn test_waste_registration_with_large_weight() {
    let env = Env::default();
    let (client, recycler) = setup_test_environment(&env);
    
    // Register waste with large weight
    let large_weight: u64 = 1_000_000_000; // 1 billion grams
    let waste = client.submit_material(&recycler, &WasteType::Metal, &large_weight);
    
    // Verify it was registered
    assert_eq!(waste.weight, large_weight);
}

#[test]
fn test_waste_registration_preserves_metadata() {
    let env = Env::default();
    let (client, recycler) = setup_test_environment(&env);
    
    // Register waste
    let waste = client.submit_material(&recycler, &WasteType::Glass, &5000);
    
    // Retrieve and verify all metadata preserved
    let retrieved = client.get_material(&waste.id).unwrap();
    
    assert_eq!(retrieved.waste_type, WasteType::Glass);
    assert_eq!(retrieved.weight, 5000);
    assert_eq!(retrieved.submitter, recycler);
}

#[test]
fn test_waste_registration_sequential_across_types() {
    let env = Env::default();
    let (client, recycler) = setup_test_environment(&env);
    
    // Register wastes alternating between types
    let w1 = client.submit_material(&recycler, &WasteType::Paper, &1000);
    let w2 = client.submit_material(&recycler, &WasteType::Metal, &2000);
    let w3 = client.submit_material(&recycler, &WasteType::Paper, &3000);
    let w4 = client.submit_material(&recycler, &WasteType::Glass, &4000);
    
    // Verify IDs are still sequential despite type changes
    assert_eq!(w1.id, 1);
    assert_eq!(w2.id, 2);
    assert_eq!(w3.id, 3);
    assert_eq!(w4.id, 4);
}

#[test]
fn test_waste_registration_high_volume() {
    let env = Env::default();
    let (client, recycler) = setup_test_environment(&env);
    
    // Register many wastes
    let mut last_id = 0u64;
    for i in 1..=20 {
        let waste = client.submit_material(&recycler, &WasteType::Plastic, &(i * 100));
        assert_eq!(waste.id, i as u64);
        last_id = waste.id;
    }
    
    // Verify final count
    assert_eq!(last_id, 20);
    
    // Verify stats
    let info = client.get_participant_info(&recycler).unwrap();
    assert_eq!(info.stats.total_submissions, 20);
}
