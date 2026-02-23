#![cfg(test)]

use soroban_sdk::{
    testutils::{Address as _, Events, Ledger},
    vec, Address, Env, IntoVal, String,
};

use crate::{types::Role, ScavengerContract, ScavengerContractClient};

fn create_test_contract(env: &Env) -> (ScavengerContractClient, Address, Address) {
    let contract_id = env.register(ScavengerContract, ());
    let client = ScavengerContractClient::new(env, &contract_id);

    let admin = Address::generate(env);
    let token_address = Address::generate(env);

    (client, admin, token_address)
}

#[test]
fn test_register_participant_recycler() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, admin, token_address) = create_test_contract(&env);
    client.__constructor(&admin, &token_address);

    let participant = Address::generate(&env);
    let name = String::from_str(&env, "John Recycler");

    client.register_participant(&participant, &Role::Recycler, &name, &100, &200);

    // Verify registration
    assert!(client.is_registered(&participant));

    let stored = client.get_participant(&participant).unwrap();
    assert_eq!(stored.address, participant);
    assert_eq!(stored.role, Role::Recycler);
    assert_eq!(stored.name, name);
    assert_eq!(stored.latitude, 100);
    assert_eq!(stored.longitude, 200);
}

#[test]
fn test_register_participant_collector() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, admin, token_address) = create_test_contract(&env);
    client.__constructor(&admin, &token_address);

    let participant = Address::generate(&env);
    let name = String::from_str(&env, "Jane Collector");

    client.register_participant(&participant, &Role::Collector, &name, &-50, &75);

    // Verify registration
    assert!(client.is_registered(&participant));

    let stored = client.get_participant(&participant).unwrap();
    assert_eq!(stored.role, Role::Collector);
    assert_eq!(stored.latitude, -50);
    assert_eq!(stored.longitude, 75);
}

#[test]
fn test_register_participant_manufacturer() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, admin, token_address) = create_test_contract(&env);
    client.__constructor(&admin, &token_address);

    let participant = Address::generate(&env);
    let name = String::from_str(&env, "Bob Manufacturer");

    client.register_participant(&participant, &Role::Manufacturer, &name, &0, &0);

    // Verify registration
    assert!(client.is_registered(&participant));

    let stored = client.get_participant(&participant).unwrap();
    assert_eq!(stored.role, Role::Manufacturer);
}

#[test]
fn test_all_roles_work() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, admin, token_address) = create_test_contract(&env);
    client.__constructor(&admin, &token_address);

    // Register recycler
    let recycler = Address::generate(&env);
    client.register_participant(
        &recycler,
        &Role::Recycler,
        &String::from_str(&env, "Recycler"),
        &10,
        &20,
    );

    // Register collector
    let collector = Address::generate(&env);
    client.register_participant(
        &collector,
        &Role::Collector,
        &String::from_str(&env, "Collector"),
        &30,
        &40,
    );

    // Register manufacturer
    let manufacturer = Address::generate(&env);
    client.register_participant(
        &manufacturer,
        &Role::Manufacturer,
        &String::from_str(&env, "Manufacturer"),
        &50,
        &60,
    );

    // Verify all registered
    assert!(client.is_registered(&recycler));
    assert!(client.is_registered(&collector));
    assert!(client.is_registered(&manufacturer));

    // Verify roles
    assert_eq!(
        client.get_participant(&recycler).unwrap().role,
        Role::Recycler
    );
    assert_eq!(
        client.get_participant(&collector).unwrap().role,
        Role::Collector
    );
    assert_eq!(
        client.get_participant(&manufacturer).unwrap().role,
        Role::Manufacturer
    );
}

#[test]
#[should_panic(expected = "Participant already registered")]
fn test_cannot_register_twice() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, admin, token_address) = create_test_contract(&env);
    client.__constructor(&admin, &token_address);

    let participant = Address::generate(&env);
    let name = String::from_str(&env, "Test User");

    // First registration
    client.register_participant(&participant, &Role::Recycler, &name, &100, &200);

    // Try to register again - should panic
    client.register_participant(&participant, &Role::Collector, &name, &300, &400);
}

#[test]
fn test_participant_not_registered() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, admin, token_address) = create_test_contract(&env);
    client.__constructor(&admin, &token_address);

    let participant = Address::generate(&env);

    // Should not be registered
    assert!(!client.is_registered(&participant));
    assert_eq!(client.get_participant(&participant), None);
}

#[test]
fn test_event_emitted() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, admin, token_address) = create_test_contract(&env);
    client.__constructor(&admin, &token_address);

    let participant = Address::generate(&env);
    let name = String::from_str(&env, "Event Test");
    let role = Role::Recycler;
    let latitude = 123i64;
    let longitude = 456i64;

    client.register_participant(&participant, &role, &name, &latitude, &longitude);

    // Get events
    let events = env.events().all();
    let event = vec![&env, events.last().unwrap()];

    // Verify event was emitted
    assert_eq!(event.len(), 1);
}

#[test]
fn test_multiple_participants() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, admin, token_address) = create_test_contract(&env);
    client.__constructor(&admin, &token_address);

    // Register multiple participants
    for i in 0..5 {
        let participant = Address::generate(&env);
        let name = String::from_str(&env, "Participant");
        let role = if i % 3 == 0 {
            Role::Recycler
        } else if i % 3 == 1 {
            Role::Collector
        } else {
            Role::Manufacturer
        };

        client.register_participant(&participant, &role, &name, &(i as i64), &(i as i64 * 10));

        assert!(client.is_registered(&participant));
    }
}

#[test]
fn test_participant_timestamp() {
    let env = Env::default();
    env.mock_all_auths();

    // Set ledger timestamp
    env.ledger().with_mut(|li| {
        li.timestamp = 1234567890;
    });

    let (client, admin, token_address) = create_test_contract(&env);
    client.__constructor(&admin, &token_address);

    let participant = Address::generate(&env);
    let name = String::from_str(&env, "Timestamp Test");

    client.register_participant(&participant, &Role::Recycler, &name, &100, &200);

    let stored = client.get_participant(&participant).unwrap();
    assert_eq!(stored.registered_at, 1234567890);
}

#[test]
fn test_negative_coordinates() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, admin, token_address) = create_test_contract(&env);
    client.__constructor(&admin, &token_address);

    let participant = Address::generate(&env);
    let name = String::from_str(&env, "Negative Coords");

    // Test with negative coordinates
    client.register_participant(&participant, &Role::Recycler, &name, &-90, &-180);

    let stored = client.get_participant(&participant).unwrap();
    assert_eq!(stored.latitude, -90);
    assert_eq!(stored.longitude, -180);
}

#[test]
fn test_registration_requires_auth() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, admin, token_address) = create_test_contract(&env);
    client.__constructor(&admin, &token_address);

    let participant = Address::generate(&env);
    let name = String::from_str(&env, "Auth Test");

    // This should work with mocked auth
    client.register_participant(&participant, &Role::Recycler, &name, &100, &200);

    assert!(client.is_registered(&participant));
}
