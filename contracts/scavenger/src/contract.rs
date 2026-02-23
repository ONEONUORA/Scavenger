use soroban_sdk::{contract, contractimpl, Address, Env};

use crate::storage::Storage;

// Default percentages
const DEFAULT_COLLECTOR_PERCENTAGE: u32 = 5;
const DEFAULT_OWNER_PERCENTAGE: u32 = 50;
const INITIAL_COUNTER_VALUE: u64 = 1;

#[contract]
pub struct ScavengerContract;

#[contractimpl]
impl ScavengerContract {
    /// Initialize the contract with admin and token address
    /// Sets default percentages: collector 5%, owner 50%
    /// Initializes counter to 1
    /// Can only be called once
    pub fn __constructor(env: &Env, admin: Address, token_address: Address) {
        // Prevent re-initialization
        if Storage::is_initialized(env) {
            panic!("Contract already initialized");
        }

        // Validate admin address
        admin.require_auth();

        // Set admin
        Storage::set_admin(env, &admin);

        // Set token address
        Storage::set_token_address(env, &token_address);

        // Set default percentages
        Storage::set_collector_percentage(env, DEFAULT_COLLECTOR_PERCENTAGE);
        Storage::set_owner_percentage(env, DEFAULT_OWNER_PERCENTAGE);

        // Initialize counter to 1
        Storage::set_counter(env, INITIAL_COUNTER_VALUE);

        // Initialize total earned to 0
        Storage::set_total_earned(env, 0);

        // Mark as initialized
        Storage::set_initialized(env);
    }

    /// Get the current admin address
    pub fn get_admin(env: &Env) -> Address {
        Storage::get_admin(env).expect("Admin not set")
    }

    /// Get the scavenger token address
    pub fn get_token_address(env: &Env) -> Address {
        Storage::get_token_address(env).expect("Token address not set")
    }

    /// Get the charity contract address
    pub fn get_charity_address(env: &Env) -> Option<Address> {
        Storage::get_charity_address(env)
    }

    /// Get the collector percentage
    pub fn get_collector_percentage(env: &Env) -> u32 {
        Storage::get_collector_percentage(env).expect("Collector percentage not set")
    }

    /// Get the owner percentage
    pub fn get_owner_percentage(env: &Env) -> u32 {
        Storage::get_owner_percentage(env).expect("Owner percentage not set")
    }

    /// Get the total tokens earned
    pub fn get_total_earned(env: &Env) -> i128 {
        Storage::get_total_earned(env)
    }

    /// Get the current counter value
    pub fn get_counter(env: &Env) -> u64 {
        Storage::get_counter(env)
    }

    /// Check if contract is initialized
    pub fn is_initialized(env: &Env) -> bool {
        Storage::is_initialized(env)
    }

    /// Set charity address (admin only)
    pub fn set_charity_address(env: &Env, admin: Address, charity_address: Address) {
        Self::require_admin(env, &admin);
        Storage::set_charity_address(env, &charity_address);
    }

    /// Update the collector percentage (admin only)
    pub fn update_collector_percentage(env: &Env, admin: Address, new_percentage: u32) {
        Self::require_admin(env, &admin);

        let owner_pct = Storage::get_owner_percentage(env).expect("Owner percentage not set");
        assert!(
            new_percentage + owner_pct <= 100,
            "Total percentages cannot exceed 100"
        );

        Storage::set_collector_percentage(env, new_percentage);
    }

    /// Update the owner percentage (admin only)
    pub fn update_owner_percentage(env: &Env, admin: Address, new_percentage: u32) {
        Self::require_admin(env, &admin);

        let collector_pct = Storage::get_collector_percentage(env)
            .expect("Collector percentage not set");
        assert!(
            collector_pct + new_percentage <= 100,
            "Total percentages cannot exceed 100"
        );

        Storage::set_owner_percentage(env, new_percentage);
    }

    // Private helper function to require admin authentication
    fn require_admin(env: &Env, admin: &Address) {
        let stored_admin = Storage::get_admin(env).expect("Admin not set");
        assert!(
            stored_admin == *admin,
            "Only admin can perform this action"
        );
        admin.require_auth();
    }
}
