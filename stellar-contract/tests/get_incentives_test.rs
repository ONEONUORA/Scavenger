#![cfg(test)]

use soroban_sdk::{testutils::Address as _, Address, Env};
use stellar_scavngr_contract::{ParticipantRole, ScavengerContract, ScavengerContractClient, WasteType};

// ========== Basic Functionality Tests ==========

#[test]
fn test_get_incentives_returns_active_only() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, ScavengerContract);
    let client = ScavengerContractClient::new(&env, &contract_id);
    
    let manufacturer = Address::generate(&env);
    client.register_participant(&manufacturer, &ParticipantRole::Manufacturer);

    // Create multiple incentives for Plastic
    let incentive1 = client.create_incentive(&manufacturer, &WasteType::Plastic, &50, &10000);
    let incentive2 = client.create_incentive(&manufacturer, &WasteType::Plastic, &30, &5000);
    let incentive3 = client.create_incentive(&manufacturer, &WasteType::Plastic, &70, &15000);

    // Deactivate one incentive
    client.deactivate_incentive(&incentive2.id, &manufacturer);

    // Get active incentives
    let incentives = client.get_incentives(&WasteType::Plastic);

    // Should only return 2 active incentives (not the deactivated one)
    assert_eq!(incentives.len(), 2);
    
    // Verify all returned incentives are active
    for incentive in incentives.iter() {
        assert!(incentive.active);
    }
    
    // Verify the deactivated one is not in the list
    let has_deactivated = incentives.iter().any(|i| i.id == incentive2.id);
    assert!(!has_deactivated);
}

#[test]
fn test_get_incentives_filters_by_waste_type() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, ScavengerContract);
    let client = ScavengerContractClient::new(&env, &contract_id);
    
    let manufacturer = Address::generate(&env);
    client.register_participant(&manufacturer, &ParticipantRole::Manufacturer);

    // Create incentives for different waste types
    client.create_incentive(&manufacturer, &WasteType::Plastic, &50, &10000);
    client.create_incentive(&manufacturer, &WasteType::Metal, &60, &12000);
    client.create_incentive(&manufacturer, &WasteType::Plastic, &40, &8000);
    client.create_incentive(&manufacturer, &WasteType::Glass, &30, &6000);

    // Get incentives for Plastic only
    let plastic_incentives = client.get_incentives(&WasteType::Plastic);

    // Should only return Plastic incentives
    assert_eq!(plastic_incentives.len(), 2);
    
    for incentive in plastic_incentives.iter() {
        assert_eq!(incentive.waste_type, WasteType::Plastic);
    }
}

#[test]
fn test_get_incentives_sorted_by_reward_descending() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, ScavengerContract);
    let client = ScavengerContractClient::new(&env, &contract_id);
    
    let manufacturer = Address::generate(&env);
    client.register_participant(&manufacturer, &ParticipantRole::Manufacturer);

    // Create incentives with different reward amounts (in random order)
    client.create_incentive(&manufacturer, &WasteType::Paper, &30, &5000);
    client.create_incentive(&manufacturer, &WasteType::Paper, &70, &10000);
    client.create_incentive(&manufacturer, &WasteType::Paper, &50, &8000);
    client.create_incentive(&manufacturer, &WasteType::Paper, &90, &15000);
    client.create_incentive(&manufacturer, &WasteType::Paper, &20, &3000);

    // Get incentives
    let incentives = client.get_incentives(&WasteType::Paper);

    // Should be sorted in descending order by reward_points
    assert_eq!(incentives.len(), 5);
    assert_eq!(incentives.get(0).unwrap().reward_points, 90);
    assert_eq!(incentives.get(1).unwrap().reward_points, 70);
    assert_eq!(incentives.get(2).unwrap().reward_points, 50);
    assert_eq!(incentives.get(3).unwrap().reward_points, 30);
    assert_eq!(incentives.get(4).unwrap().reward_points, 20);
}

// ========== Edge Cases Tests ==========

#[test]
fn test_get_incentives_empty_for_no_incentives() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, ScavengerContract);
    let client = ScavengerContractClient::new(&env, &contract_id);

    // Get incentives for a waste type with no incentives
    let incentives = client.get_incentives(&WasteType::Metal);

    // Should return empty vector
    assert_eq!(incentives.len(), 0);
}

#[test]
fn test_get_incentives_empty_when_all_deactivated() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, ScavengerContract);
    let client = ScavengerContractClient::new(&env, &contract_id);
    
    let manufacturer = Address::generate(&env);
    client.register_participant(&manufacturer, &ParticipantRole::Manufacturer);

    // Create incentives
    let incentive1 = client.create_incentive(&manufacturer, &WasteType::Glass, &40, &8000);
    let incentive2 = client.create_incentive(&manufacturer, &WasteType::Glass, &50, &10000);

    // Deactivate all
    client.deactivate_incentive(&incentive1.id, &manufacturer);
    client.deactivate_incentive(&incentive2.id, &manufacturer);

    // Get incentives
    let incentives = client.get_incentives(&WasteType::Glass);

    // Should return empty vector
    assert_eq!(incentives.len(), 0);
}

#[test]
fn test_get_incentives_single_incentive() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, ScavengerContract);
    let client = ScavengerContractClient::new(&env, &contract_id);
    
    let manufacturer = Address::generate(&env);
    client.register_participant(&manufacturer, &ParticipantRole::Manufacturer);

    // Create single incentive
    let created = client.create_incentive(&manufacturer, &WasteType::PetPlastic, &60, &12000);

    // Get incentives
    let incentives = client.get_incentives(&WasteType::PetPlastic);

    // Should return single incentive
    assert_eq!(incentives.len(), 1);
    assert_eq!(incentives.get(0).unwrap().id, created.id);
    assert_eq!(incentives.get(0).unwrap().reward_points, 60);
}

// ========== Sorting Tests ==========

#[test]
fn test_get_incentives_sorting_with_equal_rewards() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, ScavengerContract);
    let client = ScavengerContractClient::new(&env, &contract_id);
    
    let manufacturer = Address::generate(&env);
    client.register_participant(&manufacturer, &ParticipantRole::Manufacturer);

    // Create incentives with some equal reward amounts
    client.create_incentive(&manufacturer, &WasteType::Metal, &50, &10000);
    client.create_incentive(&manufacturer, &WasteType::Metal, &50, &8000);
    client.create_incentive(&manufacturer, &WasteType::Metal, &70, &12000);
    client.create_incentive(&manufacturer, &WasteType::Metal, &30, &6000);

    // Get incentives
    let incentives = client.get_incentives(&WasteType::Metal);

    // Should be sorted correctly
    assert_eq!(incentives.len(), 4);
    assert_eq!(incentives.get(0).unwrap().reward_points, 70);
    assert_eq!(incentives.get(1).unwrap().reward_points, 50);
    assert_eq!(incentives.get(2).unwrap().reward_points, 50);
    assert_eq!(incentives.get(3).unwrap().reward_points, 30);
}

#[test]
fn test_get_incentives_already_sorted() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, ScavengerContract);
    let client = ScavengerContractClient::new(&env, &contract_id);
    
    let manufacturer = Address::generate(&env);
    client.register_participant(&manufacturer, &ParticipantRole::Manufacturer);

    // Create incentives already in descending order
    client.create_incentive(&manufacturer, &WasteType::Paper, &90, &15000);
    client.create_incentive(&manufacturer, &WasteType::Paper, &70, &12000);
    client.create_incentive(&manufacturer, &WasteType::Paper, &50, &10000);

    // Get incentives
    let incentives = client.get_incentives(&WasteType::Paper);

    // Should maintain correct order
    assert_eq!(incentives.len(), 3);
    assert_eq!(incentives.get(0).unwrap().reward_points, 90);
    assert_eq!(incentives.get(1).unwrap().reward_points, 70);
    assert_eq!(incentives.get(2).unwrap().reward_points, 50);
}

// ========== Multiple Waste Types Tests ==========

#[test]
fn test_get_incentives_independent_per_waste_type() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, ScavengerContract);
    let client = ScavengerContractClient::new(&env, &contract_id);
    
    let manufacturer = Address::generate(&env);
    client.register_participant(&manufacturer, &ParticipantRole::Manufacturer);

    // Create incentives for different waste types
    client.create_incentive(&manufacturer, &WasteType::Plastic, &50, &10000);
    client.create_incentive(&manufacturer, &WasteType::Metal, &60, &12000);
    client.create_incentive(&manufacturer, &WasteType::Plastic, &40, &8000);

    // Get incentives for each type
    let plastic = client.get_incentives(&WasteType::Plastic);
    let metal = client.get_incentives(&WasteType::Metal);
    let glass = client.get_incentives(&WasteType::Glass);

    // Each should be independent
    assert_eq!(plastic.len(), 2);
    assert_eq!(metal.len(), 1);
    assert_eq!(glass.len(), 0);
}

// ========== Data Integrity Tests ==========

#[test]
fn test_get_incentives_returns_complete_data() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, ScavengerContract);
    let client = ScavengerContractClient::new(&env, &contract_id);
    
    let manufacturer = Address::generate(&env);
    client.register_participant(&manufacturer, &ParticipantRole::Manufacturer);

    // Create incentive
    let created = client.create_incentive(&manufacturer, &WasteType::Plastic, &50, &10000);

    // Get incentives
    let incentives = client.get_incentives(&WasteType::Plastic);
    let retrieved = incentives.get(0).unwrap();

    // Verify all fields are correct
    assert_eq!(retrieved.id, created.id);
    assert_eq!(retrieved.rewarder, manufacturer);
    assert_eq!(retrieved.waste_type, WasteType::Plastic);
    assert_eq!(retrieved.reward_points, 50);
    assert_eq!(retrieved.total_budget, 10000);
    assert_eq!(retrieved.remaining_budget, 10000);
    assert!(retrieved.active);
}

#[test]
fn test_get_incentives_reflects_budget_changes() {
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

    // Create incentive
    let incentive = client.create_incentive(&manufacturer, &WasteType::Metal, &100, &1000);

    // Submit and verify material to claim reward
    let desc = soroban_sdk::String::from_str(&env, "Test");
    let material = client.submit_material(&WasteType::Metal, &5000, &collector, &desc);
    client.verify_material(&material.id, &recycler);
    
    // Claim reward (5kg * 100 = 500 points)
    client.claim_incentive_reward(&incentive.id, &material.id, &collector);

    // Get incentives
    let incentives = client.get_incentives(&WasteType::Metal);
    let retrieved = incentives.get(0).unwrap();

    // Should reflect reduced budget
    assert_eq!(retrieved.remaining_budget, 500);
    assert!(retrieved.active);
}

#[test]
fn test_get_incentives_excludes_auto_deactivated() {
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

    // Create incentive with small budget
    let incentive = client.create_incentive(&manufacturer, &WasteType::Paper, &100, &500);

    // Submit and verify material to exhaust budget
    let desc = soroban_sdk::String::from_str(&env, "Test");
    let material = client.submit_material(&WasteType::Paper, &5000, &collector, &desc);
    client.verify_material(&material.id, &recycler);
    
    // Claim reward (5kg * 100 = 500 points, exhausts budget)
    client.claim_incentive_reward(&incentive.id, &material.id, &collector);

    // Get incentives
    let incentives = client.get_incentives(&WasteType::Paper);

    // Should not include auto-deactivated incentive
    assert_eq!(incentives.len(), 0);
}

// ========== Comprehensive Coverage Tests ==========

#[test]
fn test_get_incentives_all_waste_types() {
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

    // Get incentives for each type
    assert_eq!(client.get_incentives(&WasteType::Paper).len(), 1);
    assert_eq!(client.get_incentives(&WasteType::PetPlastic).len(), 1);
    assert_eq!(client.get_incentives(&WasteType::Plastic).len(), 1);
    assert_eq!(client.get_incentives(&WasteType::Metal).len(), 1);
    assert_eq!(client.get_incentives(&WasteType::Glass).len(), 1);
}

#[test]
fn test_get_incentives_multiple_manufacturers() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, ScavengerContract);
    let client = ScavengerContractClient::new(&env, &contract_id);
    
    let manufacturer1 = Address::generate(&env);
    let manufacturer2 = Address::generate(&env);
    
    client.register_participant(&manufacturer1, &ParticipantRole::Manufacturer);
    client.register_participant(&manufacturer2, &ParticipantRole::Manufacturer);

    // Create incentives from different manufacturers
    client.create_incentive(&manufacturer1, &WasteType::Plastic, &50, &10000);
    client.create_incentive(&manufacturer2, &WasteType::Plastic, &60, &12000);
    client.create_incentive(&manufacturer1, &WasteType::Plastic, &40, &8000);

    // Get incentives
    let incentives = client.get_incentives(&WasteType::Plastic);

    // Should include all active incentives regardless of manufacturer
    assert_eq!(incentives.len(), 3);
    
    // Should be sorted by reward
    assert_eq!(incentives.get(0).unwrap().reward_points, 60);
    assert_eq!(incentives.get(1).unwrap().reward_points, 50);
    assert_eq!(incentives.get(2).unwrap().reward_points, 40);
}

#[test]
fn test_get_incentives_no_side_effects() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, ScavengerContract);
    let client = ScavengerContractClient::new(&env, &contract_id);
    
    let manufacturer = Address::generate(&env);
    client.register_participant(&manufacturer, &ParticipantRole::Manufacturer);

    // Create incentives
    client.create_incentive(&manufacturer, &WasteType::Metal, &50, &10000);
    client.create_incentive(&manufacturer, &WasteType::Metal, &70, &12000);

    // Get incentives multiple times
    let incentives1 = client.get_incentives(&WasteType::Metal);
    let incentives2 = client.get_incentives(&WasteType::Metal);
    let incentives3 = client.get_incentives(&WasteType::Metal);

    // Should be identical (read-only operation)
    assert_eq!(incentives1.len(), incentives2.len());
    assert_eq!(incentives2.len(), incentives3.len());
    
    for i in 0..incentives1.len() {
        assert_eq!(incentives1.get(i).unwrap().id, incentives2.get(i).unwrap().id);
        assert_eq!(incentives2.get(i).unwrap().id, incentives3.get(i).unwrap().id);
    }
}

#[test]
fn test_get_incentives_large_list() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register_contract(None, ScavengerContract);
    let client = ScavengerContractClient::new(&env, &contract_id);
    
    let manufacturer = Address::generate(&env);
    client.register_participant(&manufacturer, &ParticipantRole::Manufacturer);

    // Create many incentives
    for i in 1..=10 {
        client.create_incentive(&manufacturer, &WasteType::Glass, &(i * 10), &(i * 1000));
    }

    // Get incentives
    let incentives = client.get_incentives(&WasteType::Glass);

    // Should return all and be sorted correctly
    assert_eq!(incentives.len(), 10);
    
    // Verify descending order
    for i in 0..9 {
        let curr = incentives.get(i).unwrap();
        let next = incentives.get(i + 1).unwrap();
        assert!(curr.reward_points >= next.reward_points);
    }
}
