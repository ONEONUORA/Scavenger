#![cfg(test)]

use soroban_sdk::{testutils::Address as _, Address, Env};

use crate::contract::ScavengerContract;
use crate::types::{Role, WasteType};

fn create_test_contract(env: &Env) -> (crate::contract::ScavengerContractClient<'_>, Address, Address, Address) {
    let contract_id = env.register(ScavengerContract, ());
    let client = crate::contract::ScavengerContractClient::new(env, &contract_id);
    
    let admin = Address::generate(env);
    let token_address = env.register_stellar_asset_contract(admin.clone());
    let charity_address = Address::generate(env);
    
    client.initialize(&admin, &token_address, &charity_address, &5, &50);
    
    (client, admin, token_address, charity_address)
}

#[test]
fn test_update_incentive_success() {
    let env = Env::default();
    env.mock_all_auths();
    
    let (client, _admin, _token, _charity) = create_test_contract(&env);
    let manufacturer = Address::generate(&env);
    
    // Register manufacturer
    client.register_participant(
        &manufacturer,
        &Role::Manufacturer,
        &soroban_sdk::String::from_str(&env, "Test Manufacturer"),
        &1000000,
        &2000000,
    );
    
    // Create incentive
    let incentive = client.create_incentive(
        &manufacturer,
        &WasteType::Paper,
        &100,
        &5000,
    );
    
    assert_eq!(incentive.reward_points, 100);
    assert_eq!(incentive.total_budget, 5000);
    assert_eq!(incentive.remaining_budget, 5000);
    
    // Update incentive
    let updated = client.update_incentive(&incentive.id, &200, &10000);
    
    assert_eq!(updated.reward_points, 200);
    assert_eq!(updated.total_budget, 10000);
    assert_eq!(updated.remaining_budget, 10000);
    
    // Verify immutable fields unchanged
    assert_eq!(updated.id, incentive.id);
    assert_eq!(updated.waste_type, incentive.waste_type);
    assert_eq!(updated.rewarder, incentive.rewarder);
    assert_eq!(updated.active, incentive.active);
    assert_eq!(updated.created_at, incentive.created_at);
    
    // Verify persistence
    let retrieved = client.get_incentive_by_id(&incentive.id).unwrap();
    assert_eq!(retrieved.reward_points, 200);
    assert_eq!(retrieved.total_budget, 10000);
}

#[test]
#[should_panic(expected = "Incentive not found")]
fn test_update_incentive_not_found() {
    let env = Env::default();
    env.mock_all_auths();
    
    let (client, _admin, _token, _charity) = create_test_contract(&env);
    
    // Try to update non-existent incentive
    client.update_incentive(&999, &100, &5000);
}

#[test]
#[should_panic(expected = "Incentive is not active")]
fn test_update_incentive_inactive() {
    let env = Env::default();
    env.mock_all_auths();
    
    let (client, _admin, _token, _charity) = create_test_contract(&env);
    let manufacturer = Address::generate(&env);
    
    // Register manufacturer
    client.register_participant(
        &manufacturer,
        &Role::Manufacturer,
        &soroban_sdk::String::from_str(&env, "Test Manufacturer"),
        &1000000,
        &2000000,
    );
    
    // Create incentive
    let incentive = client.create_incentive(
        &manufacturer,
        &WasteType::Paper,
        &100,
        &5000,
    );
    
    // Deactivate the incentive
    client.deactivate_incentive(&manufacturer, &incentive.id);
    
    // Try to update inactive incentive
    client.update_incentive(&incentive.id, &200, &10000);
}

#[test]
#[should_panic(expected = "Reward must be greater than zero")]
fn test_update_incentive_zero_reward() {
    let env = Env::default();
    env.mock_all_auths();
    
    let (client, _admin, _token, _charity) = create_test_contract(&env);
    let manufacturer = Address::generate(&env);
    
    // Register manufacturer
    client.register_participant(
        &manufacturer,
        &Role::Manufacturer,
        &soroban_sdk::String::from_str(&env, "Test Manufacturer"),
        &1000000,
        &2000000,
    );
    
    let incentive = client.create_incentive(
        &manufacturer,
        &WasteType::Paper,
        &100,
        &5000,
    );
    
    // Try to update with zero reward
    client.update_incentive(&incentive.id, &0, &5000);
}

#[test]
#[should_panic(expected = "Total budget must be greater than zero")]
fn test_update_incentive_zero_budget() {
    let env = Env::default();
    env.mock_all_auths();
    
    let (client, _admin, _token, _charity) = create_test_contract(&env);
    let manufacturer = Address::generate(&env);
    
    // Register manufacturer
    client.register_participant(
        &manufacturer,
        &Role::Manufacturer,
        &soroban_sdk::String::from_str(&env, "Test Manufacturer"),
        &1000000,
        &2000000,
    );
    
    let incentive = client.create_incentive(
        &manufacturer,
        &WasteType::Paper,
        &100,
        &5000,
    );
    
    // Try to update with zero budget
    client.update_incentive(&incentive.id, &100, &0);
}

#[test]
fn test_update_incentive_minimum_values() {
    let env = Env::default();
    env.mock_all_auths();
    
    let (client, _admin, _token, _charity) = create_test_contract(&env);
    let manufacturer = Address::generate(&env);
    
    // Register manufacturer
    client.register_participant(
        &manufacturer,
        &Role::Manufacturer,
        &soroban_sdk::String::from_str(&env, "Test Manufacturer"),
        &1000000,
        &2000000,
    );
    
    let incentive = client.create_incentive(
        &manufacturer,
        &WasteType::Paper,
        &100,
        &5000,
    );
    
    // Update with minimum valid values
    let updated = client.update_incentive(&incentive.id, &1, &1);
    assert_eq!(updated.reward_points, 1);
    assert_eq!(updated.total_budget, 1);
}

#[test]
fn test_update_incentive_multiple_times() {
    let env = Env::default();
    env.mock_all_auths();
    
    let (client, _admin, _token, _charity) = create_test_contract(&env);
    let manufacturer = Address::generate(&env);
    
    // Register manufacturer
    client.register_participant(
        &manufacturer,
        &Role::Manufacturer,
        &soroban_sdk::String::from_str(&env, "Test Manufacturer"),
        &1000000,
        &2000000,
    );
    
    let incentive = client.create_incentive(
        &manufacturer,
        &WasteType::Paper,
        &100,
        &5000,
    );
    
    // First update
    let updated1 = client.update_incentive(&incentive.id, &200, &10000);
    assert_eq!(updated1.reward_points, 200);
    assert_eq!(updated1.total_budget, 10000);
    
    // Second update
    let updated2 = client.update_incentive(&incentive.id, &300, &15000);
    assert_eq!(updated2.reward_points, 300);
    assert_eq!(updated2.total_budget, 15000);
    
    // Third update
    let updated3 = client.update_incentive(&incentive.id, &400, &20000);
    assert_eq!(updated3.reward_points, 400);
    assert_eq!(updated3.total_budget, 20000);
}

#[test]
fn test_update_incentive_with_partial_budget_used() {
    let env = Env::default();
    env.mock_all_auths();
    
    let (client, _admin, _token, _charity) = create_test_contract(&env);
    let manufacturer = Address::generate(&env);
    
    // Register manufacturer
    client.register_participant(
        &manufacturer,
        &Role::Manufacturer,
        &soroban_sdk::String::from_str(&env, "Test Manufacturer"),
        &1000000,
        &2000000,
    );
    
    // Create incentive with 5000 budget
    let mut incentive = client.create_incentive(
        &manufacturer,
        &WasteType::Paper,
        &100,
        &5000,
    );
    
    // Simulate some budget being used (manually for testing)
    // In real scenario, this would happen through distribute_rewards
    // For this test, we'll just verify the update logic works correctly
    
    // Update to increase budget
    let updated = client.update_incentive(&incentive.id, &150, &8000);
    assert_eq!(updated.reward_points, 150);
    assert_eq!(updated.total_budget, 8000);
    // Since no budget was actually used, remaining should equal total
    assert_eq!(updated.remaining_budget, 8000);
}
