use soroban_sdk::{contract, contractimpl, token, Address, Env, Vec};

use crate::events;
use crate::storage::Storage;
use crate::types::{GlobalMetrics, Incentive, Material, Participant, Role, WasteTransfer, WasteType};

#[contract]
pub struct ScavengerContract;

#[contractimpl]
impl ScavengerContract {
    /// Initialize the contract with admin and configuration
    pub fn __constructor(
        env: &Env,
        admin: Address,
        token_address: Address,
        charity_address: Address,
        collector_percentage: u32,
        owner_percentage: u32,
    ) {
        // Validate percentages
        assert!(
            collector_percentage + owner_percentage <= 100,
            "Total percentages cannot exceed 100"
        );

        // Set admin
        Storage::set_admin(env, &admin);

        // Set configuration
        Storage::set_token_address(env, &token_address);
        Storage::set_charity_address(env, &charity_address);
        Storage::set_collector_percentage(env, collector_percentage);
        Storage::set_owner_percentage(env, owner_percentage);
        Storage::set_total_earned(env, 0);
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
    pub fn get_charity_address(env: &Env) -> Address {
        Storage::get_charity_address(env).expect("Charity address not set")
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

    // ========== Global Metrics (Issue #55) ==========

    /// Retrieve global contract metrics (total wastes and total tokens earned)
    pub fn get_metrics(env: &Env) -> GlobalMetrics {
        // We use the Material counter (MAT_CNT) to determine total waste items logged.
        // This calculates the metric efficiently from storage without iteration.
        let waste_count = env.storage().instance().get(&soroban_sdk::symbol_short!("MAT_CNT")).unwrap_or(0);
        
        GlobalMetrics {
            total_wastes_count: waste_count,
            total_tokens_earned: Storage::get_total_earned(env),
        }
    }

    // ===============================================

    /// Update the token address (admin only)
    pub fn update_token_address(env: &Env, admin: Address, new_address: Address) {
        Self::require_admin(env, &admin);
        Storage::set_token_address(env, &new_address);
    }

    /// Update the charity address (admin only)
    pub fn update_charity_address(env: &Env, admin: Address, new_address: Address) {
        Self::require_admin(env, &admin);
        Storage::set_charity_address(env, &new_address);
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

    /// Update both percentages at once (admin only)
    pub fn update_percentages(
        env: &Env,
        admin: Address,
        collector_percentage: u32,
        owner_percentage: u32,
    ) {
        Self::require_admin(env, &admin);
        
        assert!(
            collector_percentage + owner_percentage <= 100,
            "Total percentages cannot exceed 100"
        );
        
        Storage::set_collector_percentage(env, collector_percentage);
        Storage::set_owner_percentage(env, owner_percentage);
    }

    /// Transfer admin rights to a new address (admin only)
    pub fn transfer_admin(env: &Env, current_admin: Address, new_admin: Address) {
        Self::require_admin(env, &current_admin);
        Storage::set_admin(env, &new_admin);
    }

    /// Register a participant
    pub fn register_participant(
        env: &Env,
        address: Address,
        role: Role,
        name: soroban_sdk::String,
        latitude: i64,
        longitude: i64,
    ) -> Participant {
        address.require_auth();

        assert!(
            !Storage::is_participant_registered(env, &address),
            "Participant already registered"
        );

        let participant = Participant {
            address: address.clone(),
            role,
            name,
            latitude,
            longitude,
            registered_at: env.ledger().timestamp(),
        };

        Storage::set_participant(env, &address, &participant);
        participant
    }

    /// Get participant information
    pub fn get_participant(env: &Env, address: Address) -> Option<Participant> {
        Storage::get_participant(env, &address)
    }

    /// Check if participant is registered
    pub fn is_participant_registered(env: &Env, address: Address) -> bool {
        Storage::is_participant_registered(env, &address)
    }

    /// Create a new incentive (manufacturers only)
    pub fn create_incentive(
        env: &Env,
        rewarder: Address,
        waste_type: WasteType,
        reward_points: u64,
        total_budget: u64,
    ) -> Incentive {
        // Require authentication
        rewarder.require_auth();

        // Check caller is registered
        assert!(
            Storage::is_participant_registered(env, &rewarder),
            "Rewarder not registered"
        );

        // Get participant and verify role
        let participant = Storage::get_participant(env, &rewarder)
            .expect("Rewarder not found");

        assert!(
            participant.role.can_manufacture(),
            "Only manufacturers can create incentives"
        );

        // Generate incentive ID
        let incentive_id = Storage::next_incentive_id(env);

        // Create Incentive struct
        let incentive = Incentive::new(
            incentive_id,
            rewarder.clone(),
            waste_type,
            reward_points,
            total_budget,
            env.ledger().timestamp(),
        );

        // Store in all three incentive maps
        Storage::set_incentive(env, incentive_id, &incentive);
        Storage::add_incentive_to_rewarder(env, &rewarder, incentive_id);
        Storage::add_incentive_to_waste_type(env, waste_type, incentive_id);

        // Emit IncentiveSet event
        events::emit_incentive_set(
            env,
            incentive_id,
            &rewarder,
            waste_type,
            reward_points,
            total_budget,
        );

        incentive
    }

    /// Get incentive by ID
    pub fn get_incentive_by_id(env: &Env, incentive_id: u64) -> Option<Incentive> {
        Storage::get_incentive(env, incentive_id)
    }

    /// Check if incentive exists
    pub fn incentive_exists(env: &Env, incentive_id: u64) -> bool {
        Storage::incentive_exists(env, incentive_id)
    }

    /// Get all incentive IDs created by a specific rewarder (manufacturer)
    pub fn get_incentives_by_rewarder(env: &Env, rewarder: Address) -> Vec<u64> {
        Storage::get_incentives_by_rewarder(env, &rewarder)
    }

    /// Get all incentive IDs for a specific waste type
    pub fn get_incentives_by_waste_type(env: &Env, waste_type: WasteType) -> Vec<u64> {
        Storage::get_incentives_by_waste_type(env, waste_type)
    }

    /// Submit material for recycling
    pub fn submit_material(
        env: &Env,
        submitter: Address,
        waste_type: WasteType,
        weight: u64,
    ) -> Material {
        submitter.require_auth();

        assert!(
            Storage::is_participant_registered(env, &submitter),
            "Submitter not registered"
        );

        let material_id = Storage::next_material_id(env);
        let material = Material::new(
            material_id,
            waste_type,
            weight,
            submitter,
            env.ledger().timestamp(),
        );

        Storage::set_material(env, material_id, &material);
        material
    }

    /// Transfer waste to another participant
    pub fn transfer_waste(
        env: &Env,
        waste_id: u64,
        from: Address,
        to: Address,
    ) {
        from.require_auth();

        assert!(
            Storage::is_participant_registered(env, &from),
            "Sender not registered"
        );
        assert!(
            Storage::is_participant_registered(env, &to),
            "Receiver not registered"
        );

        let mut material = Storage::get_material(env, waste_id)
            .expect("Material not found");

        assert!(
            material.current_owner == from,
            "Only current owner can transfer"
        );

        // Update material owner
        material.current_owner = to.clone();
        Storage::set_material(env, waste_id, &material);

        // Record transfer
        let transfer = WasteTransfer::new(
            waste_id,
            from,
            to,
            env.ledger().timestamp(),
        );
        Storage::add_transfer(env, waste_id, &transfer);
    }

    /// Get transfer history for a waste item
    pub fn get_transfer_history(env: &Env, waste_id: u64) -> Vec<WasteTransfer> {
        Storage::get_transfer_history(env, waste_id)
    }

    /// Distribute token rewards through the supply chain
    pub fn distribute_rewards(
        env: &Env,
        waste_id: u64,
        incentive_id: u64,
        manufacturer: Address,
    ) -> i128 {
        manufacturer.require_auth();

        // Get waste material
        let material = Storage::get_material(env, waste_id)
            .expect("Material not found");

        assert!(material.verified, "Material must be verified");

        // Get manufacturer incentive
        let incentive = Storage::get_incentive(env, incentive_id)
            .expect("Incentive not found");

        assert!(
            incentive.rewarder == manufacturer,
            "Only incentive creator can distribute rewards"
        );

        assert!(
            incentive.waste_type == material.waste_type,
            "Waste type mismatch"
        );

        assert!(incentive.active, "Incentive not active");

        // Calculate total reward (incentive * weight in kg)
        let weight_kg = material.weight / 1000;
        let total_reward = (incentive.reward_points as i128) * (weight_kg as i128);

        assert!(
            (total_reward as u64) <= incentive.remaining_budget,
            "Insufficient incentive budget"
        );

        // Get waste transfer history
        let transfers = Storage::get_transfer_history(env, waste_id);

        // Get configuration
        let collector_pct = Storage::get_collector_percentage(env)
            .expect("Collector percentage not set");
        let owner_pct = Storage::get_owner_percentage(env)
            .expect("Owner percentage not set");

        let token_address = Storage::get_token_address(env)
            .expect("Token address not set");
        let token_client = token::Client::new(env, &token_address);

        // Calculate collector shares (5% each from total)
        let collector_share = (total_reward * (collector_pct as i128)) / 100;
        
        // Calculate owner shares (50% of total)
        let owner_share = (total_reward * (owner_pct as i128)) / 100;

        let mut total_distributed: i128 = 0;

        // Iterate through transfer history and reward collectors
        for transfer in transfers.iter() {
            let participant = Storage::get_participant(env, &transfer.to);
            if let Some(p) = participant {
                if matches!(p.role, Role::Collector) {
                    // Transfer tokens to collector
                    token_client.transfer(&manufacturer, &transfer.to, &collector_share);
                    
                    // Update participant statistics
                    Storage::add_earnings(env, &transfer.to, collector_share);
                    
                    // Emit TokensRewarded event
                    events::emit_tokens_rewarded(env, waste_id, &transfer.to, collector_share);
                    
                    total_distributed += collector_share;
                }
            }
        }

        // Reward the original owner (submitter) with their share
        token_client.transfer(&manufacturer, &material.submitter, &owner_share);
        Storage::add_earnings(env, &material.submitter, owner_share);
        events::emit_tokens_rewarded(env, waste_id, &material.submitter, owner_share);
        total_distributed += owner_share;

        // Recycler gets remaining amount
        let recycler_amount = total_reward - total_distributed;
        if recycler_amount > 0 {
            token_client.transfer(&manufacturer, &material.current_owner, &recycler_amount);
            Storage::add_earnings(env, &material.current_owner, recycler_amount);
            events::emit_tokens_rewarded(env, waste_id, &material.current_owner, recycler_amount);
        }

        // Update incentive budget
        let mut updated_incentive = incentive;
        updated_incentive.remaining_budget -= total_reward as u64;
        if updated_incentive.remaining_budget == 0 {
            updated_incentive.active = false;
        }
        Storage::set_incentive(env, incentive_id, &updated_incentive);

        // Update total earned
        Storage::add_to_total_earned(env, total_reward);

        total_reward
    }

    /// Get participant statistics
    pub fn get_participant_stats(env: &Env, address: Address) -> crate::types::ParticipantStats {
        Storage::get_stats(env, &address)
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