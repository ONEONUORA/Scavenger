#![cfg(test)]

use soroban_sdk::{testutils::Address as _, Address, Env};
use stellar_scavngr_contract::{ParticipantRole, ScavengerContract, ScavengerContractClient, WasteType};

// ========== Basic Functionality Tests ==========

#[test]
fn test_get_active_incentive_returns_highest_reward() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, ScavengerContract);
    let client = ScavengerContractClient::new(&env, &contract_id);
    
    let manufacturer = Address::generate(&env);
    client.register_participant(&manufacturer, &ParticipantRole::Manufacturer);

    // Create multiple incentives for Plastic with different rewards
    client.create_incentive(&manufacturer, &WasteType::Plastic, &50, &10000);
    client.create_incentive(&manufacturer, &WasteType::Plastic, &30, &5000);
    client.create_incentive(&manufacturer, &WasteType::Plastic, &70, &15000); // Highest

    // Get active incentive
    let result = client.get_active_incentive_for_manufacturer(&manufacturer, &WasteType::Plastic);

    // Should return the one with highest reward (70)
    assert!(result.is_some());
    let incentive = result.unwrap();
    assert_eq!(incentive.reward_points, 70);
    assert_eq!(incentive.waste_type, WasteType::Plastic);
    assert_eq!(incentive.rewarder, manufacturer);
    assert!(incentive.active);
}

#[test]
fn test_get_active_incentive_filters_by_waste_type() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, ScavengerContract);
    let client = ScavengerContractClient::new(&env, &contract_id);
    
    let manufacturer = Address::generate(&env);
    client.register_participant(&manufacturer, &ParticipantRole::Manufacturer);

    // Create incentives for different waste types
    client.create_incentive(&manufacturer, &WasteType::Plastic, &50, &10000);
    client.create_incentive(&manufacturer, &WasteType::Metal, &80, &12000); // Higher but wrong type
    client.create_incentive(&manufacturer, &WasteType::Plastic, &60, &8000); // Highest for Plastic

    // Get active incentive for Plastic
    let result = client.get_active_incentive_for_manufacturer(&manufacturer, &WasteType::Plastic);

    // Should return the highest Plastic incentive (60), not Metal (80)
    assert!(result.is_some());
    let incentive = result.unwrap();
    assert_eq!(incentive.reward_points, 60);
    assert_eq!(incentive.waste_type, WasteType::Plastic);
}

#[test]
fn test_get_active_incentive_filters_by_manufacturer() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, ScavengerContract);
    let client = ScavengerContractClient::new(&env, &contract_id);
    
    let manufacturer1 = Address::generate(&env);
    let manufacturer2 = Address::generate(&env);
    
    client.register_participant(&manufacturer1, &ParticipantRole::Manufacturer);
    client.register_participant(&manufacturer2, &ParticipantRole::Manufacturer);

    // Create incentives from different manufacturers
    client.create_incentive(&manufacturer1, &WasteType::Paper, &40, &8000);
    client.create_incentive(&manufacturer2, &WasteType::Paper, &90, &15000); // Higher but wrong manufacturer
    client.create_incentive(&manufacturer1, &WasteType::Paper, &50, &10000); // Highest for manufacturer1

    // Get active incentive for manufacturer1
    let result = client.get_active_incentive_for_manufacturer(&manufacturer1, &WasteType::Paper);

    // Should return manufacturer1's highest (50), not manufacturer2's (90)
    assert!(result.is_some());
    let incentive = result.unwrap();
    assert_eq!(incentive.reward_points, 50);
    assert_eq!(incentive.rewarder, manufacturer1);
}

#[test]
fn test_get_active_incentive_excludes_inactive() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, ScavengerContract);
    let client = ScavengerContractClient::new(&env, &contract_id);
    
    let manufacturer = Address::generate(&env);
    client.register_participant(&manufacturer, &ParticipantRole::Manufacturer);

    // Create incentives
    let incentive1 = client.create_incentive(&manufacturer, &WasteType::Metal, &80, &10000); // Highest but will be deactivated
    client.create_incentive(&manufacturer, &WasteType::Metal, &50, &8000); // Active

    // Deactivate the highest one
    client.deactivate_incentive(&incentive1.id, &manufacturer);

    // Get active incentive
    let result = client.get_active_incentive_for_manufacturer(&manufacturer, &WasteType::Metal);

    // Should return the active one (50), not the deactivated one (80)
    assert!(result.is_some());
    let incentive = result.unwrap();
    assert_eq!(incentive.reward_points, 50);
    assert!(incentive.active);
}

// ========== Edge Cases Tests ==========

#[test]
fn test_get_active_incentive_returns_none_when_no_incentives() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, ScavengerContract);
    let client = ScavengerContractClient::new(&env, &contract_id);
    
    let manufacturer = Address::generate(&env);
    client.register_participant(&manufacturer, &ParticipantRole::Manufacturer);

    // Get active incentive without creating any
    let result = client.get_active_incentive_for_manufacturer(&manufacturer, &WasteType::Glass);

    // Should return None
    assert!(result.is_none());
}

#[test]
fn test_get_active_incentive_returns_none_when_all_inactive() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, ScavengerContract);
    let client = ScavengerContractClient::new(&env, &contract_id);
    
    let manufacturer = Address::generate(&env);
    client.register_participant(&manufacturer, &ParticipantRole::Manufacturer);

    // Create incentives and deactivate all
    let incentive1 = client.create_incentive(&manufacturer, &WasteType::PetPlastic, &50, &10000);
    let incentive2 = client.create_incentive(&manufacturer, &WasteType::PetPlastic, &60, &12000);
    
    client.deactivate_incentive(&incentive1.id, &manufacturer);
    client.deactivate_incentive(&incentive2.id, &manufacturer);

    // Get active incentive
    let result = client.get_active_incentive_for_manufacturer(&manufacturer, &WasteType::PetPlastic);

    // Should return None
    assert!(result.is_none());
}

#[test]
fn test_get_active_incentive_returns_none_for_wrong_waste_type() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, ScavengerContract);
    let client = ScavengerContractClient::new(&env, &contract_id);
    
    let manufacturer = Address::generate(&env);
    client.register_participant(&manufacturer, &ParticipantRole::Manufacturer);

    // Create incentives for Plastic only
    client.create_incentive(&manufacturer, &WasteType::Plastic, &50, &10000);
    client.create_incentive(&manufacturer, &WasteType::Plastic, &60, &12000);

    // Query for Metal
    let result = client.get_active_incentive_for_manufacturer(&manufacturer, &WasteType::Metal);

    // Should return None
    assert!(result.is_none());
}

#[test]
fn test_get_active_incentive_single_incentive() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, ScavengerContract);
    let client = ScavengerContractClient::new(&env, &contract_id);
    
    let manufacturer = Address::generate(&env);
    client.register_participant(&manufacturer, &ParticipantRole::Manufacturer);

    // Create single incentive
    let created = client.create_incentive(&manufacturer, &WasteType::Glass, &45, &9000);

    // Get active incentive
    let result = client.get_active_incentive_for_manufacturer(&manufacturer, &WasteType::Glass);

    // Should return the single incentive
    assert!(result.is_some());
    let incentive = result.unwrap();
    assert_eq!(incentive.id, created.id);
    assert_eq!(incentive.reward_points, 45);
}

// ========== Equal Rewards Tests ==========

#[test]
fn test_get_active_incentive_with_equal_rewards() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, ScavengerContract);
    let client = ScavengerContractClient::new(&env, &contract_id);
    
    let manufacturer = Address::generate(&env);
    client.register_participant(&manufacturer, &ParticipantRole::Manufacturer);

    // Create incentives with equal rewards
    client.create_incentive(&manufacturer, &WasteType::Paper, &50, &10000);
    client.create_incentive(&manufacturer, &WasteType::Paper, &50, &8000);
    client.create_incentive(&manufacturer, &WasteType::Paper, &50, &12000);

    // Get active incentive
    let result = client.get_active_incentive_for_manufacturer(&manufacturer, &WasteType::Paper);

    // Should return one of them (any is valid since they're equal)
    assert!(result.is_some());
    let incentive = result.unwrap();
    assert_eq!(incentive.reward_points, 50);
    assert_eq!(incentive.waste_type, WasteType::Paper);
}

// ========== Budget Exhaustion Tests ==========

#[test]
fn test_get_active_incentive_excludes_auto_deactivated() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, ScavengerContract);
    let client = ScavengerContractClient::new(&env, &contract_id);
    
    let manufacturer = Address::generate(&env);
    let collector = Address::generate(&env);
    let recycler = Address::generate(&env);
    
    client.register_participant(&manufacturer, &ParticipantRole::Manufacturer);
    client.register_participant(&collector, &ParticipantRole::Collector);
    client.register_participant(&recycler, &ParticipantRole::Recycler);

    // Create incentives
    let incentive1 = client.create_incentive(&manufacturer, &WasteType::Metal, &100, &500); // Will be exhausted
    client.create_incentive(&manufacturer, &WasteType::Metal, &80, &10000); // Active

    // Submit and verify material to exhaust first incentive
    let desc = soroban_sdk::String::from_str(&env, "Test");
    let material = client.submit_material(&WasteType::Metal, &5000, &collector, &desc);
    client.verify_material(&material.id, &recycler);
    
    // Claim reward (5kg * 100 = 500 points, exhausts budget)
    client.claim_incentive_reward(&incentive1.id, &material.id, &collector);

    // Get active incentive
    let result = client.get_active_incentive_for_manufacturer(&manufacturer, &WasteType::Metal);

    // Should return the still-active one (80), not the exhausted one (100)
    assert!(result.is_some());
    let incentive = result.unwrap();
    assert_eq!(incentive.reward_points, 80);
    assert!(incentive.active);
}

// ========== All Waste Types Tests ==========

#[test]
fn test_get_active_incentive_all_waste_types() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, ScavengerContract);
    let client = ScavengerContractClient::new(&env, &contract_id);
    
    let manufacturer = Address::generate(&env);
    client.register_participant(&manufacturer, &ParticipantRole::Manufacturer);

    // Create incentives for all waste types
    client.create_incentive(&manufacturer, &WasteType::Paper, &30, &5000);
    client.create_incentive(&manufacturer, &WasteType::PetPlastic, &50, &8000);
    client.create_incentive(&manufacturer, &WasteType::Plastic, &40, &7000);
    client.create_incentive(&manufacturer, &WasteType::Metal, &70, &12000);
    client.create_incentive(&manufacturer, &WasteType::Glass, &35, &6000);

    // Get active incentive for each type
    let paper = client.get_active_incentive_for_manufacturer(&manufacturer, &WasteType::Paper);
    let pet = client.get_active_incentive_for_manufacturer(&manufacturer, &WasteType::PetPlastic);
    let plastic = client.get_active_incentive_for_manufacturer(&manufacturer, &WasteType::Plastic);
    let metal = client.get_active_incentive_for_manufacturer(&manufacturer, &WasteType::Metal);
    let glass = client.get_active_incentive_for_manufacturer(&manufacturer, &WasteType::Glass);

    // All should return the correct incentive
    assert_eq!(paper.unwrap().reward_points, 30);
    assert_eq!(pet.unwrap().reward_points, 50);
    assert_eq!(plastic.unwrap().reward_points, 40);
    assert_eq!(metal.unwrap().reward_points, 70);
    assert_eq!(glass.unwrap().reward_points, 35);
}

// ========== Data Integrity Tests ==========

#[test]
fn test_get_active_incentive_returns_complete_data() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, ScavengerContract);
    let client = ScavengerContractClient::new(&env, &contract_id);
    
    let manufacturer = Address::generate(&env);
    client.register_participant(&manufacturer, &ParticipantRole::Manufacturer);

    // Create incentive
    let created = client.create_incentive(&manufacturer, &WasteType::Plastic, &55, &11000);

    // Get active incentive
    let result = client.get_active_incentive_for_manufacturer(&manufacturer, &WasteType::Plastic);
    let retrieved = result.unwrap();

    // Verify all fields are correct
    assert_eq!(retrieved.id, created.id);
    assert_eq!(retrieved.rewarder, manufacturer);
    assert_eq!(retrieved.waste_type, WasteType::Plastic);
    assert_eq!(retrieved.reward_points, 55);
    assert_eq!(retrieved.total_budget, 11000);
    assert_eq!(retrieved.remaining_budget, 11000);
    assert!(retrieved.active);
}

#[test]
fn test_get_active_incentive_no_side_effects() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, ScavengerContract);
    let client = ScavengerContractClient::new(&env, &contract_id);
    
    let manufacturer = Address::generate(&env);
    client.register_participant(&manufacturer, &ParticipantRole::Manufacturer);

    // Create incentives
    client.create_incentive(&manufacturer, &WasteType::Metal, &50, &10000);
    client.create_incentive(&manufacturer, &WasteType::Metal, &70, &12000);

    // Get active incentive multiple times
    let result1 = client.get_active_incentive_for_manufacturer(&manufacturer, &WasteType::Metal);
    let result2 = client.get_active_incentive_for_manufacturer(&manufacturer, &WasteType::Metal);
    let result3 = client.get_active_incentive_for_manufacturer(&manufacturer, &WasteType::Metal);

    // Should be identical (read-only operation)
    assert!(result1.is_some());
    assert!(result2.is_some());
    assert!(result3.is_some());
    
    let incentive1 = result1.unwrap();
    let incentive2 = result2.unwrap();
    let incentive3 = result3.unwrap();
    
    assert_eq!(incentive1.id, incentive2.id);
    assert_eq!(incentive2.id, incentive3.id);
    assert_eq!(incentive1.reward_points, 70);
}

// ========== Complex Scenarios Tests ==========

#[test]
fn test_get_active_incentive_mixed_active_inactive() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, ScavengerContract);
    let client = ScavengerContractClient::new(&env, &contract_id);
    
    let manufacturer = Address::generate(&env);
    client.register_participant(&manufacturer, &ParticipantRole::Manufacturer);

    // Create multiple incentives
    let incentive1 = client.create_incentive(&manufacturer, &WasteType::Glass, &90, &15000); // Highest but will deactivate
    client.create_incentive(&manufacturer, &WasteType::Glass, &60, &10000); // Active
    let incentive3 = client.create_incentive(&manufacturer, &WasteType::Glass, &75, &12000); // Second highest but will deactivate
    client.create_incentive(&manufacturer, &WasteType::Glass, &50, &8000); // Active

    // Deactivate some
    client.deactivate_incentive(&incentive1.id, &manufacturer);
    client.deactivate_incentive(&incentive3.id, &manufacturer);

    // Get active incentive
    let result = client.get_active_incentive_for_manufacturer(&manufacturer, &WasteType::Glass);

    // Should return highest active (60), not deactivated ones (90, 75)
    assert!(result.is_some());
    let incentive = result.unwrap();
    assert_eq!(incentive.reward_points, 60);
    assert!(incentive.active);
}

#[test]
fn test_get_active_incentive_multiple_manufacturers_isolation() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, ScavengerContract);
    let client = ScavengerContractClient::new(&env, &contract_id);
    
    let manufacturer1 = Address::generate(&env);
    let manufacturer2 = Address::generate(&env);
    let manufacturer3 = Address::generate(&env);
    
    client.register_participant(&manufacturer1, &ParticipantRole::Manufacturer);
    client.register_participant(&manufacturer2, &ParticipantRole::Manufacturer);
    client.register_participant(&manufacturer3, &ParticipantRole::Manufacturer);

    // Create incentives from different manufacturers for same waste type
    client.create_incentive(&manufacturer1, &WasteType::Plastic, &40, &8000);
    client.create_incentive(&manufacturer2, &WasteType::Plastic, &90, &15000);
    client.create_incentive(&manufacturer3, &WasteType::Plastic, &60, &12000);
    client.create_incentive(&manufacturer1, &WasteType::Plastic, &50, &10000);

    // Get active incentive for each manufacturer
    let result1 = client.get_active_incentive_for_manufacturer(&manufacturer1, &WasteType::Plastic);
    let result2 = client.get_active_incentive_for_manufacturer(&manufacturer2, &WasteType::Plastic);
    let result3 = client.get_active_incentive_for_manufacturer(&manufacturer3, &WasteType::Plastic);

    // Each should return their own highest
    assert_eq!(result1.unwrap().reward_points, 50);
    assert_eq!(result1.unwrap().rewarder, manufacturer1);
    
    assert_eq!(result2.unwrap().reward_points, 90);
    assert_eq!(result2.unwrap().rewarder, manufacturer2);
    
    assert_eq!(result3.unwrap().reward_points, 60);
    assert_eq!(result3.unwrap().rewarder, manufacturer3);
}

#[test]
fn test_get_active_incentive_large_number_of_incentives() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, ScavengerContract);
    let client = ScavengerContractClient::new(&env, &contract_id);
    
    let manufacturer = Address::generate(&env);
    client.register_participant(&manufacturer, &ParticipantRole::Manufacturer);

    // Create many incentives with varying rewards
    for i in 1..=10 {
        client.create_incentive(&manufacturer, &WasteType::Paper, &(i * 10), &(i * 1000));
    }

    // Get active incentive
    let result = client.get_active_incentive_for_manufacturer(&manufacturer, &WasteType::Paper);

    // Should return the highest (100)
    assert!(result.is_some());
    let incentive = result.unwrap();
    assert_eq!(incentive.reward_points, 100);
}
