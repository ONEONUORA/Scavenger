#![cfg(test)]

use soroban_sdk::{testutils::Address as _, Address, Env};

use crate::{ScavengerContract, ScavengerContractClient};

fn create_test_contract(env: &Env) -> (ScavengerContractClient, Address, Address) {
    let contract_id = env.register(ScavengerContract, ());
    let client = ScavengerContractClient::new(env, &contract_id);

    let admin = Address::generate(env);
    let token_address = Address::generate(env);

    (client, admin, token_address)
}

#[test]
fn test_initialization() {
    let env = Env::default();
    let (client, admin, token_address) = create_test_contract(&env);

    env.mock_all_auths();

    // Initialize contract
    client.__constructor(&admin, &token_address);

    // Verify initialization
    assert_eq!(client.get_admin(), admin);
    assert_eq!(client.get_token_address(), token_address);
    assert_eq!(client.get_collector_percentage(), 5); // Default 5%
    assert_eq!(client.get_owner_percentage(), 50); // Default 50%
    assert_eq!(client.get_counter(), 1); // Counter initialized to 1
    assert_eq!(client.get_total_earned(), 0);
    assert_eq!(client.is_initialized(), true);
}

#[test]
#[should_panic(expected = "Contract already initialized")]
fn test_cannot_reinitialize() {
    let env = Env::default();
    let (client, admin, token_address) = create_test_contract(&env);

    env.mock_all_auths();

    // Initialize contract
    client.__constructor(&admin, &token_address);

    // Try to initialize again - should panic
    client.__constructor(&admin, &token_address);
}

#[test]
fn test_default_percentages() {
    let env = Env::default();
    let (client, admin, token_address) = create_test_contract(&env);

    env.mock_all_auths();

    client.__constructor(&admin, &token_address);

    // Verify default percentages
    assert_eq!(client.get_collector_percentage(), 5);
    assert_eq!(client.get_owner_percentage(), 50);
}

#[test]
fn test_counter_initialization() {
    let env = Env::default();
    let (client, admin, token_address) = create_test_contract(&env);

    env.mock_all_auths();

    client.__constructor(&admin, &token_address);

    // Verify counter starts at 1
    assert_eq!(client.get_counter(), 1);
}

#[test]
fn test_all_storage_set_correctly() {
    let env = Env::default();
    let (client, admin, token_address) = create_test_contract(&env);

    env.mock_all_auths();

    client.__constructor(&admin, &token_address);

    // Verify all storage is set
    assert_eq!(client.get_admin(), admin);
    assert_eq!(client.get_token_address(), token_address);
    assert_eq!(client.get_collector_percentage(), 5);
    assert_eq!(client.get_owner_percentage(), 50);
    assert_eq!(client.get_counter(), 1);
    assert_eq!(client.get_total_earned(), 0);
    assert_eq!(client.is_initialized(), true);
}

#[test]
fn test_charity_address_optional() {
    let env = Env::default();
    let (client, admin, token_address) = create_test_contract(&env);

    env.mock_all_auths();

    client.__constructor(&admin, &token_address);

    // Charity address should be None initially
    assert_eq!(client.get_charity_address(), None);

    // Admin can set it later
    let charity_address = Address::generate(&env);
    client.set_charity_address(&admin, &charity_address);

    assert_eq!(client.get_charity_address(), Some(charity_address));
}

#[test]
fn test_update_collector_percentage() {
    let env = Env::default();
    let (client, admin, token_address) = create_test_contract(&env);

    env.mock_all_auths();

    client.__constructor(&admin, &token_address);

    // Update collector percentage
    client.update_collector_percentage(&admin, 10);

    assert_eq!(client.get_collector_percentage(), 10);
}

#[test]
#[should_panic(expected = "Total percentages cannot exceed 100")]
fn test_update_collector_percentage_validation() {
    let env = Env::default();
    let (client, admin, token_address) = create_test_contract(&env);

    env.mock_all_auths();

    client.__constructor(&admin, &token_address);

    // Try to set collector to 60% (60 + 50 = 110 > 100)
    client.update_collector_percentage(&admin, 60);
}

#[test]
fn test_update_owner_percentage() {
    let env = Env::default();
    let (client, admin, token_address) = create_test_contract(&env);

    env.mock_all_auths();

    client.__constructor(&admin, &token_address);

    // Update owner percentage
    client.update_owner_percentage(&admin, 40);

    assert_eq!(client.get_owner_percentage(), 40);
}

#[test]
#[should_panic(expected = "Total percentages cannot exceed 100")]
fn test_update_owner_percentage_validation() {
    let env = Env::default();
    let (client, admin, token_address) = create_test_contract(&env);

    env.mock_all_auths();

    client.__constructor(&admin, &token_address);

    // Try to set owner to 96% (5 + 96 = 101 > 100)
    client.update_owner_percentage(&admin, 96);
}

#[test]
fn test_initialization_requires_auth() {
    let env = Env::default();
    let (client, admin, token_address) = create_test_contract(&env);

    env.mock_all_auths();

    // This should work with mocked auth
    client.__constructor(&admin, &token_address);

    assert_eq!(client.is_initialized(), true);
}
