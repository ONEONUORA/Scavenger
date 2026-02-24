#![cfg(test)]

use soroban_sdk::{testutils::Address as _, Address, Env, String};
use stellar_scavngr_contract::{ScavengerContract, ScavengerContractClient, WasteType};

#[test]
fn test_get_waste_returns_correct_data() {
    let env = Env::default();
    let contract_id = env.register_contract(None, ScavengerContract);
    let client = ScavengerContractClient::new(&env, &contract_id);

    let user = Address::generate(&env);
    let description = String::from_str(&env, "Test waste material");
    env.mock_all_auths();

    // Register participant
    client.register_participant(&user, &stellar_scavngr_contract::ParticipantRole::Recycler);

    // Submit material
    let material = client.submit_material(&WasteType::Plastic, &5000, &user, &description);

    // Get waste by ID
    let retrieved = client.get_waste(&material.id);

    // Verify correct data returned
    assert!(retrieved.is_some());
    let waste = retrieved.unwrap();
    assert_eq!(waste.id, material.id);
    assert_eq!(waste.waste_type, WasteType::Plastic);
    assert_eq!(waste.weight, 5000);
    assert_eq!(waste.submitter, user);
    assert_eq!(waste.description, description);
}

#[test]
fn test_get_waste_handles_non_existent_id() {
    let env = Env::default();
    let contract_id = env.register_contract(None, ScavengerContract);
    let client = ScavengerContractClient::new(&env, &contract_id);

    // Try to get waste that doesn't exist
    let result = client.get_waste(&999);

    // Should return None gracefully
    assert!(result.is_none());
}

#[test]
fn test_get_waste_with_zero_id() {
    let env = Env::default();
    let contract_id = env.register_contract(None, ScavengerContract);
    let client = ScavengerContractClient::new(&env, &contract_id);

    // Try to get waste with ID 0 (should not exist)
    let result = client.get_waste(&0);

    // Should return None gracefully
    assert!(result.is_none());
}

#[test]
fn test_get_waste_multiple_materials() {
    let env = Env::default();
    let contract_id = env.register_contract(None, ScavengerContract);
    let client = ScavengerContractClient::new(&env, &contract_id);

    let user = Address::generate(&env);
    env.mock_all_auths();

    // Register participant
    client.register_participant(&user, &stellar_scavngr_contract::ParticipantRole::Recycler);

    // Submit multiple materials
    let desc1 = String::from_str(&env, "Plastic bottles");
    let desc2 = String::from_str(&env, "Metal cans");
    let desc3 = String::from_str(&env, "Glass jars");

    let m1 = client.submit_material(&WasteType::Plastic, &1000, &user, &desc1);
    let m2 = client.submit_material(&WasteType::Metal, &2000, &user, &desc2);
    let m3 = client.submit_material(&WasteType::Glass, &3000, &user, &desc3);

    // Get each waste and verify
    let w1 = client.get_waste(&m1.id).unwrap();
    let w2 = client.get_waste(&m2.id).unwrap();
    let w3 = client.get_waste(&m3.id).unwrap();

    assert_eq!(w1.waste_type, WasteType::Plastic);
    assert_eq!(w1.weight, 1000);
    
    assert_eq!(w2.waste_type, WasteType::Metal);
    assert_eq!(w2.weight, 2000);
    
    assert_eq!(w3.waste_type, WasteType::Glass);
    assert_eq!(w3.weight, 3000);
}

#[test]
fn test_get_waste_after_verification() {
    let env = Env::default();
    let contract_id = env.register_contract(None, ScavengerContract);
    let client = ScavengerContractClient::new(&env, &contract_id);

    let submitter = Address::generate(&env);
    let verifier = Address::generate(&env);
    let description = String::from_str(&env, "Paper waste");
    env.mock_all_auths();

    // Register participants
    client.register_participant(&submitter, &stellar_scavngr_contract::ParticipantRole::Collector);
    client.register_participant(&verifier, &stellar_scavngr_contract::ParticipantRole::Recycler);

    // Submit material
    let material = client.submit_material(&WasteType::Paper, &4000, &submitter, &description);
    
    // Verify it's not verified initially
    let waste_before = client.get_waste(&material.id).unwrap();
    assert!(!waste_before.verified);

    // Verify the material
    client.verify_material(&material.id, &verifier);

    // Get waste after verification
    let waste_after = client.get_waste(&material.id).unwrap();
    assert!(waste_after.verified);
}

#[test]
fn test_get_waste_consistency() {
    let env = Env::default();
    let contract_id = env.register_contract(None, ScavengerContract);
    let client = ScavengerContractClient::new(&env, &contract_id);

    let user = Address::generate(&env);
    let description = String::from_str(&env, "Consistent waste");
    env.mock_all_auths();

    // Register participant
    client.register_participant(&user, &stellar_scavngr_contract::ParticipantRole::Recycler);

    // Submit material
    let material = client.submit_material(&WasteType::Metal, &7000, &user, &description);

    // Get waste multiple times
    let w1 = client.get_waste(&material.id).unwrap();
    let w2 = client.get_waste(&material.id).unwrap();
    let w3 = client.get_waste(&material.id).unwrap();

    // All retrievals should return identical data
    assert_eq!(w1.id, w2.id);
    assert_eq!(w2.id, w3.id);
    assert_eq!(w1.waste_type, w2.waste_type);
    assert_eq!(w2.waste_type, w3.waste_type);
    assert_eq!(w1.weight, w2.weight);
    assert_eq!(w2.weight, w3.weight);
}

#[test]
fn test_get_waste_all_waste_types() {
    let env = Env::default();
    let contract_id = env.register_contract(None, ScavengerContract);
    let client = ScavengerContractClient::new(&env, &contract_id);

    let user = Address::generate(&env);
    env.mock_all_auths();

    // Register participant
    client.register_participant(&user, &stellar_scavngr_contract::ParticipantRole::Recycler);

    // Submit one of each waste type
    let desc = String::from_str(&env, "Test");
    let paper = client.submit_material(&WasteType::Paper, &1000, &user, &desc);
    let pet = client.submit_material(&WasteType::PetPlastic, &2000, &user, &desc);
    let plastic = client.submit_material(&WasteType::Plastic, &3000, &user, &desc);
    let metal = client.submit_material(&WasteType::Metal, &4000, &user, &desc);
    let glass = client.submit_material(&WasteType::Glass, &5000, &user, &desc);

    // Verify all can be retrieved
    assert_eq!(client.get_waste(&paper.id).unwrap().waste_type, WasteType::Paper);
    assert_eq!(client.get_waste(&pet.id).unwrap().waste_type, WasteType::PetPlastic);
    assert_eq!(client.get_waste(&plastic.id).unwrap().waste_type, WasteType::Plastic);
    assert_eq!(client.get_waste(&metal.id).unwrap().waste_type, WasteType::Metal);
    assert_eq!(client.get_waste(&glass.id).unwrap().waste_type, WasteType::Glass);
}

#[test]
fn test_get_waste_large_id() {
    let env = Env::default();
    let contract_id = env.register_contract(None, ScavengerContract);
    let client = ScavengerContractClient::new(&env, &contract_id);

    // Try to get waste with very large ID
    let result = client.get_waste(&u64::MAX);

    // Should return None gracefully without panic
    assert!(result.is_none());
}

#[test]
fn test_get_waste_sequential_ids() {
    let env = Env::default();
    let contract_id = env.register_contract(None, ScavengerContract);
    let client = ScavengerContractClient::new(&env, &contract_id);

    let user = Address::generate(&env);
    let description = String::from_str(&env, "Sequential test");
    env.mock_all_auths();

    // Register participant
    client.register_participant(&user, &stellar_scavngr_contract::ParticipantRole::Recycler);

    // Submit materials and verify IDs are sequential
    let m1 = client.submit_material(&WasteType::Paper, &1000, &user, &description);
    let m2 = client.submit_material(&WasteType::Plastic, &2000, &user, &description);
    let m3 = client.submit_material(&WasteType::Metal, &3000, &user, &description);

    // Verify we can get each by ID
    assert!(client.get_waste(&m1.id).is_some());
    assert!(client.get_waste(&m2.id).is_some());
    assert!(client.get_waste(&m3.id).is_some());
    
    // Verify IDs are sequential
    assert_eq!(m2.id, m1.id + 1);
    assert_eq!(m3.id, m2.id + 1);
}

#[test]
fn test_get_waste_alias_compatibility() {
    let env = Env::default();
    let contract_id = env.register_contract(None, ScavengerContract);
    let client = ScavengerContractClient::new(&env, &contract_id);

    let user = Address::generate(&env);
    let description = String::from_str(&env, "Alias test");
    env.mock_all_auths();

    // Register participant
    client.register_participant(&user, &stellar_scavngr_contract::ParticipantRole::Recycler);

    // Submit material
    let material = client.submit_material(&WasteType::Glass, &6000, &user, &description);

    // All three functions should return the same data
    let w1 = client.get_waste(&material.id);
    let w2 = client.get_waste_by_id(&material.id);
    let w3 = client.get_material(&material.id);

    assert!(w1.is_some());
    assert!(w2.is_some());
    assert!(w3.is_some());

    let waste1 = w1.unwrap();
    let waste2 = w2.unwrap();
    let waste3 = w3.unwrap();

    assert_eq!(waste1.id, waste2.id);
    assert_eq!(waste2.id, waste3.id);
    assert_eq!(waste1.waste_type, waste2.waste_type);
    assert_eq!(waste2.waste_type, waste3.waste_type);
}

#[test]
fn test_get_waste_error_handling() {
    let env = Env::default();
    let contract_id = env.register_contract(None, ScavengerContract);
    let client = ScavengerContractClient::new(&env, &contract_id);

    // Test various non-existent IDs
    assert!(client.get_waste(&0).is_none());
    assert!(client.get_waste(&1).is_none());
    assert!(client.get_waste(&100).is_none());
    assert!(client.get_waste(&999999).is_none());
    
    // All should return None without panicking
}
