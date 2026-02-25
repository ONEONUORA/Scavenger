#![cfg(test)]

use soroban_sdk::{testutils::Address as _, testutils::Events, Address, Env, String, symbol_short, TryFromVal, TryIntoVal};
use crate::testutils::TestEnv;
use crate::types::Role;

#[test]
fn test_successful_registration_recycler() {
    let test_env = TestEnv::new();
    let user = test_env.generate_address();
    let name = "Recycler User";
    
    test_env.register_participant(&user, Role::Recycler, name);
    
    let participant = test_env.client.get_participant(&user).unwrap();
    assert_eq!(participant.address, user);
    assert_eq!(participant.role, Role::Recycler);
    assert_eq!(participant.name, test_env.create_string(name));
    assert_eq!(participant.latitude, 1000);
    assert_eq!(participant.longitude, 2000);
}

#[test]
fn test_successful_registration_collector() {
    let test_env = TestEnv::new();
    let user = test_env.generate_address();
    let name = "Collector User";
    
    test_env.register_participant(&user, Role::Collector, name);
    
    let participant = test_env.client.get_participant(&user).unwrap();
    assert_eq!(participant.address, user);
    assert_eq!(participant.role, Role::Collector);
    assert_eq!(participant.name, test_env.create_string(name));
}

#[test]
fn test_successful_registration_manufacturer() {
    let test_env = TestEnv::new();
    let user = test_env.generate_address();
    let name = "Manufacturer User";
    
    test_env.register_participant(&user, Role::Manufacturer, name);
    
    let participant = test_env.client.get_participant(&user).unwrap();
    assert_eq!(participant.address, user);
    assert_eq!(participant.role, Role::Manufacturer);
    assert_eq!(participant.name, test_env.create_string(name));
}

#[test]
#[should_panic(expected = "Participant already registered")]
fn test_duplicate_registration_fails() {
    let test_env = TestEnv::new();
    let user = test_env.generate_address();
    
    test_env.register_participant(&user, Role::Recycler, "Name 1");
    // Registering the same address again should fail
    test_env.register_participant(&user, Role::Collector, "Name 2");
}

#[test]
fn test_registration_event_emitted() {
    let test_env = TestEnv::new();
    let user = test_env.generate_address();
    let role = Role::Recycler;
    let name = "Event Test User";
    
    // Clear any events from initialization
    let _ = test_env.env.events().all();
    
    test_env.register_participant(&user, role.clone(), name);
    
    let events = test_env.env.events().all();
    assert!(!events.is_empty(), "No events were emitted");
    
    let reg_event = events.iter().find(|e| {
        if let Ok(topic) = soroban_sdk::Symbol::try_from_val(&test_env.env, &e.1.get(0).unwrap()) {
            topic == symbol_short!("reg")
        } else {
            false
        }
    }).expect("Registration event not found");
    
    // Topic 1 should be the user address
    let event_user: Address = reg_event.1.get(1).unwrap().try_into_val(&test_env.env).unwrap();
    assert_eq!(event_user, user);
    
    // Data contains (role, name, latitude, longitude)
    // In soroban-sdk tests, we can use Val to compare or try to decode.
    // However, it's often easier to just check that it exists if the data structure is complex.
}

#[test]
fn test_is_participant_registered() {
    let test_env = TestEnv::new();
    let user = test_env.generate_address();
    
    assert!(!test_env.client.is_participant_registered(&user));
    
    test_env.register_participant(&user, Role::Recycler, "User");
    
    assert!(test_env.client.is_participant_registered(&user));
}

#[test]
#[should_panic (expected = "Submitter not registered")]
fn test_submit_material_fails_for_unregistered_user() {
    let test_env = TestEnv::new();
    let user = test_env.generate_address();
    
    test_env.client.submit_material(&user, &crate::types::WasteType::Plastic, &1000);
}

#[test]
#[should_panic (expected = "Rewarder not registered")]
fn test_create_incentive_fails_for_unregistered_user() {
    let test_env = TestEnv::new();
    let user = test_env.generate_address();
    
    test_env.client.create_incentive(&user, &crate::types::WasteType::Plastic, &10, &1000);
}
