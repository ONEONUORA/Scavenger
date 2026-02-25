#![cfg(test)]

use soroban_sdk::{testutils::Address as _, Address, Env};

use crate::contract::ScavengerContract;
use crate::types::{Role, WasteType};

fn create_test_contract(env: &Env) -> (crate::contract::ScavengerContractClient<'_>, Address, Address, Address) {
    let contract_id = env.register_contract(None, ScavengerContract);
    let client = crate::contract::ScavengerContractClient::new(env, &contract_id);
    
    let admin = Address::generate(env);
    let token_address = Address::generate(env);
    let charity_address = Address::generate(env);
    
    client.__constructor(&admin, &token_address, &charity_address, &5, &50);
    
    (client, admin, token_address, charity_address)
}

#[test]
fn test_reset_waste_confirmation_success() {
    let env = Env::default();
    env.mock_all_auths();
    
    let (client, _admin, _token, _charity) = create_test_contract(&env);
    let owner = Address::generate(&env);
    let confirmer = Address::generate(&env);
    
    // Register participants
    client.register_participant(
        &owner,
        &Role::Collector,
        &soroban_sdk::String::from_str(&env, "Owner"),
        &1000000,
        &2000000,
    );
    
    client.register_participant(
        &confirmer,
        &Role::Recycler,
        &soroban_sdk::String::from_str(&env, "Confirmer"),
        &3000000,
        &4000000,
    );
    
    // Submit material
    let material = client.submit_material(&owner, &WasteType::Paper, &5000);
    
    // Verify initial state
    assert_eq!(material.is_confirmed, false);
    
    // Confirm the waste
    client.confirm_waste(&material.id, &confirmer);
    
    // Verify confirmation
    let confirmed_material = client.get_material(&material.id).unwrap();
    assert_eq!(confirmed_material.is_confirmed, true);
    assert_eq!(confirmed_material.confirmer, confirmer);
    
    // Reset confirmation
    client.reset_waste_confirmation(&material.id, &owner);
    
    // Verify reset
    let reset_material = client.get_material(&material.id).unwrap();
    assert_eq!(reset_material.is_confirmed, false);
    assert_eq!(reset_material.confirmer, reset_material.submitter);
}

#[test]
#[should_panic(expected = "Only waste owner can reset confirmation")]
fn test_reset_waste_confirmation_non_owner() {
    let env = Env::default();
    env.mock_all_auths();
    
    let (client, _admin, _token, _charity) = create_test_contract(&env);
    let owner = Address::generate(&env);
    let non_owner = Address::generate(&env);
    let confirmer = Address::generate(&env);
    
    // Register participants
    client.register_participant(
        &owner,
        &Role::Collector,
        &soroban_sdk::String::from_str(&env, "Owner"),
        &1000000,
        &2000000,
    );
    
    client.register_participant(
        &non_owner,
        &Role::Collector,
        &soroban_sdk::String::from_str(&env, "Non-Owner"),
        &3000000,
        &4000000,
    );
    
    client.register_participant(
        &confirmer,
        &Role::Recycler,
        &soroban_sdk::String::from_str(&env, "Confirmer"),
        &5000000,
        &6000000,
    );
    
    // Submit and confirm material
    let material = client.submit_material(&owner, &WasteType::Paper, &5000);
    client.confirm_waste(&material.id, &confirmer);
    
    // Try to reset as non-owner (should fail)
    client.reset_waste_confirmation(&material.id, &non_owner);
}

#[test]
#[should_panic(expected = "Waste not found")]
fn test_reset_waste_confirmation_not_found() {
    let env = Env::default();
    env.mock_all_auths();
    
    let (client, _admin, _token, _charity) = create_test_contract(&env);
    let owner = Address::generate(&env);
    
    // Register owner
    client.register_participant(
        &owner,
        &Role::Collector,
        &soroban_sdk::String::from_str(&env, "Owner"),
        &1000000,
        &2000000,
    );
    
    // Try to reset non-existent waste
    client.reset_waste_confirmation(&999, &owner);
}

#[test]
#[should_panic(expected = "Waste is not active")]
fn test_reset_waste_confirmation_inactive() {
    let env = Env::default();
    env.mock_all_auths();
    
    let (client, admin, _token, _charity) = create_test_contract(&env);
    let owner = Address::generate(&env);
    let confirmer = Address::generate(&env);
    
    // Register participants
    client.register_participant(
        &owner,
        &Role::Collector,
        &soroban_sdk::String::from_str(&env, "Owner"),
        &1000000,
        &2000000,
    );
    
    client.register_participant(
        &confirmer,
        &Role::Recycler,
        &soroban_sdk::String::from_str(&env, "Confirmer"),
        &3000000,
        &4000000,
    );
    
    // Submit and confirm material
    let material = client.submit_material(&owner, &WasteType::Paper, &5000);
    client.confirm_waste(&material.id, &confirmer);
    
    // Deactivate the waste
    client.deactivate_waste(&admin, &material.id);
    
    // Try to reset confirmation on inactive waste (should fail)
    client.reset_waste_confirmation(&material.id, &owner);
}

#[test]
fn test_reset_allows_reconfirmation() {
    let env = Env::default();
    env.mock_all_auths();
    
    let (client, _admin, _token, _charity) = create_test_contract(&env);
    let owner = Address::generate(&env);
    let confirmer1 = Address::generate(&env);
    let confirmer2 = Address::generate(&env);
    
    // Register participants
    client.register_participant(
        &owner,
        &Role::Collector,
        &soroban_sdk::String::from_str(&env, "Owner"),
        &1000000,
        &2000000,
    );
    
    client.register_participant(
        &confirmer1,
        &Role::Recycler,
        &soroban_sdk::String::from_str(&env, "Confirmer1"),
        &3000000,
        &4000000,
    );
    
    client.register_participant(
        &confirmer2,
        &Role::Manufacturer,
        &soroban_sdk::String::from_str(&env, "Confirmer2"),
        &5000000,
        &6000000,
    );
    
    // Submit material
    let material = client.submit_material(&owner, &WasteType::Paper, &5000);
    
    // First confirmation
    client.confirm_waste(&material.id, &confirmer1);
    let confirmed1 = client.get_material(&material.id).unwrap();
    assert_eq!(confirmed1.is_confirmed, true);
    assert_eq!(confirmed1.confirmer, confirmer1);
    
    // Reset confirmation
    client.reset_waste_confirmation(&material.id, &owner);
    let reset = client.get_material(&material.id).unwrap();
    assert_eq!(reset.is_confirmed, false);
    
    // Re-confirm with different confirmer
    client.confirm_waste(&material.id, &confirmer2);
    let confirmed2 = client.get_material(&material.id).unwrap();
    assert_eq!(confirmed2.is_confirmed, true);
    assert_eq!(confirmed2.confirmer, confirmer2);
}

#[test]
fn test_reset_multiple_times() {
    let env = Env::default();
    env.mock_all_auths();
    
    let (client, _admin, _token, _charity) = create_test_contract(&env);
    let owner = Address::generate(&env);
    let confirmer = Address::generate(&env);
    
    // Register participants
    client.register_participant(
        &owner,
        &Role::Collector,
        &soroban_sdk::String::from_str(&env, "Owner"),
        &1000000,
        &2000000,
    );
    
    client.register_participant(
        &confirmer,
        &Role::Recycler,
        &soroban_sdk::String::from_str(&env, "Confirmer"),
        &3000000,
        &4000000,
    );
    
    // Submit material
    let material = client.submit_material(&owner, &WasteType::Paper, &5000);
    
    // Cycle: confirm -> reset -> confirm -> reset
    for _ in 0..3 {
        // Confirm
        client.confirm_waste(&material.id, &confirmer);
        let confirmed = client.get_material(&material.id).unwrap();
        assert_eq!(confirmed.is_confirmed, true);
        
        // Reset
        client.reset_waste_confirmation(&material.id, &owner);
        let reset = client.get_material(&material.id).unwrap();
        assert_eq!(reset.is_confirmed, false);
    }
}

#[test]
fn test_new_material_not_confirmed() {
    let env = Env::default();
    env.mock_all_auths();
    
    let (client, _admin, _token, _charity) = create_test_contract(&env);
    let owner = Address::generate(&env);
    
    // Register owner
    client.register_participant(
        &owner,
        &Role::Collector,
        &soroban_sdk::String::from_str(&env, "Owner"),
        &1000000,
        &2000000,
    );
    
    // Submit material
    let material = client.submit_material(&owner, &WasteType::Paper, &5000);
    
    // Verify not confirmed by default
    assert_eq!(material.is_confirmed, false);
    assert_eq!(material.confirmer, material.submitter);
}

#[test]
fn test_confirm_waste_success() {
    let env = Env::default();
    env.mock_all_auths();
    
    let (client, _admin, _token, _charity) = create_test_contract(&env);
    let owner = Address::generate(&env);
    let confirmer = Address::generate(&env);
    
    // Register participants
    client.register_participant(
        &owner,
        &Role::Collector,
        &soroban_sdk::String::from_str(&env, "Owner"),
        &1000000,
        &2000000,
    );
    
    client.register_participant(
        &confirmer,
        &Role::Recycler,
        &soroban_sdk::String::from_str(&env, "Confirmer"),
        &3000000,
        &4000000,
    );
    
    // Submit material
    let material = client.submit_material(&owner, &WasteType::Paper, &5000);
    
    // Confirm waste
    client.confirm_waste(&material.id, &confirmer);
    
    // Verify confirmation
    let confirmed = client.get_material(&material.id).unwrap();
    assert_eq!(confirmed.is_confirmed, true);
    assert_eq!(confirmed.confirmer, confirmer);
}

#[test]
#[should_panic(expected = "Confirmer not registered")]
fn test_confirm_waste_unregistered_confirmer() {
    let env = Env::default();
    env.mock_all_auths();
    
    let (client, _admin, _token, _charity) = create_test_contract(&env);
    let owner = Address::generate(&env);
    let unregistered = Address::generate(&env);
    
    // Register only owner
    client.register_participant(
        &owner,
        &Role::Collector,
        &soroban_sdk::String::from_str(&env, "Owner"),
        &1000000,
        &2000000,
    );
    
    // Submit material
    let material = client.submit_material(&owner, &WasteType::Paper, &5000);
    
    // Try to confirm with unregistered user (should fail)
    client.confirm_waste(&material.id, &unregistered);
}

#[test]
fn test_reset_after_transfer() {
    let env = Env::default();
    env.mock_all_auths();
    
    let (client, _admin, _token, _charity) = create_test_contract(&env);
    let original_owner = Address::generate(&env);
    let new_owner = Address::generate(&env);
    let confirmer = Address::generate(&env);
    
    // Register participants
    client.register_participant(
        &original_owner,
        &Role::Collector,
        &soroban_sdk::String::from_str(&env, "Original Owner"),
        &1000000,
        &2000000,
    );
    
    client.register_participant(
        &new_owner,
        &Role::Recycler,
        &soroban_sdk::String::from_str(&env, "New Owner"),
        &3000000,
        &4000000,
    );
    
    client.register_participant(
        &confirmer,
        &Role::Manufacturer,
        &soroban_sdk::String::from_str(&env, "Confirmer"),
        &5000000,
        &6000000,
    );
    
    // Submit and confirm material
    let material = client.submit_material(&original_owner, &WasteType::Paper, &5000);
    client.confirm_waste(&material.id, &confirmer);
    
    // Transfer waste
    client.transfer_waste(&material.id, &original_owner, &new_owner);
    
    // New owner should be able to reset confirmation
    client.reset_waste_confirmation(&material.id, &new_owner);
    
    let reset = client.get_material(&material.id).unwrap();
    assert_eq!(reset.is_confirmed, false);
    assert_eq!(reset.current_owner, new_owner);
}
