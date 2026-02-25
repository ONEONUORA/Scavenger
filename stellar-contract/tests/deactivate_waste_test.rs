#![cfg(test)]

use soroban_sdk::{testutils::Address as _, Address, Env};
use stellar_contract::{ScavengerContract, ScavengerContractClient};

fn create_test_contract(env: &Env) -> (ScavengerContractClient, Address, Address, Address) {
    let contract_id = env.register_contract(None, ScavengerContract);
    let client = ScavengerContractClient::new(env, &contract_id);

    let admin = Address::generate(env);
    let token_address = Address::generate(env);
    let charity_address = Address::generate(env);

    client.initialize(
        &admin,
        &token_address,
        &charity_address,
        &50,
        &30,
        &20,
    );

    (client, admin, token_address, charity_address)
}

#[test]
fn test_deactivate_waste() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, admin, _token, _charity) = create_test_contract(&env);

    let owner = Address::generate(&env);

    // Register owner as collector
    client.register_participant(&owner, &stellar_contract::Role::Collector);

    // Register waste
    let waste = client.register_waste(
        &owner,
        &stellar_contract::WasteType::Plastic,
        &1000,
        &45_000_000,
        &-93_000_000,
    );

    // Verify waste is active
    assert_eq!(waste.is_active, true);

    // Deactivate waste as admin
    let deactivated = client.deactivate_waste(&waste.waste_id, &admin);

    // Verify waste is deactivated
    assert_eq!(deactivated.is_active, false);
    assert_eq!(deactivated.waste_id, waste.waste_id);
}

#[test]
#[should_panic(expected = "Unauthorized: caller is not admin")]
fn test_deactivate_waste_non_admin() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, _admin, _token, _charity) = create_test_contract(&env);

    let owner = Address::generate(&env);
    let non_admin = Address::generate(&env);

    // Register owner as collector
    client.register_participant(&owner, &stellar_contract::Role::Collector);

    // Register waste
    let waste = client.register_waste(
        &owner,
        &stellar_contract::WasteType::Plastic,
        &1000,
        &45_000_000,
        &-93_000_000,
    );

    // Try to deactivate as non-admin (should panic)
    client.deactivate_waste(&waste.waste_id, &non_admin);
}

#[test]
#[should_panic(expected = "Waste already deactivated")]
fn test_deactivate_already_deactivated_waste() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, admin, _token, _charity) = create_test_contract(&env);

    let owner = Address::generate(&env);

    // Register owner as collector
    client.register_participant(&owner, &stellar_contract::Role::Collector);

    // Register waste
    let waste = client.register_waste(
        &owner,
        &stellar_contract::WasteType::Plastic,
        &1000,
        &45_000_000,
        &-93_000_000,
    );

    // Deactivate waste
    client.deactivate_waste(&waste.waste_id, &admin);

    // Try to deactivate again (should panic)
    client.deactivate_waste(&waste.waste_id, &admin);
}

#[test]
#[should_panic(expected = "Waste not found")]
fn test_deactivate_nonexistent_waste() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, admin, _token, _charity) = create_test_contract(&env);

    // Try to deactivate non-existent waste (should panic)
    client.deactivate_waste(&999, &admin);
}

#[test]
fn test_deactivated_waste_not_counted_in_totals() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, admin, _token, _charity) = create_test_contract(&env);

    let owner = Address::generate(&env);

    // Register owner as collector
    client.register_participant(&owner, &stellar_contract::Role::Collector);

    // Register two waste items
    let waste1 = client.register_waste(
        &owner,
        &stellar_contract::WasteType::Plastic,
        &1000,
        &45_000_000,
        &-93_000_000,
    );

    let waste2 = client.register_waste(
        &owner,
        &stellar_contract::WasteType::Metal,
        &2000,
        &45_000_000,
        &-93_000_000,
    );

    // Get initial stats
    let initial_stats = client.get_stats();
    let initial_weight = initial_stats.total_weight;

    // Deactivate first waste
    client.deactivate_waste(&waste1.waste_id, &admin);

    // Get updated stats
    let updated_stats = client.get_stats();
    let updated_weight = updated_stats.total_weight;

    // Verify deactivated waste is not counted
    // The weight should decrease by waste1's weight
    assert!(updated_weight < initial_weight);
    
    // Verify waste2 is still counted
    assert!(updated_weight > 0);
}

#[test]
#[should_panic(expected = "Cannot transfer deactivated waste")]
fn test_deactivated_waste_cannot_be_transferred() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, admin, _token, _charity) = create_test_contract(&env);

    let owner = Address::generate(&env);
    let recipient = Address::generate(&env);

    // Register participants
    client.register_participant(&owner, &stellar_contract::Role::Collector);
    client.register_participant(&recipient, &stellar_contract::Role::Manufacturer);

    // Register waste
    let waste = client.register_waste(
        &owner,
        &stellar_contract::WasteType::Plastic,
        &1000,
        &45_000_000,
        &-93_000_000,
    );

    // Deactivate waste
    client.deactivate_waste(&waste.waste_id, &admin);

    // Try to transfer deactivated waste (should panic)
    client.transfer_waste_v2(
        &waste.waste_id,
        &owner,
        &recipient,
        &45_000_000,
        &-93_000_000,
    );
}


#[test]
#[should_panic(expected = "Cannot confirm deactivated waste")]
fn test_deactivated_waste_cannot_be_confirmed() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, admin, _token, _charity) = create_test_contract(&env);

    let owner = Address::generate(&env);
    let confirmer = Address::generate(&env);

    // Register owner as collector
    client.register_participant(&owner, &stellar_contract::Role::Collector);

    // Register waste
    let waste = client.register_waste(
        &owner,
        &stellar_contract::WasteType::Plastic,
        &1000,
        &45_000_000,
        &-93_000_000,
    );

    // Deactivate waste
    client.deactivate_waste(&waste.waste_id, &admin);

    // Try to confirm deactivated waste (should panic)
    client.confirm_waste_details(&waste.waste_id, &confirmer);
}
