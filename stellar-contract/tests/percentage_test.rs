#![cfg(test)]

use soroban_sdk::{testutils::Address as _, Address, Env};
use stellar_scavngr_contract::{ScavengerContract, ScavengerContractClient};

#[test]
fn test_set_percentages() {
    let env = Env::default();
    let contract_id = env.register_contract(None, ScavengerContract);
    let client = ScavengerContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    env.mock_all_auths();

    // Initialize admin
    client.initialize_admin(&admin);
    
    // Set percentages
    client.set_percentages(&admin, &30, &20);
    
    // Verify percentages are set
    assert_eq!(client.get_collector_percentage(), Some(30));
    assert_eq!(client.get_owner_percentage(), Some(20));
}

#[test]
#[should_panic(expected = "Total percentages cannot exceed 100")]
fn test_set_percentages_invalid_sum() {
    let env = Env::default();
    let contract_id = env.register_contract(None, ScavengerContract);
    let client = ScavengerContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    env.mock_all_auths();

    // Initialize admin
    client.initialize_admin(&admin);
    
    // Try to set invalid percentages (60 + 50 = 110 > 100)
    client.set_percentages(&admin, &60, &50);
}

#[test]
fn test_set_percentages_exactly_100() {
    let env = Env::default();
    let contract_id = env.register_contract(None, ScavengerContract);
    let client = ScavengerContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    env.mock_all_auths();

    // Initialize admin
    client.initialize_admin(&admin);
    
    // Set percentages that sum to exactly 100
    client.set_percentages(&admin, &60, &40);
    
    // Verify percentages are set
    assert_eq!(client.get_collector_percentage(), Some(60));
    assert_eq!(client.get_owner_percentage(), Some(40));
}

#[test]
#[should_panic(expected = "Unauthorized: caller is not admin")]
fn test_set_percentages_non_admin() {
    let env = Env::default();
    let contract_id = env.register_contract(None, ScavengerContract);
    let client = ScavengerContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let non_admin = Address::generate(&env);
    env.mock_all_auths();

    // Initialize admin
    client.initialize_admin(&admin);
    
    // Try to set percentages as non-admin (should panic)
    client.set_percentages(&non_admin, &30, &20);
}

#[test]
fn test_set_collector_percentage() {
    let env = Env::default();
    let contract_id = env.register_contract(None, ScavengerContract);
    let client = ScavengerContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    env.mock_all_auths();

    // Initialize admin
    client.initialize_admin(&admin);
    
    // Set initial percentages
    client.set_percentages(&admin, &30, &20);
    
    // Update collector percentage
    client.set_collector_percentage(&admin, &40);
    
    // Verify collector percentage updated, owner unchanged
    assert_eq!(client.get_collector_percentage(), Some(40));
    assert_eq!(client.get_owner_percentage(), Some(20));
}

#[test]
#[should_panic(expected = "Total percentages cannot exceed 100")]
fn test_set_collector_percentage_invalid() {
    let env = Env::default();
    let contract_id = env.register_contract(None, ScavengerContract);
    let client = ScavengerContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    env.mock_all_auths();

    // Initialize admin
    client.initialize_admin(&admin);
    
    // Set initial percentages
    client.set_percentages(&admin, &30, &20);
    
    // Try to set collector percentage that would exceed 100 (85 + 20 = 105)
    client.set_collector_percentage(&admin, &85);
}

#[test]
fn test_set_owner_percentage() {
    let env = Env::default();
    let contract_id = env.register_contract(None, ScavengerContract);
    let client = ScavengerContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    env.mock_all_auths();

    // Initialize admin
    client.initialize_admin(&admin);
    
    // Set initial percentages
    client.set_percentages(&admin, &30, &20);
    
    // Update owner percentage
    client.set_owner_percentage(&admin, &25);
    
    // Verify owner percentage updated, collector unchanged
    assert_eq!(client.get_collector_percentage(), Some(30));
    assert_eq!(client.get_owner_percentage(), Some(25));
}

#[test]
#[should_panic(expected = "Total percentages cannot exceed 100")]
fn test_set_owner_percentage_invalid() {
    let env = Env::default();
    let contract_id = env.register_contract(None, ScavengerContract);
    let client = ScavengerContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    env.mock_all_auths();

    // Initialize admin
    client.initialize_admin(&admin);
    
    // Set initial percentages
    client.set_percentages(&admin, &30, &20);
    
    // Try to set owner percentage that would exceed 100 (30 + 75 = 105)
    client.set_owner_percentage(&admin, &75);
}

#[test]
fn test_get_percentages_not_set() {
    let env = Env::default();
    let contract_id = env.register_contract(None, ScavengerContract);
    let client = ScavengerContractClient::new(&env, &contract_id);

    // Get percentages before they're set
    assert_eq!(client.get_collector_percentage(), None);
    assert_eq!(client.get_owner_percentage(), None);
}

#[test]
fn test_update_percentages_multiple_times() {
    let env = Env::default();
    let contract_id = env.register_contract(None, ScavengerContract);
    let client = ScavengerContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    env.mock_all_auths();

    // Initialize admin
    client.initialize_admin(&admin);
    
    // Set initial percentages
    client.set_percentages(&admin, &30, &20);
    assert_eq!(client.get_collector_percentage(), Some(30));
    assert_eq!(client.get_owner_percentage(), Some(20));
    
    // Update percentages
    client.set_percentages(&admin, &35, &25);
    assert_eq!(client.get_collector_percentage(), Some(35));
    assert_eq!(client.get_owner_percentage(), Some(25));
    
    // Update again
    client.set_percentages(&admin, &40, &30);
    assert_eq!(client.get_collector_percentage(), Some(40));
    assert_eq!(client.get_owner_percentage(), Some(30));
}

#[test]
fn test_set_zero_percentages() {
    let env = Env::default();
    let contract_id = env.register_contract(None, ScavengerContract);
    let client = ScavengerContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    env.mock_all_auths();

    // Initialize admin
    client.initialize_admin(&admin);
    
    // Set zero percentages (valid)
    client.set_percentages(&admin, &0, &0);
    
    // Verify percentages are set to zero
    assert_eq!(client.get_collector_percentage(), Some(0));
    assert_eq!(client.get_owner_percentage(), Some(0));
}

#[test]
fn test_set_one_percentage_to_100() {
    let env = Env::default();
    let contract_id = env.register_contract(None, ScavengerContract);
    let client = ScavengerContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    env.mock_all_auths();

    // Initialize admin
    client.initialize_admin(&admin);
    
    // Set collector to 100%, owner to 0%
    client.set_percentages(&admin, &100, &0);
    assert_eq!(client.get_collector_percentage(), Some(100));
    assert_eq!(client.get_owner_percentage(), Some(0));
    
    // Update to owner 100%, collector 0%
    client.set_percentages(&admin, &0, &100);
    assert_eq!(client.get_collector_percentage(), Some(0));
    assert_eq!(client.get_owner_percentage(), Some(100));
}

#[test]
fn test_individual_percentage_updates() {
    let env = Env::default();
    let contract_id = env.register_contract(None, ScavengerContract);
    let client = ScavengerContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    env.mock_all_auths();

    // Initialize admin
    client.initialize_admin(&admin);
    
    // Set initial percentages
    client.set_percentages(&admin, &30, &20);
    
    // Update collector only
    client.set_collector_percentage(&admin, &35);
    assert_eq!(client.get_collector_percentage(), Some(35));
    assert_eq!(client.get_owner_percentage(), Some(20));
    
    // Update owner only
    client.set_owner_percentage(&admin, &25);
    assert_eq!(client.get_collector_percentage(), Some(35));
    assert_eq!(client.get_owner_percentage(), Some(25));
}

#[test]
fn test_reward_calculation_uses_new_percentages() {
    let env = Env::default();
    let contract_id = env.register_contract(None, ScavengerContract);
    let client = ScavengerContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    env.mock_all_auths();

    // Initialize admin
    client.initialize_admin(&admin);
    
    // Set initial percentages
    client.set_percentages(&admin, &30, &20);
    
    // Simulate reward calculation
    let total_reward = 1000u32;
    let collector_share = (total_reward * client.get_collector_percentage().unwrap()) / 100;
    let owner_share = (total_reward * client.get_owner_percentage().unwrap()) / 100;
    
    assert_eq!(collector_share, 300); // 30% of 1000
    assert_eq!(owner_share, 200);     // 20% of 1000
    
    // Update percentages
    client.set_percentages(&admin, &40, &30);
    
    // Recalculate with new percentages
    let new_collector_share = (total_reward * client.get_collector_percentage().unwrap()) / 100;
    let new_owner_share = (total_reward * client.get_owner_percentage().unwrap()) / 100;
    
    assert_eq!(new_collector_share, 400); // 40% of 1000
    assert_eq!(new_owner_share, 300);     // 30% of 1000
}
