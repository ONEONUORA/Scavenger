#![cfg(test)]

use soroban_sdk::{testutils::Address as _, Address, Env, String};
use stellar_scavngr_contract::{ParticipantRole, ScavengerContract, ScavengerContractClient, WasteType};

// ========== Basic Functionality Tests ==========

#[test]
fn test_get_participant_info_returns_participant_and_stats() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, ScavengerContract);
    let client = ScavengerContractClient::new(&env, &contract_id);
    let user = Address::generate(&env);

    // Register participant
    client.register_participant(&user, &ParticipantRole::Collector);

    // Get participant info
    let info = client.get_participant_info(&user);

    assert!(info.is_some());
    let info = info.unwrap();
    
    // Verify participant data
    assert_eq!(info.participant.address, user);
    assert_eq!(info.participant.role, ParticipantRole::Collector);
    // registered_at is set by ledger timestamp (0 in test environment)
    
    // Stats should be default/zero initially (no submissions yet)
    assert_eq!(info.stats.total_submissions, 0);
    assert_eq!(info.stats.total_weight, 0);
}

#[test]
fn test_get_participant_info_with_stats() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, ScavengerContract);
    let client = ScavengerContractClient::new(&env, &contract_id);
    let user = Address::generate(&env);
    let desc = String::from_str(&env, "Test material");

    // Register participant
    client.register_participant(&user, &ParticipantRole::Collector);

    // Submit material to create stats
    client.submit_material(&WasteType::Plastic, &5000, &user, &desc);

    // Get participant info
    let info = client.get_participant_info(&user).unwrap();
    
    // Verify stats exist
    let stats = info.stats;
    assert_eq!(stats.participant, user);
    assert_eq!(stats.total_submissions, 1);
    assert_eq!(stats.total_weight, 5000);
}

#[test]
fn test_get_participant_info_unregistered_returns_none() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, ScavengerContract);
    let client = ScavengerContractClient::new(&env, &contract_id);
    let unregistered_user = Address::generate(&env);

    // Try to get info for unregistered participant
    let info = client.get_participant_info(&unregistered_user);

    assert!(info.is_none());
}

// ========== Role-Specific Tests ==========

#[test]
fn test_get_participant_info_all_roles() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, ScavengerContract);
    let client = ScavengerContractClient::new(&env, &contract_id);
    
    let recycler = Address::generate(&env);
    let collector = Address::generate(&env);
    let manufacturer = Address::generate(&env);

    // Register participants with different roles
    client.register_participant(&recycler, &ParticipantRole::Recycler);
    client.register_participant(&collector, &ParticipantRole::Collector);
    client.register_participant(&manufacturer, &ParticipantRole::Manufacturer);

    // Get info for each
    let recycler_info = client.get_participant_info(&recycler).unwrap();
    let collector_info = client.get_participant_info(&collector).unwrap();
    let manufacturer_info = client.get_participant_info(&manufacturer).unwrap();

    // Verify roles
    assert_eq!(recycler_info.participant.role, ParticipantRole::Recycler);
    assert_eq!(collector_info.participant.role, ParticipantRole::Collector);
    assert_eq!(manufacturer_info.participant.role, ParticipantRole::Manufacturer);
}

// ========== Statistics Integration Tests ==========

#[test]
fn test_get_participant_info_stats_reflect_submissions() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, ScavengerContract);
    let client = ScavengerContractClient::new(&env, &contract_id);
    let user = Address::generate(&env);
    let desc = String::from_str(&env, "Test");

    // Register participant
    client.register_participant(&user, &ParticipantRole::Collector);

    // Submit multiple materials
    client.submit_material(&WasteType::Paper, &1000, &user, &desc);
    client.submit_material(&WasteType::Plastic, &2000, &user, &desc);
    client.submit_material(&WasteType::Metal, &3000, &user, &desc);

    // Get participant info
    let info = client.get_participant_info(&user).unwrap();
    let stats = info.stats;

    // Verify stats
    assert_eq!(stats.total_submissions, 3);
    assert_eq!(stats.total_weight, 6000);
    assert_eq!(stats.paper_count, 1);
    assert_eq!(stats.plastic_count, 1);
    assert_eq!(stats.metal_count, 1);
}

#[test]
fn test_get_participant_info_stats_reflect_verifications() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, ScavengerContract);
    let client = ScavengerContractClient::new(&env, &contract_id);
    let collector = Address::generate(&env);
    let recycler = Address::generate(&env);
    let desc = String::from_str(&env, "Test");

    // Register participants
    client.register_participant(&collector, &ParticipantRole::Collector);
    client.register_participant(&recycler, &ParticipantRole::Recycler);

    // Submit and verify material
    let material = client.submit_material(&WasteType::Metal, &5000, &collector, &desc);
    client.verify_material(&material.id, &recycler);

    // Get participant info
    let info = client.get_participant_info(&collector).unwrap();
    let stats = info.stats;

    // Verify stats include verification
    assert_eq!(stats.verified_submissions, 1);
    assert!(stats.total_points > 0);
}

#[test]
fn test_get_participant_info_stats_current() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, ScavengerContract);
    let client = ScavengerContractClient::new(&env, &contract_id);
    let user = Address::generate(&env);
    let desc = String::from_str(&env, "Test");

    // Register participant
    client.register_participant(&user, &ParticipantRole::Collector);

    // Submit material
    client.submit_material(&WasteType::Paper, &1000, &user, &desc);

    // Get info - should have 1 submission
    let info1 = client.get_participant_info(&user).unwrap();
    assert_eq!(info1.stats.total_submissions, 1);

    // Submit another material
    client.submit_material(&WasteType::Plastic, &2000, &user, &desc);

    // Get info again - should have 2 submissions
    let info2 = client.get_participant_info(&user).unwrap();
    assert_eq!(info2.stats.total_submissions, 2);
}

// ========== Data Integrity Tests ==========

#[test]
fn test_get_participant_info_preserves_registration_time() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, ScavengerContract);
    let client = ScavengerContractClient::new(&env, &contract_id);
    let user = Address::generate(&env);

    // Register participant
    let participant = client.register_participant(&user, &ParticipantRole::Collector);
    let registration_time = participant.registered_at;

    // Get info
    let info = client.get_participant_info(&user).unwrap();

    // Verify registration time preserved
    assert_eq!(info.participant.registered_at, registration_time);
}

#[test]
fn test_get_participant_info_after_role_update() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, ScavengerContract);
    let client = ScavengerContractClient::new(&env, &contract_id);
    let user = Address::generate(&env);

    // Register as collector
    client.register_participant(&user, &ParticipantRole::Collector);

    // Update role to recycler
    client.update_role(&user, &ParticipantRole::Recycler);

    // Get info
    let info = client.get_participant_info(&user).unwrap();

    // Verify updated role
    assert_eq!(info.participant.role, ParticipantRole::Recycler);
}

// ========== Edge Cases Tests ==========

#[test]
fn test_get_participant_info_multiple_participants() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, ScavengerContract);
    let client = ScavengerContractClient::new(&env, &contract_id);
    
    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);
    let user3 = Address::generate(&env);

    // Register multiple participants
    client.register_participant(&user1, &ParticipantRole::Collector);
    client.register_participant(&user2, &ParticipantRole::Recycler);
    client.register_participant(&user3, &ParticipantRole::Manufacturer);

    // Get info for each
    let info1 = client.get_participant_info(&user1);
    let info2 = client.get_participant_info(&user2);
    let info3 = client.get_participant_info(&user3);

    // All should return Some
    assert!(info1.is_some());
    assert!(info2.is_some());
    assert!(info3.is_some());

    // Verify addresses match
    assert_eq!(info1.unwrap().participant.address, user1);
    assert_eq!(info2.unwrap().participant.address, user2);
    assert_eq!(info3.unwrap().participant.address, user3);
}

#[test]
fn test_get_participant_info_no_side_effects() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, ScavengerContract);
    let client = ScavengerContractClient::new(&env, &contract_id);
    let user = Address::generate(&env);

    // Register participant
    client.register_participant(&user, &ParticipantRole::Collector);

    // Get info multiple times
    let info1 = client.get_participant_info(&user).unwrap();
    let info2 = client.get_participant_info(&user).unwrap();
    let info3 = client.get_participant_info(&user).unwrap();

    // All should be identical
    assert_eq!(info1.participant, info2.participant);
    assert_eq!(info2.participant, info3.participant);
}

// ========== Comprehensive Coverage Tests ==========

#[test]
fn test_get_participant_info_with_all_waste_types() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, ScavengerContract);
    let client = ScavengerContractClient::new(&env, &contract_id);
    let user = Address::generate(&env);
    let desc = String::from_str(&env, "Test");

    // Register participant
    client.register_participant(&user, &ParticipantRole::Collector);

    // Submit all waste types
    client.submit_material(&WasteType::Paper, &1000, &user, &desc);
    client.submit_material(&WasteType::PetPlastic, &2000, &user, &desc);
    client.submit_material(&WasteType::Plastic, &3000, &user, &desc);
    client.submit_material(&WasteType::Metal, &4000, &user, &desc);
    client.submit_material(&WasteType::Glass, &5000, &user, &desc);

    // Get participant info
    let info = client.get_participant_info(&user).unwrap();
    let stats = info.stats;

    // Verify all waste types tracked
    assert_eq!(stats.paper_count, 1);
    assert_eq!(stats.pet_plastic_count, 1);
    assert_eq!(stats.plastic_count, 1);
    assert_eq!(stats.metal_count, 1);
    assert_eq!(stats.glass_count, 1);
    assert_eq!(stats.total_submissions, 5);
}

#[test]
fn test_get_participant_info_consistency_with_get_participant() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, ScavengerContract);
    let client = ScavengerContractClient::new(&env, &contract_id);
    let user = Address::generate(&env);

    // Register participant
    client.register_participant(&user, &ParticipantRole::Recycler);

    // Get via both methods
    let participant = client.get_participant(&user).unwrap();
    let info = client.get_participant_info(&user).unwrap();

    // Participant data should match
    assert_eq!(info.participant.address, participant.address);
    assert_eq!(info.participant.role, participant.role);
    assert_eq!(info.participant.registered_at, participant.registered_at);
}

#[test]
fn test_get_participant_info_consistency_with_get_stats() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, ScavengerContract);
    let client = ScavengerContractClient::new(&env, &contract_id);
    let user = Address::generate(&env);
    let desc = String::from_str(&env, "Test");

    // Register and submit
    client.register_participant(&user, &ParticipantRole::Collector);
    client.submit_material(&WasteType::Paper, &1000, &user, &desc);

    // Get via both methods
    let stats = client.get_stats(&user);
    let info = client.get_participant_info(&user).unwrap();

    // Stats should match
    let info_stats = info.stats;
    let direct_stats = stats.unwrap();
    assert_eq!(info_stats.total_submissions, direct_stats.total_submissions);
    assert_eq!(info_stats.total_weight, direct_stats.total_weight);
    assert_eq!(info_stats.total_points, direct_stats.total_points);
}

#[test]
fn test_get_participant_info_read_only() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, ScavengerContract);
    let client = ScavengerContractClient::new(&env, &contract_id);
    let user = Address::generate(&env);
    let desc = String::from_str(&env, "Test");

    // Register and submit
    client.register_participant(&user, &ParticipantRole::Collector);
    client.submit_material(&WasteType::Metal, &5000, &user, &desc);

    // Get info
    let info_before = client.get_participant_info(&user).unwrap();
    let stats_before = info_before.stats.clone();

    // Get info again
    let info_after = client.get_participant_info(&user).unwrap();
    let stats_after = info_after.stats;

    // Should be identical (read-only operation)
    assert_eq!(stats_before.total_submissions, stats_after.total_submissions);
    assert_eq!(stats_before.total_weight, stats_after.total_weight);
}
