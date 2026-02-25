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
fn test_deactivate_waste_success() {
    let env = Env::default();
    env.mock_all_auths();
    
    let (client, admin, _token, _charity) = create_test_contract(&env);
    let submitter = Address::generate(&env);
    
    // Register submitter
    client.register_participant(
        &submitter,
        &Role::Collector,
        &soroban_sdk::String::from_str(&env, "Test Collector"),
        &1000000,
        &2000000,
    );
    
    // Submit material
    let material = client.submit_material(
        &submitter,
        &WasteType::Paper,
        &5000,
    );
    
    assert_eq!(material.is_active, true);
    
    // Verify material is queryable
    let retrieved = client.get_material(&material.id);
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().id, material.id);
    
    // Deactivate waste
    client.deactivate_waste(&admin, &material.id);
    
    // Verify material is no longer queryable
    let retrieved_after = client.get_material(&material.id);
    assert!(retrieved_after.is_none());
}

#[test]
#[should_panic(expected = "Only admin can perform this action")]
fn test_deactivate_waste_non_admin() {
    let env = Env::default();
    env.mock_all_auths();
    
    let (client, _admin, _token, _charity) = create_test_contract(&env);
    let submitter = Address::generate(&env);
    let non_admin = Address::generate(&env);
    
    // Register submitter
    client.register_participant(
        &submitter,
        &Role::Collector,
        &soroban_sdk::String::from_str(&env, "Test Collector"),
        &1000000,
        &2000000,
    );
    
    // Submit material
    let material = client.submit_material(
        &submitter,
        &WasteType::Paper,
        &5000,
    );
    
    // Try to deactivate as non-admin (should fail)
    client.deactivate_waste(&non_admin, &material.id);
}

#[test]
#[should_panic(expected = "Waste not found")]
fn test_deactivate_waste_not_found() {
    let env = Env::default();
    env.mock_all_auths();
    
    let (client, admin, _token, _charity) = create_test_contract(&env);
    
    // Try to deactivate non-existent waste
    client.deactivate_waste(&admin, &999);
}

#[test]
#[should_panic(expected = "Waste already deactivated")]
fn test_deactivate_waste_already_deactivated() {
    let env = Env::default();
    env.mock_all_auths();
    
    let (client, admin, _token, _charity) = create_test_contract(&env);
    let submitter = Address::generate(&env);
    
    // Register submitter
    client.register_participant(
        &submitter,
        &Role::Collector,
        &soroban_sdk::String::from_str(&env, "Test Collector"),
        &1000000,
        &2000000,
    );
    
    // Submit material
    let material = client.submit_material(
        &submitter,
        &WasteType::Paper,
        &5000,
    );
    
    // Deactivate waste
    client.deactivate_waste(&admin, &material.id);
    
    // Try to deactivate again (should fail)
    client.deactivate_waste(&admin, &material.id);
}

#[test]
fn test_deactivated_waste_not_queryable() {
    let env = Env::default();
    env.mock_all_auths();
    
    let (client, admin, _token, _charity) = create_test_contract(&env);
    let submitter = Address::generate(&env);
    
    // Register submitter
    client.register_participant(
        &submitter,
        &Role::Collector,
        &soroban_sdk::String::from_str(&env, "Test Collector"),
        &1000000,
        &2000000,
    );
    
    // Submit multiple materials
    let material1 = client.submit_material(&submitter, &WasteType::Paper, &5000);
    let material2 = client.submit_material(&submitter, &WasteType::Plastic, &3000);
    let material3 = client.submit_material(&submitter, &WasteType::Metal, &2000);
    
    // Verify all are queryable
    assert!(client.get_material(&material1.id).is_some());
    assert!(client.get_material(&material2.id).is_some());
    assert!(client.get_material(&material3.id).is_some());
    
    // Deactivate material2
    client.deactivate_waste(&admin, &material2.id);
    
    // Verify only material2 is not queryable
    assert!(client.get_material(&material1.id).is_some());
    assert!(client.get_material(&material2.id).is_none());
    assert!(client.get_material(&material3.id).is_some());
}

#[test]
fn test_deactivate_waste_multiple() {
    let env = Env::default();
    env.mock_all_auths();
    
    let (client, admin, _token, _charity) = create_test_contract(&env);
    let submitter = Address::generate(&env);
    
    // Register submitter
    client.register_participant(
        &submitter,
        &Role::Collector,
        &soroban_sdk::String::from_str(&env, "Test Collector"),
        &1000000,
        &2000000,
    );
    
    // Submit multiple materials
    let material1 = client.submit_material(&submitter, &WasteType::Paper, &5000);
    let material2 = client.submit_material(&submitter, &WasteType::Plastic, &3000);
    let material3 = client.submit_material(&submitter, &WasteType::Metal, &2000);
    
    // Deactivate all materials
    client.deactivate_waste(&admin, &material1.id);
    client.deactivate_waste(&admin, &material2.id);
    client.deactivate_waste(&admin, &material3.id);
    
    // Verify none are queryable
    assert!(client.get_material(&material1.id).is_none());
    assert!(client.get_material(&material2.id).is_none());
    assert!(client.get_material(&material3.id).is_none());
}

#[test]
fn test_new_material_is_active_by_default() {
    let env = Env::default();
    env.mock_all_auths();
    
    let (client, _admin, _token, _charity) = create_test_contract(&env);
    let submitter = Address::generate(&env);
    
    // Register submitter
    client.register_participant(
        &submitter,
        &Role::Collector,
        &soroban_sdk::String::from_str(&env, "Test Collector"),
        &1000000,
        &2000000,
    );
    
    // Submit material
    let material = client.submit_material(
        &submitter,
        &WasteType::Paper,
        &5000,
    );
    
    // Verify material is active by default
    assert_eq!(material.is_active, true);
    
    // Verify it's queryable
    let retrieved = client.get_material(&material.id);
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().is_active, true);
}

#[test]
fn test_deactivate_waste_different_waste_types() {
    let env = Env::default();
    env.mock_all_auths();
    
    let (client, admin, _token, _charity) = create_test_contract(&env);
    let submitter = Address::generate(&env);
    
    // Register submitter
    client.register_participant(
        &submitter,
        &Role::Collector,
        &soroban_sdk::String::from_str(&env, "Test Collector"),
        &1000000,
        &2000000,
    );
    
    // Submit different waste types
    let paper = client.submit_material(&submitter, &WasteType::Paper, &5000);
    let plastic = client.submit_material(&submitter, &WasteType::Plastic, &3000);
    let metal = client.submit_material(&submitter, &WasteType::Metal, &2000);
    let glass = client.submit_material(&submitter, &WasteType::Glass, &4000);
    let pet = client.submit_material(&submitter, &WasteType::PetPlastic, &1000);
    
    // Deactivate each type
    client.deactivate_waste(&admin, &paper.id);
    client.deactivate_waste(&admin, &plastic.id);
    client.deactivate_waste(&admin, &metal.id);
    client.deactivate_waste(&admin, &glass.id);
    client.deactivate_waste(&admin, &pet.id);
    
    // Verify all are not queryable
    assert!(client.get_material(&paper.id).is_none());
    assert!(client.get_material(&plastic.id).is_none());
    assert!(client.get_material(&metal.id).is_none());
    assert!(client.get_material(&glass.id).is_none());
    assert!(client.get_material(&pet.id).is_none());
}
