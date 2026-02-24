#![cfg(test)]

use soroban_sdk::{testutils::Address as _, Address, Env, String, Vec};
use stellar_scavngr_contract::{ScavengerContract, ScavengerContractClient, ParticipantRole, WasteType};

#[test]
fn test_get_participant_wastes_returns_owned_ids() {
    let env = Env::default();
    let contract_id = env.register_contract(None, ScavengerContract);
    let client = ScavengerContractClient::new(&env, &contract_id);

    let user = Address::generate(&env);
    let description = String::from_str(&env, "Test waste");
    env.mock_all_auths();

    // Register participant
    client.register_participant(&user, &ParticipantRole::Recycler);

    // Submit multiple materials
    let m1 = client.submit_material(&WasteType::Plastic, &1000, &user, &description);
    let m2 = client.submit_material(&WasteType::Metal, &2000, &user, &description);
    let m3 = client.submit_material(&WasteType::Glass, &3000, &user, &description);

    // Get participant wastes
    let waste_ids = client.get_participant_wastes(&user);

    // Verify all waste IDs are returned
    assert_eq!(waste_ids.len(), 3);
    assert!(waste_ids.contains(&m1.id));
    assert!(waste_ids.contains(&m2.id));
    assert!(waste_ids.contains(&m3.id));
}

#[test]
fn test_get_participant_wastes_empty_result() {
    let env = Env::default();
    let contract_id = env.register_contract(None, ScavengerContract);
    let client = ScavengerContractClient::new(&env, &contract_id);

    let user = Address::generate(&env);
    env.mock_all_auths();

    // Register participant but don't submit any wastes
    client.register_participant(&user, &ParticipantRole::Recycler);

    // Get participant wastes
    let waste_ids = client.get_participant_wastes(&user);

    // Should return empty vector
    assert_eq!(waste_ids.len(), 0);
}

#[test]
fn test_get_participant_wastes_unregistered_participant() {
    let env = Env::default();
    let contract_id = env.register_contract(None, ScavengerContract);
    let client = ScavengerContractClient::new(&env, &contract_id);

    let unregistered_user = Address::generate(&env);

    // Get wastes for unregistered participant
    let waste_ids = client.get_participant_wastes(&unregistered_user);

    // Should return empty vector
    assert_eq!(waste_ids.len(), 0);
}

#[test]
fn test_get_participant_wastes_multiple_participants() {
    let env = Env::default();
    let contract_id = env.register_contract(None, ScavengerContract);
    let client = ScavengerContractClient::new(&env, &contract_id);

    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);
    let description = String::from_str(&env, "Test");
    env.mock_all_auths();

    // Register participants
    client.register_participant(&user1, &ParticipantRole::Recycler);
    client.register_participant(&user2, &ParticipantRole::Collector);

    // User1 submits 2 wastes
    let m1 = client.submit_material(&WasteType::Paper, &1000, &user1, &description);
    let m2 = client.submit_material(&WasteType::Plastic, &2000, &user1, &description);

    // User2 submits 3 wastes
    let m3 = client.submit_material(&WasteType::Metal, &3000, &user2, &description);
    let m4 = client.submit_material(&WasteType::Glass, &4000, &user2, &description);
    let m5 = client.submit_material(&WasteType::Paper, &5000, &user2, &description);

    // Get wastes for each participant
    let user1_wastes = client.get_participant_wastes(&user1);
    let user2_wastes = client.get_participant_wastes(&user2);

    // Verify correct wastes for each participant
    assert_eq!(user1_wastes.len(), 2);
    assert!(user1_wastes.contains(&m1.id));
    assert!(user1_wastes.contains(&m2.id));

    assert_eq!(user2_wastes.len(), 3);
    assert!(user2_wastes.contains(&m3.id));
    assert!(user2_wastes.contains(&m4.id));
    assert!(user2_wastes.contains(&m5.id));

    // Verify no cross-contamination
    assert!(!user1_wastes.contains(&m3.id));
    assert!(!user2_wastes.contains(&m1.id));
}

#[test]
fn test_get_participant_wastes_updates_after_transfer() {
    let env = Env::default();
    let contract_id = env.register_contract(None, ScavengerContract);
    let client = ScavengerContractClient::new(&env, &contract_id);

    let sender = Address::generate(&env);
    let receiver = Address::generate(&env);
    let description = String::from_str(&env, "Transfer test");
    let note = String::from_str(&env, "Transferring waste");
    env.mock_all_auths();

    // Register participants
    client.register_participant(&sender, &ParticipantRole::Recycler);
    client.register_participant(&receiver, &ParticipantRole::Collector);

    // Sender submits wastes
    let m1 = client.submit_material(&WasteType::Plastic, &1000, &sender, &description);
    let m2 = client.submit_material(&WasteType::Metal, &2000, &sender, &description);

    // Verify sender owns both wastes
    let sender_wastes_before = client.get_participant_wastes(&sender);
    assert_eq!(sender_wastes_before.len(), 2);
    assert!(sender_wastes_before.contains(&m1.id));
    assert!(sender_wastes_before.contains(&m2.id));

    // Receiver owns no wastes
    let receiver_wastes_before = client.get_participant_wastes(&receiver);
    assert_eq!(receiver_wastes_before.len(), 0);

    // Transfer one waste from sender to receiver
    client.transfer_waste(&m1.id, &sender, &receiver, &note);

    // Verify ownership updated
    let sender_wastes_after = client.get_participant_wastes(&sender);
    let receiver_wastes_after = client.get_participant_wastes(&receiver);

    // Sender should only have m2 now
    assert_eq!(sender_wastes_after.len(), 1);
    assert!(sender_wastes_after.contains(&m2.id));
    assert!(!sender_wastes_after.contains(&m1.id));

    // Receiver should have m1 now
    assert_eq!(receiver_wastes_after.len(), 1);
    assert!(receiver_wastes_after.contains(&m1.id));
}

#[test]
fn test_get_participant_wastes_after_multiple_transfers() {
    let env = Env::default();
    let contract_id = env.register_contract(None, ScavengerContract);
    let client = ScavengerContractClient::new(&env, &contract_id);

    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);
    let user3 = Address::generate(&env);
    let description = String::from_str(&env, "Multi-transfer test");
    let note = String::from_str(&env, "Transfer");
    env.mock_all_auths();

    // Register participants
    client.register_participant(&user1, &ParticipantRole::Recycler);
    client.register_participant(&user2, &ParticipantRole::Collector);
    client.register_participant(&user3, &ParticipantRole::Manufacturer);

    // User1 submits 3 wastes
    let m1 = client.submit_material(&WasteType::Paper, &1000, &user1, &description);
    let m2 = client.submit_material(&WasteType::Plastic, &2000, &user1, &description);
    let m3 = client.submit_material(&WasteType::Metal, &3000, &user1, &description);

    // Transfer m1: user1 -> user2
    client.transfer_waste(&m1.id, &user1, &user2, &note);

    // Transfer m2: user1 -> user3
    client.transfer_waste(&m2.id, &user1, &user3, &note);

    // Transfer m1: user2 -> user3
    client.transfer_waste(&m1.id, &user2, &user3, &note);

    // Verify final ownership
    let user1_wastes = client.get_participant_wastes(&user1);
    let user2_wastes = client.get_participant_wastes(&user2);
    let user3_wastes = client.get_participant_wastes(&user3);

    // User1 should only have m3
    assert_eq!(user1_wastes.len(), 1);
    assert!(user1_wastes.contains(&m3.id));

    // User2 should have nothing
    assert_eq!(user2_wastes.len(), 0);

    // User3 should have m1 and m2
    assert_eq!(user3_wastes.len(), 2);
    assert!(user3_wastes.contains(&m1.id));
    assert!(user3_wastes.contains(&m2.id));
}

#[test]
fn test_get_participant_wastes_all_waste_types() {
    let env = Env::default();
    let contract_id = env.register_contract(None, ScavengerContract);
    let client = ScavengerContractClient::new(&env, &contract_id);

    let user = Address::generate(&env);
    let description = String::from_str(&env, "All types test");
    env.mock_all_auths();

    // Register participant
    client.register_participant(&user, &ParticipantRole::Recycler);

    // Submit one of each waste type
    let m1 = client.submit_material(&WasteType::Paper, &1000, &user, &description);
    let m2 = client.submit_material(&WasteType::PetPlastic, &2000, &user, &description);
    let m3 = client.submit_material(&WasteType::Plastic, &3000, &user, &description);
    let m4 = client.submit_material(&WasteType::Metal, &4000, &user, &description);
    let m5 = client.submit_material(&WasteType::Glass, &5000, &user, &description);

    // Get participant wastes
    let waste_ids = client.get_participant_wastes(&user);

    // Verify all types are included
    assert_eq!(waste_ids.len(), 5);
    assert!(waste_ids.contains(&m1.id));
    assert!(waste_ids.contains(&m2.id));
    assert!(waste_ids.contains(&m3.id));
    assert!(waste_ids.contains(&m4.id));
    assert!(waste_ids.contains(&m5.id));
}

#[test]
fn test_get_participant_wastes_large_number() {
    let env = Env::default();
    let contract_id = env.register_contract(None, ScavengerContract);
    let client = ScavengerContractClient::new(&env, &contract_id);

    let user = Address::generate(&env);
    let description = String::from_str(&env, "Large test");
    env.mock_all_auths();

    // Register participant
    client.register_participant(&user, &ParticipantRole::Recycler);

    // Submit 10 wastes
    let mut expected_ids = Vec::new(&env);
    for i in 0..10 {
        let waste_type = match i % 5 {
            0 => WasteType::Paper,
            1 => WasteType::PetPlastic,
            2 => WasteType::Plastic,
            3 => WasteType::Metal,
            _ => WasteType::Glass,
        };
        let material = client.submit_material(&waste_type, &(1000 + i * 100), &user, &description);
        expected_ids.push_back(material.id);
    }

    // Get participant wastes
    let waste_ids = client.get_participant_wastes(&user);

    // Verify all wastes are returned
    assert_eq!(waste_ids.len(), 10);
    for expected_id in expected_ids.iter() {
        assert!(waste_ids.contains(&expected_id));
    }
}

#[test]
fn test_get_participant_wastes_consistency() {
    let env = Env::default();
    let contract_id = env.register_contract(None, ScavengerContract);
    let client = ScavengerContractClient::new(&env, &contract_id);

    let user = Address::generate(&env);
    let description = String::from_str(&env, "Consistency test");
    env.mock_all_auths();

    // Register participant
    client.register_participant(&user, &ParticipantRole::Recycler);

    // Submit wastes
    client.submit_material(&WasteType::Paper, &1000, &user, &description);
    client.submit_material(&WasteType::Plastic, &2000, &user, &description);

    // Get wastes multiple times
    let wastes1 = client.get_participant_wastes(&user);
    let wastes2 = client.get_participant_wastes(&user);
    let wastes3 = client.get_participant_wastes(&user);

    // All calls should return identical results
    assert_eq!(wastes1.len(), wastes2.len());
    assert_eq!(wastes2.len(), wastes3.len());

    for id in wastes1.iter() {
        assert!(wastes2.contains(&id));
        assert!(wastes3.contains(&id));
    }
}

#[test]
fn test_get_participant_wastes_after_verification() {
    let env = Env::default();
    let contract_id = env.register_contract(None, ScavengerContract);
    let client = ScavengerContractClient::new(&env, &contract_id);

    let submitter = Address::generate(&env);
    let verifier = Address::generate(&env);
    let description = String::from_str(&env, "Verification test");
    env.mock_all_auths();

    // Register participants
    client.register_participant(&submitter, &ParticipantRole::Collector);
    client.register_participant(&verifier, &ParticipantRole::Recycler);

    // Submit waste
    let material = client.submit_material(&WasteType::Metal, &5000, &submitter, &description);

    // Get wastes before verification
    let wastes_before = client.get_participant_wastes(&submitter);
    assert_eq!(wastes_before.len(), 1);
    assert!(wastes_before.contains(&material.id));

    // Verify the material
    client.verify_material(&material.id, &verifier);

    // Get wastes after verification - ownership should not change
    let wastes_after = client.get_participant_wastes(&submitter);
    assert_eq!(wastes_after.len(), 1);
    assert!(wastes_after.contains(&material.id));
}

#[test]
fn test_get_participant_wastes_order() {
    let env = Env::default();
    let contract_id = env.register_contract(None, ScavengerContract);
    let client = ScavengerContractClient::new(&env, &contract_id);

    let user = Address::generate(&env);
    let description = String::from_str(&env, "Order test");
    env.mock_all_auths();

    // Register participant
    client.register_participant(&user, &ParticipantRole::Recycler);

    // Submit wastes in specific order
    let m1 = client.submit_material(&WasteType::Paper, &1000, &user, &description);
    let m2 = client.submit_material(&WasteType::Plastic, &2000, &user, &description);
    let m3 = client.submit_material(&WasteType::Metal, &3000, &user, &description);

    // Get participant wastes
    let waste_ids = client.get_participant_wastes(&user);

    // Verify IDs are in sequential order (as they were created)
    assert_eq!(waste_ids.len(), 3);
    assert_eq!(waste_ids.get(0).unwrap(), m1.id);
    assert_eq!(waste_ids.get(1).unwrap(), m2.id);
    assert_eq!(waste_ids.get(2).unwrap(), m3.id);
}

#[test]
fn test_get_participant_wastes_no_side_effects() {
    let env = Env::default();
    let contract_id = env.register_contract(None, ScavengerContract);
    let client = ScavengerContractClient::new(&env, &contract_id);

    let user = Address::generate(&env);
    let description = String::from_str(&env, "Side effects test");
    env.mock_all_auths();

    // Register participant
    client.register_participant(&user, &ParticipantRole::Recycler);

    // Submit waste
    let material = client.submit_material(&WasteType::Glass, &3000, &user, &description);

    // Get wastes
    let waste_ids = client.get_participant_wastes(&user);

    // Verify waste data is unchanged
    let waste_data = client.get_waste(&material.id).unwrap();
    assert_eq!(waste_data.id, material.id);
    assert_eq!(waste_data.submitter, user);
    assert_eq!(waste_data.waste_type, WasteType::Glass);
    assert_eq!(waste_data.weight, 3000);

    // Get wastes again - should still work
    let waste_ids_again = client.get_participant_wastes(&user);
    assert_eq!(waste_ids.len(), waste_ids_again.len());
}
