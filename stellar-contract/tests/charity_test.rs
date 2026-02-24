#![cfg(test)]

use soroban_sdk::{testutils::Address as _, Address, Env};
use stellar_scavngr_contract::{ScavengerContract, ScavengerContractClient};

#[test]
fn test_initialize_admin() {
    let env = Env::default();
    let contract_id = env.register_contract(None, ScavengerContract);
    let client = ScavengerContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    env.mock_all_auths();

    client.initialize_admin(&admin);
    
    let stored_admin = client.get_admin();
    assert_eq!(stored_admin, admin);
}

#[test]
#[should_panic(expected = "Admin already initialized")]
fn test_initialize_admin_twice() {
    let env = Env::default();
    let contract_id = env.register_contract(None, ScavengerContract);
    let client = ScavengerContractClient::new(&env, &contract_id);

    let admin1 = Address::generate(&env);
    let admin2 = Address::generate(&env);
    env.mock_all_auths();

    client.initialize_admin(&admin1);
    client.initialize_admin(&admin2); // Should panic
}

#[test]
fn test_set_charity_contract() {
    let env = Env::default();
    let contract_id = env.register_contract(None, ScavengerContract);
    let client = ScavengerContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let charity = Address::generate(&env);
    env.mock_all_auths();

    // Initialize admin first
    client.initialize_admin(&admin);
    
    // Set charity contract
    client.set_charity_contract(&admin, &charity);
    
    // Verify charity address is set
    let stored_charity = client.get_charity_contract();
    assert!(stored_charity.is_some());
    assert_eq!(stored_charity.unwrap(), charity);
}

#[test]
#[should_panic(expected = "Unauthorized: caller is not admin")]
fn test_set_charity_contract_non_admin() {
    let env = Env::default();
    let contract_id = env.register_contract(None, ScavengerContract);
    let client = ScavengerContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let non_admin = Address::generate(&env);
    let charity = Address::generate(&env);
    env.mock_all_auths();

    // Initialize admin
    client.initialize_admin(&admin);
    
    // Try to set charity contract as non-admin (should panic)
    client.set_charity_contract(&non_admin, &charity);
}

#[test]
#[should_panic(expected = "Charity address cannot be the same as admin")]
fn test_set_charity_contract_same_as_admin() {
    let env = Env::default();
    let contract_id = env.register_contract(None, ScavengerContract);
    let client = ScavengerContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    env.mock_all_auths();

    // Initialize admin
    client.initialize_admin(&admin);
    
    // Try to set charity contract to same address as admin (should panic)
    client.set_charity_contract(&admin, &admin);
}

#[test]
fn test_get_charity_contract_not_set() {
    let env = Env::default();
    let contract_id = env.register_contract(None, ScavengerContract);
    let client = ScavengerContractClient::new(&env, &contract_id);

    // Get charity contract before it's set
    let charity = client.get_charity_contract();
    assert!(charity.is_none());
}

#[test]
fn test_charity_contract_update() {
    let env = Env::default();
    let contract_id = env.register_contract(None, ScavengerContract);
    let client = ScavengerContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let charity1 = Address::generate(&env);
    let charity2 = Address::generate(&env);
    env.mock_all_auths();

    // Initialize admin
    client.initialize_admin(&admin);
    
    // Set initial charity contract
    client.set_charity_contract(&admin, &charity1);
    assert_eq!(client.get_charity_contract().unwrap(), charity1);
    
    // Update charity contract
    client.set_charity_contract(&admin, &charity2);
    assert_eq!(client.get_charity_contract().unwrap(), charity2);
}

#[test]
fn test_charity_donations_workflow() {
    let env = Env::default();
    let contract_id = env.register_contract(None, ScavengerContract);
    let client = ScavengerContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let charity = Address::generate(&env);
    env.mock_all_auths();

    // Initialize admin
    client.initialize_admin(&admin);
    
    // Set charity contract
    client.set_charity_contract(&admin, &charity);
    
    // Verify donations can work after setting
    let stored_charity = client.get_charity_contract();
    assert!(stored_charity.is_some());
    assert_eq!(stored_charity.unwrap(), charity);
    
    // This demonstrates that the charity address is properly stored
    // and can be retrieved for donation operations
}
