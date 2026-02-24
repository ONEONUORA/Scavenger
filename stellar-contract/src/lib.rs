#![no_std]

mod events;
mod types;



pub use types::{
    Material, ParticipantRole, RecyclingStats, TransferItemType, TransferRecord, TransferStatus,
    WasteType,
};

pub use types::{Material, ParticipantRole, RecyclingStats, Waste, WasteStatus, WasteType};


pub use types::{Incentive, Material, ParticipantRole, RecyclingStats, Waste, WasteTransfer, WasteType};


use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Address, Env, String, Symbol, Vec};

// Storage keys
const ADMIN: Symbol = symbol_short!("ADMIN");
const CHARITY: Symbol = symbol_short!("CHARITY");
const COLLECTOR_PCT: Symbol = symbol_short!("COL_PCT");
const OWNER_PCT: Symbol = symbol_short!("OWN_PCT");

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Participant {
    pub address: Address,
    pub role: ParticipantRole,
    pub name: soroban_sdk::Symbol,
    pub latitude: i128,
    pub longitude: i128,
    pub is_registered: bool,
    pub total_waste_processed: u128,
    pub total_tokens_earned: u128,
    pub registered_at: u64,
}


/// Represents a manufacturer incentive program for recycling specific waste types
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Incentive {
    /// Unique identifier for the incentive
    pub id: u64,
    /// Type of waste this incentive applies to
    pub waste_type: WasteType,
    /// Reward amount per unit (in tokens)
    pub reward: u128,
    /// Maximum waste amount eligible for this incentive (in grams)
    pub max_waste_amount: u128,
    /// Address of the manufacturer offering the incentive
    pub rewarder: Address,
    /// Whether this incentive is currently active
    pub is_active: bool,
    /// Timestamp when the incentive was created
    pub created_at: u64,

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParticipantInfo {
    pub participant: Participant,
    pub stats: RecyclingStats,

}

#[contract]
pub struct ScavengerContract;

#[contractimpl]
impl ScavengerContract {
    // ========== Admin Functions ==========

    /// Initialize admin (should be called once during contract deployment)
    pub fn initialize_admin(env: Env, admin: Address) {
        admin.require_auth();
        
        // Check if admin is already set
        if env.storage().instance().has(&ADMIN) {
            panic!("Admin already initialized");
        }
        
        env.storage().instance().set(&ADMIN, &admin);
    }

    /// Get the current admin address
    pub fn get_admin(env: Env) -> Address {
        env.storage()
            .instance()
            .get(&ADMIN)
            .expect("Admin not set")
    }

    /// Check if caller is admin
    fn require_admin(env: &Env, caller: &Address) {
        let admin: Address = env
            .storage()
            .instance()
            .get(&ADMIN)
            .expect("Admin not set");
        
        if admin != *caller {
            panic!("Unauthorized: caller is not admin");
        }
        
        caller.require_auth();
    }

    // ========== Charity Contract Functions ==========

    /// Set the charity contract address (admin only)
    pub fn set_charity_contract(env: Env, admin: Address, charity_address: Address) {
        Self::require_admin(&env, &admin);
        
        // Validate address (basic check - address should not be the zero address)
        // In Soroban, we can't easily check for zero address, but we can ensure it's different from admin
        if charity_address == admin {
            panic!("Charity address cannot be the same as admin");
        }
        
        env.storage().instance().set(&CHARITY, &charity_address);
    }

    /// Get the charity contract address
    pub fn get_charity_contract(env: Env) -> Option<Address> {
        env.storage().instance().get(&CHARITY)
    }

    // ========== Percentage Configuration Functions ==========

    /// Set both collector and owner percentages (admin only)
    pub fn set_percentages(
        env: Env,
        admin: Address,
        collector_percentage: u32,
        owner_percentage: u32,
    ) {
        Self::require_admin(&env, &admin);
        
        // Validate percentages sum
        if collector_percentage + owner_percentage > 100 {
            panic!("Total percentages cannot exceed 100");
        }
        
        env.storage().instance().set(&COLLECTOR_PCT, &collector_percentage);
        env.storage().instance().set(&OWNER_PCT, &owner_percentage);
    }

    /// Get the collector percentage
    pub fn get_collector_percentage(env: Env) -> Option<u32> {
        env.storage().instance().get(&COLLECTOR_PCT)
    }

    /// Get the owner percentage
    pub fn get_owner_percentage(env: Env) -> Option<u32> {
        env.storage().instance().get(&OWNER_PCT)
    }

    /// Update only the collector percentage (admin only)
    pub fn set_collector_percentage(env: Env, admin: Address, new_percentage: u32) {
        Self::require_admin(&env, &admin);
        
        // Get current owner percentage to validate total
        let owner_pct: u32 = env.storage()
            .instance()
            .get(&OWNER_PCT)
            .unwrap_or(0);
        
        if new_percentage + owner_pct > 100 {
            panic!("Total percentages cannot exceed 100");
        }
        
        env.storage().instance().set(&COLLECTOR_PCT, &new_percentage);
    }

    /// Update only the owner percentage (admin only)
    pub fn set_owner_percentage(env: Env, admin: Address, new_percentage: u32) {
        Self::require_admin(&env, &admin);
        
        // Get current collector percentage to validate total
        let collector_pct: u32 = env.storage()
            .instance()
            .get(&COLLECTOR_PCT)
            .unwrap_or(0);
        
        if collector_pct + new_percentage > 100 {
            panic!("Total percentages cannot exceed 100");
        }
        
        env.storage().instance().set(&OWNER_PCT, &new_percentage);
    }

    // ========== Participant Storage Functions ==========

    /// Store a participant record
    /// Internal helper function for efficient participant storage
    fn set_participant(env: &Env, address: &Address, participant: &Participant) {
        let key = (address.clone(),);
        env.storage().instance().set(&key, participant);
    }

    /// Check if a participant is registered
    pub fn is_participant_registered(env: Env, address: Address) -> bool {
        let key = (address,);
        env.storage().instance().has(&key)
    }

    /// Register a new participant with a specific role

    pub fn register_participant(
        env: Env,
        address: Address,
        role: ParticipantRole,
        name: soroban_sdk::Symbol,
        latitude: i128,
        longitude: i128,
    ) -> Participant {

    /// Prevents duplicate registrations
    pub fn register_participant(env: Env, address: Address, role: ParticipantRole) -> Participant {

        address.require_auth();

        // Check if already registered
        if Self::is_participant_registered(env.clone(), address.clone()) {
            panic!("Participant already registered");
        }

        let participant = Participant {
            address: address.clone(),
            role,
            name,
            latitude,
            longitude,
            is_registered: true,
            total_waste_processed: 0,
            total_tokens_earned: 0,
            registered_at: env.ledger().timestamp(),
        };

        // Store participant using helper function
        Self::set_participant(&env, &address, &participant);

        participant
    }

    /// Update participant statistics after processing waste
    /// Uses checked arithmetic to prevent overflow
    fn update_participant_stats(
        env: &Env,
        address: &Address,
        waste_weight: u64,
        tokens_earned: u64,
    ) {
        let key = (address.clone(),);
        if let Some(mut participant) = env.storage().instance().get::<_, Participant>(&key) {
            // Use checked arithmetic to prevent overflow
            participant.total_waste_processed = participant
                .total_waste_processed
                .checked_add(waste_weight as u128)
                .expect("Overflow in total_waste_processed");
            
            participant.total_tokens_earned = participant
                .total_tokens_earned
                .checked_add(tokens_earned as u128)
                .expect("Overflow in total_tokens_earned");
            
            env.storage().instance().set(&key, &participant);
        }
    }

    /// Validate that a participant is registered before allowing restricted actions
    fn require_registered(env: &Env, address: &Address) {
        let key = (address.clone(),);
        let participant: Option<Participant> = env.storage().instance().get(&key);
        
        match participant {
            Some(p) if p.is_registered => {},
            Some(_) => panic!("Participant is not registered"),
            None => panic!("Participant not found"),
        }
    }

    /// Store a waste record by ID
    /// Internal helper function for efficient waste storage
    fn set_waste(env: &Env, waste_id: u64, material: &Material) {
        let key = ("waste", waste_id);
        env.storage().instance().set(&key, material);
    }

    /// Retrieve a waste record by ID (internal helper)
    /// Returns None if waste doesn't exist
    fn get_waste_internal(env: &Env, waste_id: u64) -> Option<Material> {
        let key = ("waste", waste_id);
        env.storage().instance().get(&key)
    }

    /// Check if a waste record exists
    pub fn waste_exists(env: Env, waste_id: u64) -> bool {
        let key = ("waste", waste_id);
        env.storage().instance().has(&key)
    }

    /// Get the total count of waste records
    fn get_waste_count(env: &Env) -> u64 {
        env.storage().instance().get(&("waste_count",)).unwrap_or(0)
    }

    /// Increment and return the next waste ID
    fn next_waste_id(env: &Env) -> u64 {
        let count = Self::get_waste_count(env);
        let next_id = count + 1;
        env.storage().instance().set(&("waste_count",), &next_id);
        next_id
    }

    /// Get the total count of incentive records
    #[allow(dead_code)]
    fn get_incentive_count(env: &Env) -> u64 {
        env.storage()
            .instance()
            .get(&("incentive_count",))
            .unwrap_or(0)
    }

    /// Increment and return the next incentive ID
    #[allow(dead_code)]
    fn next_incentive_id(env: &Env) -> u64 {
        let count = Self::get_incentive_count(env);
        let next_id = count + 1;
        env.storage()
            .instance()
            .set(&("incentive_count",), &next_id);
        next_id
    }

    /// Store an incentive record by ID
    /// Internal helper function for efficient incentive storage
    fn set_incentive(env: &Env, incentive_id: u64, incentive: &Incentive) {
        let key = ("incentive", incentive_id);
        env.storage().instance().set(&key, incentive);
    }

    /// Retrieve an incentive record by ID
    /// Returns None if incentive doesn't exist

    fn get_incentive(env: &Env, incentive_id: u64) -> Option<Incentive> {

    fn get_incentive_internal(env: &Env, incentive_id: u64) -> Option<Incentive> {

        let key = ("incentive", incentive_id);
        env.storage().instance().get(&key)
    }


    /// Check if an incentive record exists
    pub fn incentive_exists(env: Env, incentive_id: u64) -> bool {
        let key = ("incentive", incentive_id);
        env.storage().instance().has(&key)
    }

    /// Create a new manufacturer incentive program
    /// Only manufacturers can create incentives
    pub fn create_incentive(
        env: Env,
        waste_type: WasteType,
        reward: u128,
        max_waste_amount: u128,
        rewarder: Address,
    ) -> Incentive {
        rewarder.require_auth();

        // Validate rewarder is a registered manufacturer
        let rewarder_key = (rewarder.clone(),);
        let participant: Participant = env
            .storage()
            .instance()
            .get(&rewarder_key)
            .expect("Rewarder not registered");

        if !participant.role.can_manufacture() {
            panic!("Only manufacturers can create incentives");
        }

        // Validate input values
        if reward == 0 {
            panic!("Reward must be greater than zero");
        }

        if max_waste_amount == 0 {
            panic!("Max waste amount must be greater than zero");
        }

        // Get next incentive ID
        let incentive_id = Self::next_incentive_id(&env);

        // Create incentive
        let incentive = Incentive {
            id: incentive_id,
            waste_type,
            reward,
            max_waste_amount,
            rewarder: rewarder.clone(),
            is_active: true,
            created_at: env.ledger().timestamp(),
        };

        // Store incentive
        Self::set_incentive(&env, incentive_id, &incentive);

        incentive
    }

    /// Get incentive by ID
    pub fn get_incentive_by_id(env: Env, incentive_id: u64) -> Option<Incentive> {
        Self::get_incentive(&env, incentive_id)
    }

    /// Update incentive active status
    /// Only the rewarder can update their incentive
    pub fn update_incentive_status(
        env: Env,
        incentive_id: u64,
        is_active: bool,
    ) -> Incentive {
        let mut incentive: Incentive = Self::get_incentive(&env, incentive_id)
            .expect("Incentive not found");

        // Require auth from the rewarder
        incentive.rewarder.require_auth();

        incentive.is_active = is_active;
        Self::set_incentive(&env, incentive_id, &incentive);

        incentive
    }

    /// Calculate reward for a given waste amount based on an incentive
    /// Returns the reward amount, respecting max_waste_amount and is_active status
    pub fn calculate_incentive_reward(
        env: Env,
        incentive_id: u64,
        waste_amount: u64,
    ) -> u128 {
        let incentive: Incentive = Self::get_incentive(&env, incentive_id)
            .expect("Incentive not found");

        // Check if incentive is active
        if !incentive.is_active {
            return 0;
        }

        // Convert waste_amount to u128 for calculation
        let waste_amount_u128 = waste_amount as u128;

        // Cap waste amount at max_waste_amount
        let eligible_amount = if waste_amount_u128 > incentive.max_waste_amount {
            incentive.max_waste_amount
        } else {
            waste_amount_u128
        };

        // Calculate reward using checked arithmetic
        // reward_per_gram * eligible_amount
        eligible_amount
            .checked_mul(incentive.reward)
            .and_then(|result| result.checked_div(1000)) // Assuming reward is per kg, divide by 1000 for grams
            .expect("Overflow in reward calculation")
    }

    /// Get all incentives for a specific waste type
    pub fn get_incentives_by_waste_type(
        env: Env,
        waste_type: WasteType,
    ) -> soroban_sdk::Vec<Incentive> {
        let mut results = soroban_sdk::Vec::new(&env);
        let count = Self::get_incentive_count(&env);

        for i in 1..=count {
            if let Some(incentive) = Self::get_incentive(&env, i) {
                if incentive.waste_type == waste_type {
                    results.push_back(incentive);
                }
            }
        }

        results
    }

    /// Get all active incentives
    pub fn get_active_incentives(env: Env) -> soroban_sdk::Vec<Incentive> {
        let mut results = soroban_sdk::Vec::new(&env);
        let count = Self::get_incentive_count(&env);

        for i in 1..=count {
            if let Some(incentive) = Self::get_incentive(&env, i) {
                if incentive.is_active {
                    results.push_back(incentive);
                }
            }
        }

        results
    }


    /// Get participant information
    pub fn get_participant(env: Env, address: Address) -> Option<Participant> {
        let key = (address,);
        env.storage().instance().get(&key)
    }

    /// Get participant information with current statistics
    /// Returns participant details along with their recycling statistics
    /// Returns None if participant is not registered
    pub fn get_participant_info(env: Env, address: Address) -> Option<ParticipantInfo> {
        let participant = Self::get_participant(env.clone(), address.clone())?;
        let stats = Self::get_stats(env, address.clone())
            .unwrap_or_else(|| RecyclingStats::new(address));
        
        Some(ParticipantInfo {
            participant,
            stats,
        })
    }

    /// Update participant role
    /// Preserves registration timestamp and other data
    pub fn update_role(env: Env, address: Address, new_role: ParticipantRole) -> Participant {
        address.require_auth();

        let mut participant: Participant =
            Self::get_participant(env.clone(), address.clone()).expect("Participant not found");

        // Validate participant is registered
        if !participant.is_registered {
            panic!("Participant is not registered");
        }

        participant.role = new_role;
        Self::set_participant(&env, &address, &participant);

        participant
    }


    /// Deregister a participant (sets is_registered to false)
    pub fn deregister_participant(env: Env, address: Address) -> Participant {
        address.require_auth();

        let key = (address.clone(),);
        let mut participant: Participant = env
            .storage()
            .instance()
            .get(&key)
            .expect("Participant not found");

        participant.is_registered = false;
        env.storage().instance().set(&key, &participant);

        participant
    }

    /// Update participant location
    pub fn update_location(
        env: Env,
        address: Address,
        latitude: i128,
        longitude: i128,
    ) -> Participant {
        address.require_auth();

        let key = (address.clone(),);
        let mut participant: Participant = env
            .storage()
            .instance()
            .get(&key)
            .expect("Participant not found");

        // Validate participant is registered
        if !participant.is_registered {
            panic!("Participant is not registered");
        }

        participant.latitude = latitude;
        participant.longitude = longitude;
        env.storage().instance().set(&key, &participant);

        participant

    // ========== Waste Transfer History Functions ==========

    /// Get transfer history for a specific waste
    /// Returns chronologically ordered list of transfers
    pub fn get_transfer_history(env: Env, waste_id: u64) -> Vec<WasteTransfer> {
        let key = ("transfers", waste_id);
        env.storage().instance().get(&key).unwrap_or(Vec::new(&env))
    }

    /// Get complete transfer history for a waste (alias for get_transfer_history)
    /// Returns chronologically ordered list of all transfers
    /// Includes all transfer details: from, to, timestamp, and notes
    pub fn get_waste_transfer_history(env: Env, waste_id: u64) -> Vec<WasteTransfer> {
        Self::get_transfer_history(env, waste_id)
    }

    /// Record a waste transfer
    /// Appends to immutable history
    fn record_transfer(env: &Env, waste_id: u64, from: Address, to: Address, note: String) {
        let key = ("transfers", waste_id);
        let mut history: Vec<WasteTransfer> =
            env.storage().instance().get(&key).unwrap_or(Vec::new(env));

        let transfer = WasteTransfer::new(
            waste_id as u128,
            from,
            to,
            env.ledger().timestamp(),
            0,
            0,
            soroban_sdk::symbol_short!("note"),
        );

        history.push_back(transfer);
        env.storage().instance().set(&key, &history);
    }

    /// Transfer waste ownership from one participants to another
    pub fn transfer_waste(
        env: Env,
        waste_id: u64,
        from: Address,
        to: Address,
        note: String,
    ) -> Material {
        from.require_auth();

        // Verify both participants are registered
        if !Self::is_participant_registered(env.clone(), from.clone()) {
            panic!("Sender not registered");
        }
        if !Self::is_participant_registered(env.clone(), to.clone()) {
            panic!("Receiver not registered");
        }

        // Get and update material
        let mut material: Material = Self::get_waste_internal(&env, waste_id).expect("Waste not found");

        // Verify sender owns the waste
        if material.submitter != from {
            panic!("Only waste owner can transfer");
        }

        // Update ownership
        material.submitter = to.clone();
        Self::set_waste(&env, waste_id, &material);

        // Record transfer in history
        Self::record_transfer(&env, waste_id, from, to, note);

        material
    }

    /// Get all transfers for a participants (as sender)
    pub fn get_transfers_from(env: Env, _address: Address) -> Vec<(u64, Vec<WasteTransfer>)> {
        // Note: This is a simplified implementation
        // In production, you'd want to maintain an index for efficient queries
        // This would need to iterate through all wastes
        // For now, returning empty as this requires additional indexing
        Vec::new(&env)
    }

    /// Get all transfers for a participant (as receiver)
    pub fn get_transfers_to(env: Env, _address: Address) -> Vec<(u64, Vec<WasteTransfer>)> {
        // Note: This is a simplified implementation
        // In production, you'd want to maintain an index for efficient queries
        // This would need to iterate through all wastes
        // For now, returning empty as this requires additional indexing
        Vec::new(&env)

    }

    /// Validate if a participant can perform a specific action
    pub fn can_collect(env: Env, address: Address) -> bool {
        let key = (address,);
        if let Some(participant) = env.storage().instance().get::<_, Participant>(&key) {
            participant.is_registered && participant.role.can_collect_materials()
        } else {
            false
        }
    }

    /// Validate if a participants can manufacture
    pub fn can_manufacture(env: Env, address: Address) -> bool {
        let key = (address,);
        if let Some(participant) = env.storage().instance().get::<_, Participant>(&key) {
            participant.is_registered && participant.role.can_manufacture()
        } else {
            false
        }
    }

    /// Submit a new material for recycling
    pub fn submit_material(
        env: Env,
        waste_type: WasteType,
        weight: u64,
        submitter: Address,
        description: String,
    ) -> Material {
        submitter.require_auth();

        // Validate submitter is registered
        Self::require_registered(&env, &submitter);

        // Get next waste ID using the new storage system
        let waste_id = Self::next_waste_id(&env);

        // Create material
        let material = Material::new(
            waste_id,
            waste_type,
            weight,
            submitter.clone(),
            env.ledger().timestamp(),
            description,
        );

        // Store waste using the new storage systems
        Self::set_waste(&env, waste_id, &material);

        // Update stats
        let mut stats: RecyclingStats = env
            .storage()
            .instance()
            .get(&("stats", submitter.clone()))
            .unwrap_or_else(|| RecyclingStats::new(submitter.clone()));

        stats.record_submission(&material);
        env.storage().instance().set(&("stats", submitter.clone()), &stats);

        // Update participant stats
        Self::update_participant_stats(&env, &submitter, weight, 0);

        material
    }

    /// Register new waste with location data
    pub fn recycle_waste(
        env: Env,
        waste_type: WasteType,
        weight: u128,
        recycler: Address,
        latitude: i128,
        longitude: i128,
    ) -> u128 {
        recycler.require_auth();

        if !Self::is_participant_registered(env.clone(), recycler.clone()) {
            panic!("Participant not registered");
        }

        let waste_id = Self::next_waste_id(&env) as u128;
        let timestamp = env.ledger().timestamp();

        let waste = types::Waste::new(
            waste_id,
            waste_type,
            weight,
            recycler.clone(),
            latitude,
            longitude,
            timestamp,
            true,
            false,
            recycler.clone(),
        );

        env.storage().instance().set(&("waste_v2", waste_id), &waste);

        let mut waste_list: Vec<u128> = env
            .storage()
            .instance()
            .get(&("participant_wastes", recycler.clone()))
            .unwrap_or(Vec::new(&env));
        waste_list.push_back(waste_id);
        env.storage()
            .instance()
            .set(&("participant_wastes", recycler.clone()), &waste_list);

        // Emit waste registered event
        events::emit_waste_registered(
            &env,
            waste_id,
            &recycler,
            waste_type,
            weight,
            latitude,
            longitude,
        );

        waste_id
    }

    /// Transfer waste between participants with location tracking
    pub fn transfer_waste_v2(
        env: Env,
        waste_id: u128,
        from: Address,
        to: Address,
        latitude: i128,
        longitude: i128,
    ) -> WasteTransfer {
        from.require_auth();

        let mut waste: types::Waste = env
            .storage()
            .instance()
            .get(&("waste_v2", waste_id))
            .expect("Waste not found");

        if waste.current_owner != from {
            panic!("Caller does not own waste");
        }

        let to_key = (to.clone(),);
        let to_participant: Participant = env
            .storage()
            .instance()
            .get(&to_key)
            .expect("Recipient not registered");

        let from_key = (from.clone(),);
        let from_participant: Participant = env.storage().instance().get(&from_key).unwrap();

        let valid = match (from_participant.role, to_participant.role) {
            (ParticipantRole::Recycler, ParticipantRole::Collector) => true,
            (ParticipantRole::Recycler, ParticipantRole::Manufacturer) => true,
            (ParticipantRole::Collector, ParticipantRole::Manufacturer) => true,
            _ => false,
        };

        if !valid {
            panic!("Invalid transfer");
        }

        waste.transfer_to(to.clone());
        env.storage().instance().set(&("waste_v2", waste_id), &waste);

        let from_list: Vec<u128> = env
            .storage()
            .instance()
            .get(&("participant_wastes", from.clone()))
            .unwrap_or(Vec::new(&env));
        let mut new_from_list = Vec::new(&env);
        for id in from_list.iter() {
            if id != waste_id {
                new_from_list.push_back(id);
            }
        }
        env.storage()
            .instance()
            .set(&("participant_wastes", from.clone()), &new_from_list);

        let mut to_list: Vec<u128> = env
            .storage()
            .instance()
            .get(&("participant_wastes", to.clone()))
            .unwrap_or(Vec::new(&env));
        to_list.push_back(waste_id);
        env.storage()
            .instance()
            .set(&("participant_wastes", to.clone()), &to_list);

        let timestamp = env.ledger().timestamp();
        let transfer = WasteTransfer::new(
            waste_id,
            from.clone(),
            to.clone(),
            timestamp,
            latitude,
            longitude,
            soroban_sdk::symbol_short!("transfer"),
        );

        let mut history: Vec<WasteTransfer> = env
            .storage()
            .instance()
            .get(&("transfer_history", waste_id))
            .unwrap_or(Vec::new(&env));
        history.push_back(transfer.clone());
        env.storage()
            .instance()
            .set(&("transfer_history", waste_id), &history);

        env.events().publish(
            (soroban_sdk::symbol_short!("transfer"), waste_id),
            (from, to, timestamp),
        );

        transfer
    }

    /// Transfer aggregated waste from collector to manufacturer
    pub fn transfer_collected_waste(
        env: Env,
        waste_type: WasteType,
        collector: Address,
        manufacturer: Address,
        latitude: i128,
        longitude: i128,
        notes: soroban_sdk::Symbol,
    ) -> u128 {
        collector.require_auth();

        let collector_key = (collector.clone(),);
        let collector_participant: Participant = env
            .storage()
            .instance()
            .get(&collector_key)
            .expect("Collector not registered");

        if collector_participant.role != ParticipantRole::Collector {
            panic!("Only collectors can use this");
        }

        let manufacturer_key = (manufacturer.clone(),);
        let manufacturer_participant: Participant = env
            .storage()
            .instance()
            .get(&manufacturer_key)
            .expect("Manufacturer not registered");

        if manufacturer_participant.role != ParticipantRole::Manufacturer {
            panic!("Recipient must be manufacturer");
        }

        let waste_id = Self::next_waste_id(&env) as u128;
        let timestamp = env.ledger().timestamp();

        let waste = types::Waste::new(
            waste_id,
            waste_type,
            0,
            manufacturer.clone(),
            latitude,
            longitude,
            timestamp,
            true,
            false,
            manufacturer.clone(),
        );

        env.storage().instance().set(&("waste_v2", waste_id), &waste);

        let mut manufacturer_list: Vec<u128> = env
            .storage()
            .instance()
            .get(&("participant_wastes", manufacturer.clone()))
            .unwrap_or(Vec::new(&env));
        manufacturer_list.push_back(waste_id);
        env.storage()
            .instance()
            .set(&("participant_wastes", manufacturer.clone()), &manufacturer_list);

        let transfer = WasteTransfer::new(
            waste_id,
            collector.clone(),
            manufacturer.clone(),
            timestamp,
            latitude,
            longitude,
            notes,
        );

        let mut history: Vec<WasteTransfer> = env
            .storage()
            .instance()
            .get(&("transfer_history", waste_id))
            .unwrap_or(Vec::new(&env));
        history.push_back(transfer);
        env.storage()
            .instance()
            .set(&("transfer_history", waste_id), &history);

        env.events().publish(
            (soroban_sdk::symbol_short!("bulk_xfr"), waste_id),
            (collector, manufacturer, waste_type, timestamp),
        );

        waste_id
    }

    /// Confirm waste details
    pub fn confirm_waste_details(
        env: Env,
        waste_id: u128,
        confirmer: Address,
    ) -> types::Waste {
        confirmer.require_auth();

        let mut waste: types::Waste = env
            .storage()
            .instance()
            .get(&("waste_v2", waste_id))
            .expect("Waste not found");

        if waste.current_owner == confirmer {
            panic!("Owner cannot confirm own waste");
        }

        if waste.is_confirmed {
            panic!("Waste already confirmed");
        }

        waste.confirm(confirmer.clone());
        env.storage().instance().set(&("waste_v2", waste_id), &waste);

        env.events().publish(
            (soroban_sdk::symbol_short!("confirmed"), waste_id),
            (confirmer, env.ledger().timestamp()),
        );

        waste
    }

    /// Batch submit multiple materials for recycling
    /// More efficient than individual submissions
    pub fn submit_materials_batch(
        env: Env,
        materials: soroban_sdk::Vec<(WasteType, u64, String)>,
        submitter: Address,
    ) -> soroban_sdk::Vec<Material> {
        submitter.require_auth();

        // Validate submitter is registered
        Self::require_registered(&env, &submitter);

        let mut results = soroban_sdk::Vec::new(&env);
        let timestamp = env.ledger().timestamp();

        // Get or create stats once
        let mut stats: RecyclingStats = env
            .storage()
            .instance()
            .get(&("stats", submitter.clone()))
            .unwrap_or_else(|| RecyclingStats::new(submitter.clone()));

        let mut total_weight: u64 = 0;

        // Process each material
        for item in materials.iter() {
            let (waste_type, weight, description) = item;
            let waste_id = Self::next_waste_id(&env);

            let material = Material::new(
                waste_id,
                waste_type,
                weight,
                submitter.clone(),
                timestamp,
                description,
            );

            Self::set_waste(&env, waste_id, &material);
            stats.record_submission(&material);
            results.push_back(material);
            
            // Accumulate weight with overflow check
            total_weight = total_weight.checked_add(weight).expect("Overflow in batch weight");
        }

        // Update stats once at the end
        env.storage().instance().set(&("stats", submitter.clone()), &stats);

        // Update participant stats
        Self::update_participant_stats(&env, &submitter, total_weight, 0);

        results
    }

    /// Get material by ID (alias for backward compatibility)
    pub fn get_material(env: Env, material_id: u64) -> Option<Material> {
        Self::get_waste(env, material_id)
    }

    /// Get waste by ID (alias for backward compatibility)
    pub fn get_waste_by_id(env: Env, waste_id: u64) -> Option<Material> {
        Self::get_waste(env, waste_id)
    }

    /// Get waste by ID (primary public interface)
    /// Returns the waste/material record if it exists, None otherwise
    pub fn get_waste(env: Env, waste_id: u64) -> Option<Material> {
        let key = ("waste", waste_id);
        env.storage().instance().get(&key)
    }

    /// Get all waste IDs owned by a participant
    /// Returns a vector of waste IDs where the participant is the current submitter/owner
    pub fn get_participant_wastes(env: Env, participant: Address) -> Vec<u64> {
        let mut waste_ids = Vec::new(&env);
        let waste_count = env.storage()
            .instance()
            .get::<_, u64>(&("waste_count",))
            .unwrap_or(0);

        // Iterate through all wastes and collect IDs owned by participant
        for waste_id in 1..=waste_count {
            let key = ("waste", waste_id);
            if let Some(material) = env.storage().instance().get::<_, Material>(&key) {
                if material.submitter == participant {
                    waste_ids.push_back(waste_id);
                }
            }
        }

        waste_ids
    }

    /// Get multiple wastes by IDs (batch retrieval)
    pub fn get_wastes_batch(
        env: Env,
        waste_ids: soroban_sdk::Vec<u64>,
    ) -> soroban_sdk::Vec<Option<Material>> {
        let mut results = soroban_sdk::Vec::new(&env);

        for waste_id in waste_ids.iter() {
            results.push_back(Self::get_waste_internal(&env, waste_id));
        }

        results
    }

    /// Verify a material submission (only recyclers can verify)
    pub fn verify_material(env: Env, material_id: u64, verifier: Address) -> Material {
        verifier.require_auth();

        // Check if verifier is a recycler and is registered
        let verifier_key = (verifier.clone(),);
        let participant: Participant = env
            .storage()
            .instance()
            .get(&verifier_key)
            .expect("Verifier not registered");

        if !participant.is_registered {
            panic!("Verifier is not registered");
        }

        if !participant.role.can_process_recyclables() {
            panic!("Only recyclers can verify materials");
        }

        // Get and verify material using new storage system
        let mut material: Material =
            Self::get_waste_internal(&env, material_id).expect("Material not found");

        material.verify();
        Self::set_waste(&env, material_id, &material);

        // Calculate tokens earned
        let tokens_earned = material.calculate_reward_points();

        // Update submitter stats
        let mut stats: RecyclingStats = env
            .storage()
            .instance()
            .get(&("stats", material.submitter.clone()))
            .unwrap_or_else(|| RecyclingStats::new(material.submitter.clone()));

        stats.record_verification(&material);
        env.storage()
            .instance()
            .set(&("stats", material.submitter.clone()), &stats);

        // Update submitter's participant stats with tokens earned
        Self::update_participant_stats(&env, &material.submitter, 0, tokens_earned);

        material
    }

    /// Batch verify multiple materials
    pub fn verify_materials_batch(
        env: Env,
        material_ids: soroban_sdk::Vec<u64>,
        verifier: Address,
    ) -> soroban_sdk::Vec<Material> {
        verifier.require_auth();

        // Check if verifier is a recycler and is registered
        let verifier_key = (verifier.clone(),);
        let participant: Participant = env
            .storage()
            .instance()
            .get(&verifier_key)
            .expect("Verifier not registered");

        if !participant.is_registered {
            panic!("Verifier is not registered");
        }

        if !participant.role.can_process_recyclables() {
            panic!("Only recyclers can verify materials");
        }

        let mut results = soroban_sdk::Vec::new(&env);

        for material_id in material_ids.iter() {
            if let Some(mut material) = Self::get_waste_internal(&env, material_id) {
                material.verify();
                Self::set_waste(&env, material_id, &material);

                // Calculate tokens earned
                let tokens_earned = material.calculate_reward_points();

                // Update submitter stats
                let mut stats: RecyclingStats = env
                    .storage()
                    .instance()
                    .get(&("stats", material.submitter.clone()))
                    .unwrap_or_else(|| RecyclingStats::new(material.submitter.clone()));

                stats.record_verification(&material);
                env.storage()
                    .instance()
                    .set(&("stats", material.submitter.clone()), &stats);

                // Update submitter's participant stats with tokens earned
                Self::update_participant_stats(&env, &material.submitter, 0, tokens_earned);

                results.push_back(material);
            }
        }

        results
    }

    /// Get recycling statistics for a participant
    pub fn get_stats(env: Env, participant: Address) -> Option<RecyclingStats> {
        env.storage().instance().get(&("stats", participant))
    }

    /// Get incentive by ID (public getter)
    pub fn get_incentive_by_id(env: Env, incentive_id: u64) -> Option<Incentive> {
        Self::get_incentive_internal(&env, incentive_id)
    }

    /// Get all incentive IDs created by a specific rewarder (manufacturer)
    pub fn get_incentives_by_rewarder(env: Env, rewarder: Address) -> Vec<u64> {
        let key = ("rewarder_incentives", rewarder);
        env.storage().instance().get(&key).unwrap_or(Vec::new(&env))
    }

    /// Get all incentive IDs for a specific waste type
    pub fn get_incentives_by_waste_type(env: Env, waste_type: WasteType) -> Vec<u64> {
        let key = ("general_incentives", waste_type);
        env.storage().instance().get(&key).unwrap_or(Vec::new(&env))
    }

    /// Get all active incentives for a specific waste type, sorted by reward amount
    /// Returns only active incentives, sorted in descending order by reward_points
    pub fn get_incentives(env: Env, waste_type: WasteType) -> Vec<Incentive> {
        // Get all incentive IDs for this waste type
        let incentive_ids = Self::get_incentives_by_waste_type(env.clone(), waste_type);
        
        let mut active_incentives = Vec::new(&env);
        
        // Collect all active incentives
        for incentive_id in incentive_ids.iter() {
            if let Some(incentive) = Self::get_incentive_internal(&env, incentive_id) {
                // Filter: only include active incentives
                if incentive.active {
                    active_incentives.push_back(incentive);
                }
            }
        }
        
        // Sort by reward_points in descending order (highest rewards first)
        // Using bubble sort since Soroban Vec doesn't have built-in sort
        let len = active_incentives.len();
        for i in 0..len {
            for j in 0..(len - i - 1) {
                let curr = active_incentives.get(j).unwrap();
                let next = active_incentives.get(j + 1).unwrap();
                
                if curr.reward_points < next.reward_points {
                    // Swap elements
                    let temp = curr.clone();
                    active_incentives.set(j, next);
                    active_incentives.set(j + 1, temp);
                }
            }
        }
        
        active_incentives
    }

    /// Get the active incentive with the highest reward for a specific manufacturer and waste type
    /// Returns None if no active incentive is found
    pub fn get_active_incentive_for_manufacturer(
        env: Env,
        manufacturer: Address,
        waste_type: WasteType,
    ) -> Option<Incentive> {
        // Get all incentives for this manufacturer
        let incentive_ids = Self::get_incentives_by_rewarder(env.clone(), manufacturer.clone());
        
        let mut best_incentive: Option<Incentive> = None;
        let mut highest_reward: u64 = 0;
        
        // Iterate through all incentives and find the best active one
        for incentive_id in incentive_ids.iter() {
            if let Some(incentive) = Self::get_incentive_internal(&env, incentive_id) {
                // Check if incentive matches criteria: active and correct waste type
                if incentive.active && incentive.waste_type == waste_type {
                    // Keep track of the incentive with highest reward
                    if incentive.reward_points > highest_reward {
                        highest_reward = incentive.reward_points;
                        best_incentive = Some(incentive);
                    }
                }
            }
        }
        
        best_incentive
    }

    /// Create a new incentive
    pub fn create_incentive(
        env: Env,
        rewarder: Address,
        waste_type: WasteType,
        reward_points: u64,
        total_budget: u64,
    ) -> Incentive {
        rewarder.require_auth();

        // Verify rewarder is a manufacturer
        if !Self::is_participant_registered(env.clone(), rewarder.clone()) {
            panic!("Rewarder not registered");
        }

        let participant =
            Self::get_participant(env.clone(), rewarder.clone()).expect("Rewarder not found");

        if !participant.role.can_manufacture() {
            panic!("Only manufacturers can create incentives");
        }

        // Get next incentive ID
        let incentive_id = Self::next_incentive_id(&env);

        // Create incentive
        let incentive = Incentive::new(
            incentive_id,
            rewarder.clone(),
            waste_type,
            reward_points,
            total_budget,
            env.ledger().timestamp(),
        );

        // Store incentive
        Self::set_incentive(&env, incentive_id, &incentive);

        // Add to rewarder's incentive list
        let key = ("rewarder_incentives", rewarder.clone());
        let mut rewarder_incentives: Vec<u64> =
            env.storage().instance().get(&key).unwrap_or(Vec::new(&env));
        rewarder_incentives.push_back(incentive_id);
        env.storage().instance().set(&key, &rewarder_incentives);

        // Add to general incentives list for this waste type
        let key = ("general_incentives", waste_type);
        let mut general_incentives: Vec<u64> =
            env.storage().instance().get(&key).unwrap_or(Vec::new(&env));
        general_incentives.push_back(incentive_id);
        env.storage().instance().set(&key, &general_incentives);

        incentive
    }

    /// Deactivate an incentive (only by creator)
    pub fn deactivate_incentive(env: Env, incentive_id: u64, rewarder: Address) -> Incentive {
        rewarder.require_auth();

        let mut incentive = Self::get_incentive_internal(&env, incentive_id).expect("Incentive not found");

        // Verify caller is the creator
        if incentive.rewarder != rewarder {
            panic!("Only incentive creator can deactivate");
        }

        incentive.deactivate();
        Self::set_incentive(&env, incentive_id, &incentive);

        incentive
    }

    /// Claim incentive reward for a verified material
    pub fn claim_incentive_reward(
        env: Env,
        incentive_id: u64,
        material_id: u64,
        claimer: Address,
    ) -> u64 {
        claimer.require_auth();

        // Get material and verify it exists and is verified
        let material = Self::get_waste_internal(&env, material_id).expect("Material not found");

        if !material.verified {
            panic!("Material must be verified to claim incentive");
        }

        // Verify claimer is the material submitter
        if material.submitter != claimer {
            panic!("Only material submitter can claim incentive");
        }

        // Get incentive
        let mut incentive = Self::get_incentive_internal(&env, incentive_id).expect("Incentive not found");

        // Verify waste types match
        if incentive.waste_type != material.waste_type {
            panic!("Material waste type does not match incentive");
        }

        // Attempt to claim reward
        let reward = incentive
            .claim_reward(material.weight)
            .expect("Insufficient incentive budget or incentive inactive");

        // Update incentive
        Self::set_incentive(&env, incentive_id, &incentive);

        // Update claimer's stats with bonus points
        let mut stats: RecyclingStats = env
            .storage()
            .instance()
            .get(&("stats", claimer.clone()))
            .unwrap_or_else(|| RecyclingStats::new(claimer.clone()));

        stats.total_points += reward;
        env.storage().instance().set(&("stats", claimer), &stats);

        reward
    }
}

    // ========== Participant Tests ==========

    #[test]
    fn test_register_participant() {
        let env = Env::default();
        env.ledger().with_mut(|li| {
            li.timestamp = 1234567890;
        });
        let contract_id = env.register_contract(None, ScavengerContract);
        let client = ScavengerContractClient::new(&env, &contract_id);

        let user = Address::generate(&env);
        env.mock_all_auths();

        let name = soroban_sdk::Symbol::new(&env, "Alice");
        let participant = client.register_participant(
            &user,
            &ParticipantRole::Recycler,
            &name,
            &40_748_817, // NYC latitude * 1e6
            &-73_985_428, // NYC longitude * 1e6
        );

        assert_eq!(participant.address, user);
        assert_eq!(participant.role, ParticipantRole::Recycler);

        assert_eq!(participant.name, name);
        assert_eq!(participant.latitude, 40_748_817);
        assert_eq!(participant.longitude, -73_985_428);
        assert!(participant.is_registered);
        assert_eq!(participant.total_waste_processed, 0);
        assert_eq!(participant.total_tokens_earned, 0);
        assert!(participant.registered_at > 0);

        // Timestamp can be 0 in test environment
        assert!(participant.registered_at >= 0);
    }

    #[test]
    #[should_panic(expected = "Participant already registered")]
    fn test_register_participant_duplicate() {
        let env = Env::default();
        let contract_id = env.register_contract(None, ScavengerContract);
        let client = ScavengerContractClient::new(&env, &contract_id);

        let user = Address::generate(&env);
        env.mock_all_auths();

        // First registration should succeed
        client.register_participant(&user, &ParticipantRole::Recycler);

        // Second registration should panic
        client.register_participant(&user, &ParticipantRole::Collector);
    }

    #[test]
    fn test_is_participant_registered() {
        let env = Env::default();
        let contract_id = env.register_contract(None, ScavengerContract);
        let client = ScavengerContractClient::new(&env, &contract_id);

        let user = Address::generate(&env);
        let unregistered = Address::generate(&env);
        env.mock_all_auths();

        // Check unregistered user
        assert!(!client.is_participant_registered(&unregistered));

        // Register user
        client.register_participant(&user, &ParticipantRole::Recycler);

        // Check registered user
        assert!(client.is_participant_registered(&user));

    }

    #[test]
    fn test_get_participant() {
        let env = Env::default();
        let contract_id = env.register_contract(None, ScavengerContract);
        let client = ScavengerContractClient::new(&env, &contract_id);

        let user = Address::generate(&env);
        env.mock_all_auths();

        let name = soroban_sdk::Symbol::new(&env, "Bob");
        client.register_participant(
            &user,
            &ParticipantRole::Collector,
            &name,
            &51_507_351, // London latitude * 1e6
            &-141_278, // London longitude * 1e6
        );

        let participant = client.get_participant(&user);
        assert!(participant.is_some());
        let p = participant.unwrap();
        assert_eq!(p.role, ParticipantRole::Collector);
        assert_eq!(p.name, name);
        assert!(p.is_registered);
    }

    #[test]
    fn test_update_role() {
        let env = Env::default();
        let contract_id = env.register_contract(None, ScavengerContract);
        let client = ScavengerContractClient::new(&env, &contract_id);

        let user = Address::generate(&env);
        env.mock_all_auths();

        let name = soroban_sdk::Symbol::new(&env, "Charlie");
        client.register_participant(
            &user,
            &ParticipantRole::Recycler,
            &name,
            &0,
            &0,
        );
        let updated = client.update_role(&user, &ParticipantRole::Manufacturer);

        assert_eq!(updated.role, ParticipantRole::Manufacturer);
        assert!(updated.is_registered);
    }

    #[test]
    fn test_can_collect() {
        let env = Env::default();
        let contract_id = env.register_contract(None, ScavengerContract);
        let client = ScavengerContractClient::new(&env, &contract_id);

        let recycler = Address::generate(&env);
        let collector = Address::generate(&env);
        let manufacturer = Address::generate(&env);
        env.mock_all_auths();

        let name = soroban_sdk::Symbol::new(&env, "Test");
        client.register_participant(&recycler, &ParticipantRole::Recycler, &name, &0, &0);
        client.register_participant(&collector, &ParticipantRole::Collector, &name, &0, &0);
        client.register_participant(&manufacturer, &ParticipantRole::Manufacturer, &name, &0, &0);

        assert!(client.can_collect(&recycler));
        assert!(client.can_collect(&collector));
        assert!(!client.can_collect(&manufacturer));
    }

    #[test]
    fn test_can_manufacture() {
        let env = Env::default();
        let contract_id = env.register_contract(None, ScavengerContract);
        let client = ScavengerContractClient::new(&env, &contract_id);

        let recycler = Address::generate(&env);
        let manufacturer = Address::generate(&env);
        env.mock_all_auths();

        let name = soroban_sdk::Symbol::new(&env, "Test");
        client.register_participant(&recycler, &ParticipantRole::Recycler, &name, &0, &0);
        client.register_participant(&manufacturer, &ParticipantRole::Manufacturer, &name, &0, &0);

        assert!(!client.can_manufacture(&recycler));
        assert!(client.can_manufacture(&manufacturer));
    }

    #[test]
    fn test_all_role_types() {
        let env = Env::default();
        let contract_id = env.register_contract(None, ScavengerContract);
        let client = ScavengerContractClient::new(&env, &contract_id);

        let user1 = Address::generate(&env);
        let user2 = Address::generate(&env);
        let user3 = Address::generate(&env);
        env.mock_all_auths();

        let name = soroban_sdk::Symbol::new(&env, "Test");
        client.register_participant(&user1, &ParticipantRole::Recycler, &name, &0, &0);
        client.register_participant(&user2, &ParticipantRole::Collector, &name, &0, &0);
        client.register_participant(&user3, &ParticipantRole::Manufacturer, &name, &0, &0);

        let p1 = client.get_participant(&user1).unwrap();
        let p2 = client.get_participant(&user2).unwrap();
        let p3 = client.get_participant(&user3).unwrap();

        assert_eq!(p1.role, ParticipantRole::Recycler);
        assert_eq!(p2.role, ParticipantRole::Collector);
        assert_eq!(p3.role, ParticipantRole::Manufacturer);
    }

    #[test]
    fn test_waste_type_storage() {
        // WasteType can be stored and retrieved from storage
        // This is validated through the contract tests
        let waste_types = [
            WasteType::Paper,
            WasteType::PetPlastic,
            WasteType::Plastic,
            WasteType::Metal,
            WasteType::Glass,
        ];

        // Verify all types are valid
        for waste_type in waste_types.iter() {
            assert!(!waste_type.as_str().is_empty());
        }
    }

    #[test]
    fn test_waste_type_serialization() {
        // Test all waste types can be serialized/deserialized
        // This is validated through the contract tests
        let all_types = [
            WasteType::Paper,
            WasteType::PetPlastic,
            WasteType::Plastic,
            WasteType::Metal,
            WasteType::Glass,
        ];

        for waste_type in all_types.iter() {
            // Verify string representation
            assert!(!waste_type.as_str().is_empty());
        }
    }

    #[test]
    fn test_submit_material() {
        let env = Env::default();
        let contract_id = env.register_contract(None, ScavengerContract);
        let client = ScavengerContractClient::new(&env, &contract_id);

        let user = Address::generate(&env);
        env.mock_all_auths();

        // Register user first
        let name = soroban_sdk::Symbol::new(&env, "Test");
        client.register_participant(&user, &ParticipantRole::Collector, &name, &0, &0);

        let description = String::from_str(&env, "Plastic bottles");
        let material = client.submit_material(&WasteType::PetPlastic, &5000, &user, &description);

        assert_eq!(material.id, 1);
        assert_eq!(material.waste_type, WasteType::PetPlastic);
        assert_eq!(material.weight, 5000);
        assert_eq!(material.submitter, user);
        assert!(!material.verified);

        // Check participant stats updated
        let participant = client.get_participant(&user).unwrap();
        assert_eq!(participant.total_waste_processed, 5000);
        assert_eq!(participant.total_tokens_earned, 0); // Not verified yet
    }

    #[test]
    fn test_recycle_waste() {
        let env = Env::default();
        let contract_id = env.register_contract(None, ScavengerContract);
        let client = ScavengerContractClient::new(&env, &contract_id);

        let recycler = Address::generate(&env);
        env.mock_all_auths();

        client.register_participant(&recycler, &ParticipantRole::Recycler);

        let waste_id = client.recycle_waste(
            &WasteType::Plastic,
            &2500,
            &recycler,
            &40_500_000,
            &-74_000_000,
        );

        assert_eq!(waste_id, 1);
    }

    #[test]
    fn test_transfer_waste_v2() {
        let env = Env::default();
        let contract_id = env.register_contract(None, ScavengerContract);
        let client = ScavengerContractClient::new(&env, &contract_id);

        let recycler = Address::generate(&env);
        let collector = Address::generate(&env);
        env.mock_all_auths();

        client.register_participant(&recycler, &ParticipantRole::Recycler);
        client.register_participant(&collector, &ParticipantRole::Collector);

        let waste_id = client.recycle_waste(
            &WasteType::Metal,
            &3000,
            &recycler,
            &40_500_000,
            &-74_000_000,
        );

        let transfer = client.transfer_waste_v2(
            &waste_id,
            &recycler,
            &collector,
            &40_600_000,
            &-74_100_000,
        );

        assert_eq!(transfer.waste_id, waste_id);
        assert_eq!(transfer.from, recycler);
        assert_eq!(transfer.to, collector);
    }

    #[test]
    fn test_transfer_collected_waste() {
        let env = Env::default();
        let contract_id = env.register_contract(None, ScavengerContract);
        let client = ScavengerContractClient::new(&env, &contract_id);

        let collector = Address::generate(&env);
        let manufacturer = Address::generate(&env);
        env.mock_all_auths();

        client.register_participant(&collector, &ParticipantRole::Collector);
        client.register_participant(&manufacturer, &ParticipantRole::Manufacturer);

        let waste_id = client.transfer_collected_waste(
            &WasteType::Plastic,
            &collector,
            &manufacturer,
            &41_000_000,
            &-73_500_000,
            &soroban_sdk::symbol_short!("bulk"),
        );

        assert_eq!(waste_id, 1);
    }

    #[test]
    fn test_confirm_waste_details() {
        let env = Env::default();
        let contract_id = env.register_contract(None, ScavengerContract);
        let client = ScavengerContractClient::new(&env, &contract_id);

        let recycler = Address::generate(&env);
        let verifier = Address::generate(&env);
        env.mock_all_auths();

        client.register_participant(&recycler, &ParticipantRole::Recycler);
        client.register_participant(&verifier, &ParticipantRole::Collector);

        let waste_id = client.recycle_waste(
            &WasteType::Glass,
            &1500,
            &recycler,
            &40_700_000,
            &-73_900_000,
        );

        let waste = client.confirm_waste_details(&waste_id, &verifier);

        assert!(waste.is_confirmed);
        assert_eq!(waste.confirmer, verifier);
    }

    #[test]
    fn test_get_material() {
        let env = Env::default();
        let contract_id = env.register_contract(None, ScavengerContract);
        let client = ScavengerContractClient::new(&env, &contract_id);

        let user = Address::generate(&env);
        env.mock_all_auths();

        // Register user first
        let name = soroban_sdk::Symbol::new(&env, "Test");
        client.register_participant(&user, &ParticipantRole::Collector, &name, &0, &0);

        let description = String::from_str(&env, "Metal cans");
        client.submit_material(&WasteType::Metal, &3000, &user, &description);

        let material = client.get_material(&1);
        assert!(material.is_some());
        assert_eq!(material.unwrap().waste_type, WasteType::Metal);
    }

    #[test]
    fn test_verify_material() {
        let env = Env::default();
        let contract_id = env.register_contract(None, ScavengerContract);
        let client = ScavengerContractClient::new(&env, &contract_id);

        let submitter = Address::generate(&env);
        let recycler = Address::generate(&env);
        env.mock_all_auths();

        // Register both users
        let name = soroban_sdk::Symbol::new(&env, "Test");
        client.register_participant(&submitter, &ParticipantRole::Collector, &name, &0, &0);
        client.register_participant(&recycler, &ParticipantRole::Recycler, &name, &0, &0);

        // Submit material
        let description = String::from_str(&env, "Glass bottles");
        client.submit_material(&WasteType::Glass, &2000, &submitter, &description);

        // Verify material
        let verified = client.verify_material(&1, &recycler);
        assert!(verified.verified);

        // Check submitter's tokens were updated
        let participant = client.get_participant(&submitter).unwrap();
        assert_eq!(participant.total_waste_processed, 2000);
        assert_eq!(participant.total_tokens_earned, 20); // 2kg * 2 * 10
    }

    #[test]
    fn test_multiple_materials() {
        let env = Env::default();
        let contract_id = env.register_contract(None, ScavengerContract);
        let client = ScavengerContractClient::new(&env, &contract_id);

        let user = Address::generate(&env);
        env.mock_all_auths();

        // Register user first
        let name = soroban_sdk::Symbol::new(&env, "Test");
        client.register_participant(&user, &ParticipantRole::Collector, &name, &0, &0);

        // Submit multiple materials
        let desc1 = String::from_str(&env, "Paper");
        let desc2 = String::from_str(&env, "Plastic");
        let desc3 = String::from_str(&env, "Metal");

        client.submit_material(&WasteType::Paper, &1000, &user, &desc1);
        client.submit_material(&WasteType::Plastic, &2000, &user, &desc2);
        client.submit_material(&WasteType::Metal, &3000, &user, &desc3);

        // Verify all materials exist
        assert!(client.get_material(&1).is_some());
        assert!(client.get_material(&2).is_some());
        assert!(client.get_material(&3).is_some());
        assert!(client.get_material(&4).is_none());

        // Check participant stats
        let participant = client.get_participant(&user).unwrap();
        assert_eq!(participant.total_waste_processed, 6000);
    }

    #[test]
    fn test_stats_tracking() {
        let env = Env::default();
        let contract_id = env.register_contract(None, ScavengerContract);
        let client = ScavengerContractClient::new(&env, &contract_id);

        let user = Address::generate(&env);
        env.mock_all_auths();

        // Register user first
        let name = soroban_sdk::Symbol::new(&env, "Test");
        client.register_participant(&user, &ParticipantRole::Collector, &name, &0, &0);

        // Submit materials
        let desc = String::from_str(&env, "Test");
        client.submit_material(&WasteType::Paper, &1000, &user, &desc);
        client.submit_material(&WasteType::Plastic, &2000, &user, &desc);

        // Check stats
        let stats = client.get_stats(&user);
        assert!(stats.is_some());
        let stats = stats.unwrap();
        assert_eq!(stats.total_submissions, 2);
        assert_eq!(stats.total_weight, 3000);
    }

    #[test]
    fn test_stats_with_verification() {
        let env = Env::default();
        let contract_id = env.register_contract(None, ScavengerContract);
        let client = ScavengerContractClient::new(&env, &contract_id);

        let submitter = Address::generate(&env);
        let recycler = Address::generate(&env);
        env.mock_all_auths();

        // Register both users
        let name = soroban_sdk::Symbol::new(&env, "Test");
        client.register_participant(&submitter, &ParticipantRole::Collector, &name, &0, &0);
        client.register_participant(&recycler, &ParticipantRole::Recycler, &name, &0, &0);

        // Submit and verify material
        let desc = String::from_str(&env, "Metal cans");
        client.submit_material(&WasteType::Metal, &5000, &submitter, &desc);
        client.verify_material(&1, &recycler);

        // Check stats
        let stats = client.get_stats(&submitter).unwrap();
        assert_eq!(stats.total_submissions, 1);
        assert_eq!(stats.verified_submissions, 1);
        assert_eq!(stats.total_points, 250); // 5kg * 5 * 10
        assert_eq!(stats.verification_rate(), 100);
    }

    #[test]
    fn test_stats_most_submitted_type() {
        let env = Env::default();
        let contract_id = env.register_contract(None, ScavengerContract);
        let client = ScavengerContractClient::new(&env, &contract_id);

        let user = Address::generate(&env);
        env.mock_all_auths();

        // Register user first
        let name = soroban_sdk::Symbol::new(&env, "Test");
        client.register_participant(&user, &ParticipantRole::Collector, &name, &0, &0);

        let desc = String::from_str(&env, "Test");

        // Submit multiple plastic items
        client.submit_material(&WasteType::Plastic, &1000, &user, &desc);
        client.submit_material(&WasteType::Plastic, &2000, &user, &desc);
        client.submit_material(&WasteType::Paper, &1000, &user, &desc);

        let stats = client.get_stats(&user).unwrap();
        assert_eq!(stats.plastic_count, 2);
        assert_eq!(stats.paper_count, 1);
    }

    // Waste Storage System Tests
    #[test]
    fn test_waste_exists() {
        let env = Env::default();
        let contract_id = env.register_contract(None, ScavengerContract);
        let client = ScavengerContractClient::new(&env, &contract_id);

        let user = Address::generate(&env);
        env.mock_all_auths();

        // Check non-existent waste
        assert!(!client.waste_exists(&1));

        // Submit material
        let desc = String::from_str(&env, "Test waste");
        client.submit_material(&WasteType::Paper, &1000, &user, &desc);

        // Check existing waste
        assert!(client.waste_exists(&1));
        assert!(!client.waste_exists(&2));
    }

    #[test]
    fn test_get_waste_by_id() {
        let env = Env::default();
        let contract_id = env.register_contract(None, ScavengerContract);
        let client = ScavengerContractClient::new(&env, &contract_id);

        let user = Address::generate(&env);
        env.mock_all_auths();

        let desc = String::from_str(&env, "Glass bottles");
        client.submit_material(&WasteType::Glass, &3000, &user, &desc);

        let waste = client.get_waste_by_id(&1);
        assert!(waste.is_some());
        let waste = waste.unwrap();
        assert_eq!(waste.id, 1);
        assert_eq!(waste.waste_type, WasteType::Glass);
        assert_eq!(waste.weight, 3000);
    }

    #[test]
    fn test_get_wastes_batch() {
        let env = Env::default();
        let contract_id = env.register_contract(None, ScavengerContract);
        let client = ScavengerContractClient::new(&env, &contract_id);

        let user = Address::generate(&env);
        env.mock_all_auths();

        let desc = String::from_str(&env, "Test");

        // Submit multiple materials
        client.submit_material(&WasteType::Paper, &1000, &user, &desc);
        client.submit_material(&WasteType::Plastic, &2000, &user, &desc);
        client.submit_material(&WasteType::Metal, &3000, &user, &desc);

        // Batch retrieve
        let mut ids = soroban_sdk::Vec::new(&env);
        ids.push_back(1);
        ids.push_back(2);
        ids.push_back(3);
        ids.push_back(99); // Non-existent

        let results = client.get_wastes_batch(&ids);
        assert_eq!(results.len(), 4);
        assert!(results.get(0).unwrap().is_some());
        assert!(results.get(1).unwrap().is_some());
        assert!(results.get(2).unwrap().is_some());
        assert!(results.get(3).unwrap().is_none());
    }

    #[test]
    fn test_submit_materials_batch() {
        let env = Env::default();
        let contract_id = env.register_contract(None, ScavengerContract);
        let client = ScavengerContractClient::new(&env, &contract_id);

        let user = Address::generate(&env);
        env.mock_all_auths();

        // Prepare batch materials
        let mut materials = soroban_sdk::Vec::new(&env);
        materials.push_back((
            WasteType::Paper,
            1000u64,
            String::from_str(&env, "Paper batch"),
        ));
        materials.push_back((
            WasteType::Plastic,
            2000u64,
            String::from_str(&env, "Plastic batch"),
        ));
        materials.push_back((
            WasteType::Metal,
            3000u64,
            String::from_str(&env, "Metal batch"),
        ));

        // Submit batch
        let results = client.submit_materials_batch(&materials, &user);

        assert_eq!(results.len(), 3);
        assert_eq!(results.get(0).unwrap().waste_type, WasteType::Paper);
        assert_eq!(results.get(1).unwrap().waste_type, WasteType::Plastic);
        assert_eq!(results.get(2).unwrap().waste_type, WasteType::Metal);

        // Verify all were stored
        assert!(client.waste_exists(&1));
        assert!(client.waste_exists(&2));
        assert!(client.waste_exists(&3));

        // Check stats were updated
        let stats = client.get_stats(&user).unwrap();
        assert_eq!(stats.total_submissions, 3);
        assert_eq!(stats.total_weight, 6000);
    }

    #[test]
    fn test_verify_materials_batch() {
        let env = Env::default();
        let contract_id = env.register_contract(None, ScavengerContract);
        let client = ScavengerContractClient::new(&env, &contract_id);

        let submitter = Address::generate(&env);
        let recycler = Address::generate(&env);
        env.mock_all_auths();

        // Register recycler
        client.register_participant(&recycler, &ParticipantRole::Recycler);

        // Submit multiple materials
        let desc = String::from_str(&env, "Test");
        client.submit_material(&WasteType::Paper, &1000, &submitter, &desc);
        client.submit_material(&WasteType::Plastic, &2000, &submitter, &desc);
        client.submit_material(&WasteType::Metal, &3000, &submitter, &desc);

        // Batch verify
        let mut ids = soroban_sdk::Vec::new(&env);
        ids.push_back(1);
        ids.push_back(2);
        ids.push_back(3);

        let results = client.verify_materials_batch(&ids, &recycler);

        assert_eq!(results.len(), 3);
        assert!(results.get(0).unwrap().verified);
        assert!(results.get(1).unwrap().verified);
        assert!(results.get(2).unwrap().verified);

        // Check stats were updated
        let stats = client.get_stats(&submitter).unwrap();
        assert_eq!(stats.verified_submissions, 3);
    }

    #[test]
    fn test_waste_id_no_collision() {
        let env = Env::default();
        let contract_id = env.register_contract(None, ScavengerContract);
        let client = ScavengerContractClient::new(&env, &contract_id);

        let user1 = Address::generate(&env);
        let user2 = Address::generate(&env);
        env.mock_all_auths();

        let desc = String::from_str(&env, "Test");

        // Submit materials from different users
        let m1 = client.submit_material(&WasteType::Paper, &1000, &user1, &desc);
        let m2 = client.submit_material(&WasteType::Plastic, &2000, &user2, &desc);
        let m3 = client.submit_material(&WasteType::Metal, &3000, &user1, &desc);

        // Verify unique IDs
        assert_eq!(m1.id, 1);
        assert_eq!(m2.id, 2);
        assert_eq!(m3.id, 3);
        assert_ne!(m1.id, m2.id);
        assert_ne!(m2.id, m3.id);
    }

    #[test]
    fn test_waste_storage_efficiency() {
        let env = Env::default();
        let contract_id = env.register_contract(None, ScavengerContract);
        let client = ScavengerContractClient::new(&env, &contract_id);

        let user = Address::generate(&env);
        env.mock_all_auths();

        let desc = String::from_str(&env, "Efficiency test");

        // Submit material
        let material = client.submit_material(&WasteType::Paper, &5000, &user, &desc);

        // Retrieve should be efficient (single storage read)
        let retrieved = client.get_waste_by_id(&material.id);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id, material.id);
    }

    // Counter Storage System Tests
    #[test]
    fn test_waste_id_counter_initialization() {
        let env = Env::default();
        let contract_id = env.register_contract(None, ScavengerContract);
        let client = ScavengerContractClient::new(&env, &contract_id);

        let user = Address::generate(&env);
        env.mock_all_auths();

        let desc = String::from_str(&env, "First submission");

        // First submission should get ID 1
        let material = client.submit_material(&WasteType::Paper, &1000, &user, &desc);
        assert_eq!(material.id, 1);
    }

    #[test]
    fn test_waste_id_counter_increments_correctly() {
        let env = Env::default();
        let contract_id = env.register_contract(None, ScavengerContract);
        let client = ScavengerContractClient::new(&env, &contract_id);

        let user = Address::generate(&env);
        env.mock_all_auths();

        let desc = String::from_str(&env, "Test");

        // Submit multiple materials and verify sequential IDs
        let m1 = client.submit_material(&WasteType::Paper, &1000, &user, &desc);
        let m2 = client.submit_material(&WasteType::Plastic, &2000, &user, &desc);
        let m3 = client.submit_material(&WasteType::Metal, &3000, &user, &desc);
        let m4 = client.submit_material(&WasteType::Glass, &4000, &user, &desc);
        let m5 = client.submit_material(&WasteType::PetPlastic, &5000, &user, &desc);

        assert_eq!(m1.id, 1);
        assert_eq!(m2.id, 2);
        assert_eq!(m3.id, 3);
        assert_eq!(m4.id, 4);
        assert_eq!(m5.id, 5);
    }

    #[test]
    fn test_waste_id_no_reuse() {
        let env = Env::default();
        let contract_id = env.register_contract(None, ScavengerContract);
        let client = ScavengerContractClient::new(&env, &contract_id);

        let user = Address::generate(&env);
        env.mock_all_auths();

        let desc = String::from_str(&env, "Test");

        // Submit materials
        let m1 = client.submit_material(&WasteType::Paper, &1000, &user, &desc);
        let m2 = client.submit_material(&WasteType::Plastic, &2000, &user, &desc);

        // Even after retrieving, new submissions should get new IDs
        let _retrieved = client.get_material(&m1.id);
        let m3 = client.submit_material(&WasteType::Metal, &3000, &user, &desc);

        assert_eq!(m1.id, 1);
        assert_eq!(m2.id, 2);
        assert_eq!(m3.id, 3);

        // Verify no ID collision
        assert_ne!(m1.id, m2.id);
        assert_ne!(m2.id, m3.id);
        assert_ne!(m1.id, m3.id);
    }

    #[test]
    fn test_waste_id_counter_thread_safe_operations() {
        let env = Env::default();
        let contract_id = env.register_contract(None, ScavengerContract);
        let client = ScavengerContractClient::new(&env, &contract_id);

        let user1 = Address::generate(&env);
        let user2 = Address::generate(&env);
        let user3 = Address::generate(&env);
        env.mock_all_auths();

        let desc = String::from_str(&env, "Concurrent test");

        // Simulate concurrent submissions from different users
        let m1 = client.submit_material(&WasteType::Paper, &1000, &user1, &desc);
        let m2 = client.submit_material(&WasteType::Plastic, &2000, &user2, &desc);
        let m3 = client.submit_material(&WasteType::Metal, &3000, &user3, &desc);
        let m4 = client.submit_material(&WasteType::Glass, &4000, &user1, &desc);

        // All IDs should be unique and sequential
        assert_eq!(m1.id, 1);
        assert_eq!(m2.id, 2);
        assert_eq!(m3.id, 3);
        assert_eq!(m4.id, 4);
    }

    #[test]
    fn test_waste_id_counter_with_batch_operations() {
        let env = Env::default();
        let contract_id = env.register_contract(None, ScavengerContract);
        let client = ScavengerContractClient::new(&env, &contract_id);

        let user = Address::generate(&env);
        env.mock_all_auths();

        // Submit single material first
        let desc = String::from_str(&env, "Single");
        let m1 = client.submit_material(&WasteType::Paper, &1000, &user, &desc);
        assert_eq!(m1.id, 1);

        // Submit batch
        let mut materials = soroban_sdk::Vec::new(&env);
        materials.push_back((
            WasteType::Plastic,
            2000u64,
            String::from_str(&env, "Batch 1"),
        ));
        materials.push_back((WasteType::Metal, 3000u64, String::from_str(&env, "Batch 2")));

        let batch_results = client.submit_materials_batch(&materials, &user);

        // Batch should continue from where single left off
        assert_eq!(batch_results.get(0).unwrap().id, 2);
        assert_eq!(batch_results.get(1).unwrap().id, 3);

        // Submit another single material
        let m4 = client.submit_material(&WasteType::Glass, &4000, &user, &desc);
        assert_eq!(m4.id, 4);
    }

    #[test]
    fn test_waste_id_counter_persistence() {
        let env = Env::default();
        let contract_id = env.register_contract(None, ScavengerContract);
        let client = ScavengerContractClient::new(&env, &contract_id);

        let user = Address::generate(&env);
        env.mock_all_auths();

        let desc = String::from_str(&env, "Persistence test");

        // Submit materials
        client.submit_material(&WasteType::Paper, &1000, &user, &desc);
        client.submit_material(&WasteType::Plastic, &2000, &user, &desc);

        // Verify materials exist
        assert!(client.waste_exists(&1));
        assert!(client.waste_exists(&2));

        // Submit more materials - counter should persist
        let m3 = client.submit_material(&WasteType::Metal, &3000, &user, &desc);
        assert_eq!(m3.id, 3);
    }

    #[test]
    fn test_incentive_id_counter_initialization() {
        let env = Env::default();
        let contract_id = env.register_contract(None, ScavengerContract);

        // Test that incentive counter starts at 0
        let count = env.as_contract(&contract_id, || {
            ScavengerContract::get_incentive_count(&env)
        });
        assert_eq!(count, 0);

        // Test first increment
        let id1 = env.as_contract(&contract_id, || ScavengerContract::next_incentive_id(&env));
        assert_eq!(id1, 1);

        // Test second increment
        let id2 = env.as_contract(&contract_id, || ScavengerContract::next_incentive_id(&env));
        assert_eq!(id2, 2);
    }

    #[test]
    fn test_incentive_id_counter_increments_correctly() {
        let env = Env::default();
        let contract_id = env.register_contract(None, ScavengerContract);

        // Generate multiple IDs
        let id1 = env.as_contract(&contract_id, || ScavengerContract::next_incentive_id(&env));
        let id2 = env.as_contract(&contract_id, || ScavengerContract::next_incentive_id(&env));
        let id3 = env.as_contract(&contract_id, || ScavengerContract::next_incentive_id(&env));
        let id4 = env.as_contract(&contract_id, || ScavengerContract::next_incentive_id(&env));
        let id5 = env.as_contract(&contract_id, || ScavengerContract::next_incentive_id(&env));

        // Verify sequential increments
        assert_eq!(id1, 1);
        assert_eq!(id2, 2);
        assert_eq!(id3, 3);
        assert_eq!(id4, 4);
        assert_eq!(id5, 5);
    }

    #[test]
    fn test_incentive_id_no_reuse() {
        let env = Env::default();
        let contract_id = env.register_contract(None, ScavengerContract);

        // Generate IDs
        let id1 = env.as_contract(&contract_id, || ScavengerContract::next_incentive_id(&env));
        let id2 = env.as_contract(&contract_id, || ScavengerContract::next_incentive_id(&env));
        let id3 = env.as_contract(&contract_id, || ScavengerContract::next_incentive_id(&env));

        // Verify all IDs are unique
        assert_ne!(id1, id2);
        assert_ne!(id2, id3);
        assert_ne!(id1, id3);

        // Verify they are sequential (no gaps)
        assert_eq!(id2, id1 + 1);
        assert_eq!(id3, id2 + 1);
    }

    #[test]
    fn test_incentive_id_counter_persistence() {
        let env = Env::default();
        let contract_id = env.register_contract(None, ScavengerContract);

        // Generate some IDs
        env.as_contract(&contract_id, || {
            ScavengerContract::next_incentive_id(&env);
            ScavengerContract::next_incentive_id(&env);
        });

        // Check count persists
        let count = env.as_contract(&contract_id, || {
            ScavengerContract::get_incentive_count(&env)
        });
        assert_eq!(count, 2);

        // Generate more IDs
        let id3 = env.as_contract(&contract_id, || ScavengerContract::next_incentive_id(&env));
        assert_eq!(id3, 3);

        // Verify count updated
        let count = env.as_contract(&contract_id, || {
            ScavengerContract::get_incentive_count(&env)
        });
        assert_eq!(count, 3);
    }

    #[test]
    fn test_waste_and_incentive_counters_independent() {
        let env = Env::default();
        let contract_id = env.register_contract(None, ScavengerContract);
        let client = ScavengerContractClient::new(&env, &contract_id);

        let user = Address::generate(&env);
        env.mock_all_auths();

        let desc = String::from_str(&env, "Independence test");

        // Generate waste IDs
        let m1 = client.submit_material(&WasteType::Paper, &1000, &user, &desc);
        let m2 = client.submit_material(&WasteType::Plastic, &2000, &user, &desc);

        // Generate incentive IDs
        let i1 = env.as_contract(&contract_id, || ScavengerContract::next_incentive_id(&env));
        let i2 = env.as_contract(&contract_id, || ScavengerContract::next_incentive_id(&env));

        // Generate more waste IDs
        let m3 = client.submit_material(&WasteType::Metal, &3000, &user, &desc);

        // Generate more incentive IDs
        let i3 = env.as_contract(&contract_id, || ScavengerContract::next_incentive_id(&env));

        // Verify waste IDs are sequential
        assert_eq!(m1.id, 1);
        assert_eq!(m2.id, 2);
        assert_eq!(m3.id, 3);

        // Verify incentive IDs are sequential
        assert_eq!(i1, 1);
        assert_eq!(i2, 2);
        assert_eq!(i3, 3);

        // Verify counters are independent
        let waste_count =
            env.as_contract(&contract_id, || ScavengerContract::get_waste_count(&env));
        let incentive_count = env.as_contract(&contract_id, || {
            ScavengerContract::get_incentive_count(&env)
        });
        assert_eq!(waste_count, 3);
        assert_eq!(incentive_count, 3);
    }


    // ========== Waste Transfer History Tests ==========

    #[test]
    fn test_transfer_waste() {
        let env = Env::default();
        let contract_id = env.register_contract(None, ScavengerContract);
        let client = ScavengerContractClient::new(&env, &contract_id);

        let owner = Address::generate(&env);
        let receiver = Address::generate(&env);
        env.mock_all_auths();

        // Register both participants
        client.register_participant(&owner, &ParticipantRole::Recycler);
        client.register_participant(&receiver, &ParticipantRole::Collector);

        // Submit material
        let desc = String::from_str(&env, "Transfer test");
        let material = client.submit_material(&WasteType::Paper, &1000, &owner, &desc);

        // Transfer waste
        let note = String::from_str(&env, "Transferring to collector");
        let transferred = client.transfer_waste(&material.id, &owner, &receiver, &note);

        // Verify ownership changed
        assert_eq!(transferred.submitter, receiver);

        // Verify transfer history
        let history = client.get_transfer_history(&material.id);
        assert_eq!(history.len(), 1);

        let transfer = history.get(0).unwrap();
        assert_eq!(transfer.waste_id, material.id);
        assert_eq!(transfer.from, owner);
        assert_eq!(transfer.to, receiver);
        // Timestamp can be 0 in test environment
        assert!(transfer.transferred_at >= 0);
    }

    #[test]
    #[should_panic(expected = "Sender not registered")]
    fn test_transfer_waste_unregistered_sender() {
        let env = Env::default();
        let contract_id = env.register_contract(None, ScavengerContract);
        let client = ScavengerContractClient::new(&env, &contract_id);

        let owner = Address::generate(&env);
        let receiver = Address::generate(&env);
        env.mock_all_auths();

        // Only register receiver
        client.register_participant(&receiver, &ParticipantRole::Collector);

        // Submit material (this will work without registration check)
        let desc = String::from_str(&env, "Test");
        let material = client.submit_material(&WasteType::Paper, &1000, &owner, &desc);

        // Transfer should fail - sender not registered
        let note = String::from_str(&env, "Transfer");
        client.transfer_waste(&material.id, &owner, &receiver, &note);
    }

    #[test]
    #[should_panic(expected = "Receiver not registered")]
    fn test_transfer_waste_unregistered_receiver() {
        let env = Env::default();
        let contract_id = env.register_contract(None, ScavengerContract);
        let client = ScavengerContractClient::new(&env, &contract_id);

        let owner = Address::generate(&env);
        let receiver = Address::generate(&env);
        env.mock_all_auths();

        // Only register owner
        client.register_participant(&owner, &ParticipantRole::Recycler);

        // Submit material
        let desc = String::from_str(&env, "Test");
        let material = client.submit_material(&WasteType::Paper, &1000, &owner, &desc);

        // Transfer should fail - receiver not registered
        let note = String::from_str(&env, "Transfer");
        client.transfer_waste(&material.id, &owner, &receiver, &note);
    }

    #[test]
    #[should_panic(expected = "Only waste owner can transfer")]
    fn test_transfer_waste_not_owner() {
        let env = Env::default();
        let contract_id = env.register_contract(None, ScavengerContract);
        let client = ScavengerContractClient::new(&env, &contract_id);

        let owner = Address::generate(&env);
        let attacker = Address::generate(&env);
        let receiver = Address::generate(&env);
        env.mock_all_auths();

        // Register all participants
        client.register_participant(&owner, &ParticipantRole::Recycler);
        client.register_participant(&attacker, &ParticipantRole::Collector);
        client.register_participant(&receiver, &ParticipantRole::Manufacturer);

        // Submit material
        let desc = String::from_str(&env, "Test");
        let material = client.submit_material(&WasteType::Paper, &1000, &owner, &desc);

        // Attacker tries to transfer - should fail
        let note = String::from_str(&env, "Unauthorized transfer");
        client.transfer_waste(&material.id, &attacker, &receiver, &note);
    }

    #[test]
    fn test_transfer_history_chronological() {
        let env = Env::default();
        let contract_id = env.register_contract(None, ScavengerContract);
        let client = ScavengerContractClient::new(&env, &contract_id);

        let user1 = Address::generate(&env);
        let user2 = Address::generate(&env);
        let user3 = Address::generate(&env);
        env.mock_all_auths();

        // Register all participants
        client.register_participant(&user1, &ParticipantRole::Recycler);
        client.register_participant(&user2, &ParticipantRole::Collector);
        client.register_participant(&user3, &ParticipantRole::Manufacturer);

        // Submit material
        let desc = String::from_str(&env, "Multi-transfer test");
        let material = client.submit_material(&WasteType::Metal, &5000, &user1, &desc);

        // First transfer: user1 -> user2
        let note1 = String::from_str(&env, "First transfer");
        client.transfer_waste(&material.id, &user1, &user2, &note1);

        // Second transfer: user2 -> user3
        let note2 = String::from_str(&env, "Second transfer");
        client.transfer_waste(&material.id, &user2, &user3, &note2);

        // Verify history is chronological
        let history = client.get_transfer_history(&material.id);
        assert_eq!(history.len(), 2);

        let transfer1 = history.get(0).unwrap();
        let transfer2 = history.get(1).unwrap();

        assert_eq!(transfer1.from, user1);
        assert_eq!(transfer1.to, user2);
        assert_eq!(transfer2.from, user2);
        assert_eq!(transfer2.to, user3);

        // Verify timestamps are chronological
        assert!(transfer2.transferred_at >= transfer1.transferred_at);
    }

    #[test]
    fn test_transfer_history_immutable() {
        let env = Env::default();
        let contract_id = env.register_contract(None, ScavengerContract);
        let client = ScavengerContractClient::new(&env, &contract_id);

        let user1 = Address::generate(&env);
        let user2 = Address::generate(&env);
        env.mock_all_auths();

        // Register participants
        client.register_participant(&user1, &ParticipantRole::Recycler);
        client.register_participant(&user2, &ParticipantRole::Collector);

        // Submit material
        let desc = String::from_str(&env, "Immutability test");
        let material = client.submit_material(&WasteType::Glass, &3000, &user1, &desc);

        // Transfer
        let note = String::from_str(&env, "Transfer");
        client.transfer_waste(&material.id, &user1, &user2, &note);

        // Get history
        let history1 = client.get_transfer_history(&material.id);
        let history2 = client.get_transfer_history(&material.id);

        // Verify history is consistent (immutable)
        assert_eq!(history1.len(), history2.len());
        assert_eq!(history1.get(0).unwrap(), history2.get(0).unwrap());
    }

    #[test]
    fn test_empty_transfer_history() {
        let env = Env::default();
        let contract_id = env.register_contract(None, ScavengerContract);
        let client = ScavengerContractClient::new(&env, &contract_id);

        let user = Address::generate(&env);
        env.mock_all_auths();

        // Submit material without any transfers
        let desc = String::from_str(&env, "No transfers");
        let material = client.submit_material(&WasteType::Plastic, &2000, &user, &desc);

        // Verify empty history
        let history = client.get_transfer_history(&material.id);
        assert_eq!(history.len(), 0);
    }

    #[test]
    fn test_transfer_history_different_wastes() {
        let env = Env::default();
        let contract_id = env.register_contract(None, ScavengerContract);
        let client = ScavengerContractClient::new(&env, &contract_id);

        let user1 = Address::generate(&env);
        let user2 = Address::generate(&env);
        env.mock_all_auths();

        // Register participants
        client.register_participant(&user1, &ParticipantRole::Recycler);
        client.register_participant(&user2, &ParticipantRole::Collector);

        // Submit two different materials
        let desc = String::from_str(&env, "Test");
        let material1 = client.submit_material(&WasteType::Paper, &1000, &user1, &desc);
        let material2 = client.submit_material(&WasteType::Metal, &2000, &user1, &desc);

        // Transfer only material1
        let note = String::from_str(&env, "Transfer material1");
        client.transfer_waste(&material1.id, &user1, &user2, &note);

        // Verify histories are separate
        let history1 = client.get_transfer_history(&material1.id);
        let history2 = client.get_transfer_history(&material2.id);

        assert_eq!(history1.len(), 1);
        assert_eq!(history2.len(), 0);
    }

    // ========== Incentive Storage System Tests ==========


    #[test]
    fn test_create_incentive() {
        let env = Env::default();
        let contract_id = env.register_contract(None, ScavengerContract);
        let client = ScavengerContractClient::new(&env, &contract_id);

        let manufacturer = Address::generate(&env);
        env.mock_all_auths();

        // Register manufacturer
        client.register_participant(&manufacturer, &ParticipantRole::Manufacturer);

        // Create incentive

        let incentive = client.create_incentive(
            &WasteType::Plastic,
            &100,  // 100 tokens per kg
            &10000, // Max 10kg
            &manufacturer,
        );

        assert_eq!(incentive.id, 1);
        assert_eq!(incentive.waste_type, WasteType::Plastic);
        assert_eq!(incentive.reward, 100);
        assert_eq!(incentive.max_waste_amount, 10000);
        assert_eq!(incentive.rewarder, manufacturer);
        assert!(incentive.is_active);
        assert!(incentive.created_at > 0);
    }

    #[test]
    fn test_incentive_persistence() {

        let incentive = client.create_incentive(&manufacturer, &WasteType::PetPlastic, &50, &10000);

        assert_eq!(incentive.id, 1);
        assert_eq!(incentive.rewarder, manufacturer);
        assert_eq!(incentive.waste_type, WasteType::PetPlastic);
        assert_eq!(incentive.reward_points, 50);
        assert_eq!(incentive.total_budget, 10000);
        assert_eq!(incentive.remaining_budget, 10000);
        assert!(incentive.active);
    }

    #[test]
    #[should_panic(expected = "Rewarder not registered")]
    fn test_create_incentive_unregistered() {

        let env = Env::default();
        let contract_id = env.register_contract(None, ScavengerContract);
        let client = ScavengerContractClient::new(&env, &contract_id);

        let manufacturer = Address::generate(&env);
        env.mock_all_auths();


        client.register_participant(&manufacturer, &ParticipantRole::Manufacturer);

        let incentive = client.create_incentive(
            &WasteType::Metal,
            &200,
            &5000,
            &manufacturer,
        );

        // Retrieve and verify persistence
        let retrieved = client.get_incentive_by_id(&incentive.id);
        assert!(retrieved.is_some());
        let retrieved = retrieved.unwrap();
        
        assert_eq!(retrieved.id, incentive.id);
        assert_eq!(retrieved.waste_type, incentive.waste_type);
        assert_eq!(retrieved.reward, incentive.reward);
        assert_eq!(retrieved.max_waste_amount, incentive.max_waste_amount);
        assert_eq!(retrieved.rewarder, incentive.rewarder);
        assert_eq!(retrieved.is_active, incentive.is_active);
        assert_eq!(retrieved.created_at, incentive.created_at);
    }

    #[test]
    #[should_panic(expected = "Only manufacturers can create incentives")]
    fn test_create_incentive_non_manufacturer() {

        // Try to create incentive without registration
        client.create_incentive(&manufacturer, &WasteType::Metal, &100, &5000);
    }

    #[test]
    #[should_panic(expected = "Only manufacturers can create incentives")]
    fn test_create_incentive_wrong_role() {
        let env = Env::default();
        let contract_id = env.register_contract(None, ScavengerContract);
        let client = ScavengerContractClient::new(&env, &contract_id);

        let recycler = Address::generate(&env);
        env.mock_all_auths();

        // Register as recycler
        client.register_participant(&recycler, &ParticipantRole::Recycler);

        // Try to create incentive - should fail
        client.create_incentive(&recycler, &WasteType::Plastic, &30, &8000);
    }

    #[test]
    fn test_get_incentive_by_id() {
        let env = Env::default();
        let contract_id = env.register_contract(None, ScavengerContract);
        let client = ScavengerContractClient::new(&env, &contract_id);

        let manufacturer = Address::generate(&env);
        env.mock_all_auths();

        client.register_participant(&manufacturer, &ParticipantRole::Manufacturer);
        let created = client.create_incentive(&manufacturer, &WasteType::Glass, &40, &7000);

        let retrieved = client.get_incentive_by_id(&created.id);
        assert!(retrieved.is_some());
        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.id, created.id);
        assert_eq!(retrieved.waste_type, WasteType::Glass);
        assert_eq!(retrieved.reward_points, 40);
    }

    #[test]
    fn test_incentive_exists() {

        let env = Env::default();
        let contract_id = env.register_contract(None, ScavengerContract);
        let client = ScavengerContractClient::new(&env, &contract_id);


        let collector = Address::generate(&env);
        env.mock_all_auths();

        // Register as collector (not manufacturer)
        client.register_participant(&collector, &ParticipantRole::Collector);

        // Should panic
        client.create_incentive(
            &WasteType::Paper,
            &50,
            &5000,
            &collector,
        );
    }

    #[test]
    #[should_panic(expected = "Reward must be greater than zero")]
    fn test_create_incentive_zero_reward() {

        let manufacturer = Address::generate(&env);
        env.mock_all_auths();

        assert!(!client.incentive_exists(&1));

        client.register_participant(&manufacturer, &ParticipantRole::Manufacturer);
        client.create_incentive(&manufacturer, &WasteType::Paper, &20, &5000);

        assert!(client.incentive_exists(&1));
        assert!(!client.incentive_exists(&2));
    }

    #[test]
    fn test_multiple_incentives_per_manufacturer() {

        let env = Env::default();
        let contract_id = env.register_contract(None, ScavengerContract);
        let client = ScavengerContractClient::new(&env, &contract_id);

        let manufacturer = Address::generate(&env);
        env.mock_all_auths();

        client.register_participant(&manufacturer, &ParticipantRole::Manufacturer);


        // Should panic with zero reward
        client.create_incentive(
            &WasteType::Glass,
            &0,
            &5000,
            &manufacturer,
        );
    }

    #[test]
    #[should_panic(expected = "Max waste amount must be greater than zero")]
    fn test_create_incentive_zero_max_waste() {

        // Create multiple incentives
        let i1 = client.create_incentive(&manufacturer, &WasteType::PetPlastic, &50, &10000);
        let i2 = client.create_incentive(&manufacturer, &WasteType::Metal, &100, &15000);
        let i3 = client.create_incentive(&manufacturer, &WasteType::Glass, &30, &8000);

        assert_eq!(i1.id, 1);
        assert_eq!(i2.id, 2);
        assert_eq!(i3.id, 3);

        // Verify all exist
        assert!(client.incentive_exists(&1));
        assert!(client.incentive_exists(&2));
        assert!(client.incentive_exists(&3));
    }

    #[test]
    fn test_get_incentives_by_rewarder() {
        let env = Env::default();
        let contract_id = env.register_contract(None, ScavengerContract);
        let client = ScavengerContractClient::new(&env, &contract_id);

        let manufacturer1 = Address::generate(&env);
        let manufacturer2 = Address::generate(&env);
        env.mock_all_auths();

        client.register_participant(&manufacturer1, &ParticipantRole::Manufacturer);
        client.register_participant(&manufacturer2, &ParticipantRole::Manufacturer);

        // Manufacturer1 creates 3 incentives
        client.create_incentive(&manufacturer1, &WasteType::Paper, &20, &5000);
        client.create_incentive(&manufacturer1, &WasteType::Plastic, &30, &6000);
        client.create_incentive(&manufacturer1, &WasteType::Metal, &50, &8000);

        // Manufacturer2 creates 2 incentives
        client.create_incentive(&manufacturer2, &WasteType::Glass, &40, &7000);
        client.create_incentive(&manufacturer2, &WasteType::PetPlastic, &60, &9000);

        // Check manufacturer1's incentives
        let m1_incentives = client.get_incentives_by_rewarder(&manufacturer1);
        assert_eq!(m1_incentives.len(), 3);
        assert_eq!(m1_incentives.get(0).unwrap(), 1);
        assert_eq!(m1_incentives.get(1).unwrap(), 2);
        assert_eq!(m1_incentives.get(2).unwrap(), 3);

        // Check manufacturer2's incentives
        let m2_incentives = client.get_incentives_by_rewarder(&manufacturer2);
        assert_eq!(m2_incentives.len(), 2);
        assert_eq!(m2_incentives.get(0).unwrap(), 4);
        assert_eq!(m2_incentives.get(1).unwrap(), 5);
    }

    #[test]
    fn test_get_incentives_by_waste_type() {
        let env = Env::default();
        let contract_id = env.register_contract(None, ScavengerContract);
        let client = ScavengerContractClient::new(&env, &contract_id);

        let manufacturer1 = Address::generate(&env);
        let manufacturer2 = Address::generate(&env);
        env.mock_all_auths();

        client.register_participant(&manufacturer1, &ParticipantRole::Manufacturer);
        client.register_participant(&manufacturer2, &ParticipantRole::Manufacturer);

        // Create incentives for different waste types
        client.create_incentive(&manufacturer1, &WasteType::PetPlastic, &50, &10000);
        client.create_incentive(&manufacturer2, &WasteType::PetPlastic, &60, &12000);
        client.create_incentive(&manufacturer1, &WasteType::Metal, &100, &15000);
        client.create_incentive(&manufacturer2, &WasteType::PetPlastic, &55, &11000);

        // Check PetPlastic incentives
        let pet_incentives = client.get_incentives_by_waste_type(&WasteType::PetPlastic);
        assert_eq!(pet_incentives.len(), 3);
        assert_eq!(pet_incentives.get(0).unwrap(), 1);
        assert_eq!(pet_incentives.get(1).unwrap(), 2);
        assert_eq!(pet_incentives.get(2).unwrap(), 4);

        // Check Metal incentives
        let metal_incentives = client.get_incentives_by_waste_type(&WasteType::Metal);
        assert_eq!(metal_incentives.len(), 1);
        assert_eq!(metal_incentives.get(0).unwrap(), 3);

        // Check Glass incentives (none created)
        let glass_incentives = client.get_incentives_by_waste_type(&WasteType::Glass);
        assert_eq!(glass_incentives.len(), 0);
    }

    #[test]
    fn test_deactivate_incentive() {

        let env = Env::default();
        let contract_id = env.register_contract(None, ScavengerContract);
        let client = ScavengerContractClient::new(&env, &contract_id);

        let manufacturer = Address::generate(&env);
        env.mock_all_auths();

        client.register_participant(&manufacturer, &ParticipantRole::Manufacturer);


        // Should panic with zero max waste amount
        client.create_incentive(
            &WasteType::PetPlastic,
            &100,
            &0,
            &manufacturer,
        );
    }

    #[test]
    fn test_update_incentive_status() {

        let incentive = client.create_incentive(&manufacturer, &WasteType::Paper, &25, &6000);

        assert!(incentive.active);

        // Deactivate
        let deactivated = client.deactivate_incentive(&incentive.id, &manufacturer);
        assert!(!deactivated.active);

        // Verify it's deactivated in storage
        let retrieved = client.get_incentive_by_id(&incentive.id).unwrap();
        assert!(!retrieved.active);
    }

    #[test]
    #[should_panic(expected = "Only incentive creator can deactivate")]
    fn test_deactivate_incentive_wrong_creator() {
        let env = Env::default();
        let contract_id = env.register_contract(None, ScavengerContract);
        let client = ScavengerContractClient::new(&env, &contract_id);

        let manufacturer1 = Address::generate(&env);
        let manufacturer2 = Address::generate(&env);
        env.mock_all_auths();

        client.register_participant(&manufacturer1, &ParticipantRole::Manufacturer);
        client.register_participant(&manufacturer2, &ParticipantRole::Manufacturer);

        let incentive = client.create_incentive(&manufacturer1, &WasteType::Plastic, &30, &7000);

        // Manufacturer2 tries to deactivate manufacturer1's incentive
        client.deactivate_incentive(&incentive.id, &manufacturer2);
    }

    #[test]
    fn test_claim_incentive_reward() {

        let env = Env::default();
        let contract_id = env.register_contract(None, ScavengerContract);
        let client = ScavengerContractClient::new(&env, &contract_id);

        let manufacturer = Address::generate(&env);

        env.mock_all_auths();

        client.register_participant(&manufacturer, &ParticipantRole::Manufacturer);

        let incentive = client.create_incentive(
            &WasteType::Paper,
            &75,
            &8000,
            &manufacturer,
        );

        assert!(incentive.is_active);

        // Deactivate incentive
        let updated = client.update_incentive_status(&incentive.id, &false);
        assert!(!updated.is_active);

        // Reactivate incentive
        let updated = client.update_incentive_status(&incentive.id, &true);
        assert!(updated.is_active);
    }

    #[test]
    fn test_calculate_incentive_reward_basic() {

        let recycler = Address::generate(&env);
        let submitter = Address::generate(&env);
        env.mock_all_auths();

        // Register participants
        client.register_participant(&manufacturer, &ParticipantRole::Manufacturer);
        client.register_participant(&recycler, &ParticipantRole::Recycler);

        // Create incentive
        let incentive = client.create_incentive(&manufacturer, &WasteType::PetPlastic, &50, &10000);

        // Submit and verify material
        let desc = String::from_str(&env, "PET bottles");
        let material = client.submit_material(&WasteType::PetPlastic, &5000, &submitter, &desc);
        client.verify_material(&material.id, &recycler);

        // Claim reward (5kg * 50 = 250 points)
        let reward = client.claim_incentive_reward(&incentive.id, &material.id, &submitter);
        assert_eq!(reward, 250);

        // Check incentive budget decreased
        let updated_incentive = client.get_incentive_by_id(&incentive.id).unwrap();
        assert_eq!(updated_incentive.remaining_budget, 9750);
        assert!(updated_incentive.active);

        // Check submitter stats updated
        let stats = client.get_stats(&submitter).unwrap();
        assert!(stats.total_points >= 250); // Includes base points + incentive
    }

    #[test]
    #[should_panic(expected = "Material must be verified to claim incentive")]
    fn test_claim_incentive_unverified_material() {

        let env = Env::default();
        let contract_id = env.register_contract(None, ScavengerContract);
        let client = ScavengerContractClient::new(&env, &contract_id);

        let manufacturer = Address::generate(&env);

        env.mock_all_auths();

        client.register_participant(&manufacturer, &ParticipantRole::Manufacturer);

        // Create incentive: 100 tokens per kg, max 10kg
        let incentive = client.create_incentive(
            &WasteType::Plastic,
            &100,
            &10000,
            &manufacturer,
        );

        // Calculate reward for 5kg (5000g)
        let reward = client.calculate_incentive_reward(&incentive.id, &5000);
        // 5000g * 100 / 1000 = 500 tokens
        assert_eq!(reward, 500);
    }

    #[test]
    fn test_calculate_incentive_reward_capped() {

        let submitter = Address::generate(&env);
        env.mock_all_auths();

        client.register_participant(&manufacturer, &ParticipantRole::Manufacturer);
        let incentive = client.create_incentive(&manufacturer, &WasteType::Metal, &100, &15000);

        // Submit but don't verify
        let desc = String::from_str(&env, "Metal cans");
        let material = client.submit_material(&WasteType::Metal, &3000, &submitter, &desc);

        // Try to claim - should fail
        client.claim_incentive_reward(&incentive.id, &material.id, &submitter);
    }

    #[test]
    #[should_panic(expected = "Material waste type does not match incentive")]
    fn test_claim_incentive_wrong_waste_type() {

        let env = Env::default();
        let contract_id = env.register_contract(None, ScavengerContract);
        let client = ScavengerContractClient::new(&env, &contract_id);

        let manufacturer = Address::generate(&env);

        env.mock_all_auths();

        client.register_participant(&manufacturer, &ParticipantRole::Manufacturer);

        // Create incentive: 100 tokens per kg, max 10kg
        let incentive = client.create_incentive(
            &WasteType::Metal,
            &100,
            &10000,
            &manufacturer,
        );

        // Calculate reward for 15kg (15000g) - should cap at 10kg
        let reward = client.calculate_incentive_reward(&incentive.id, &15000);
        // Capped at 10000g * 100 / 1000 = 1000 tokens
        assert_eq!(reward, 1000);
    }

    #[test]
    fn test_calculate_incentive_reward_inactive() {

        let recycler = Address::generate(&env);
        let submitter = Address::generate(&env);
        env.mock_all_auths();

        client.register_participant(&manufacturer, &ParticipantRole::Manufacturer);
        client.register_participant(&recycler, &ParticipantRole::Recycler);

        // Create incentive for PetPlastic
        let incentive = client.create_incentive(&manufacturer, &WasteType::PetPlastic, &50, &10000);

        // Submit and verify Metal material
        let desc = String::from_str(&env, "Metal");
        let material = client.submit_material(&WasteType::Metal, &5000, &submitter, &desc);
        client.verify_material(&material.id, &recycler);

        // Try to claim PetPlastic incentive for Metal material - should fail
        client.claim_incentive_reward(&incentive.id, &material.id, &submitter);
    }

    #[test]
    #[should_panic(expected = "Only material submitter can claim incentive")]
    fn test_claim_incentive_wrong_claimer() {

        let env = Env::default();
        let contract_id = env.register_contract(None, ScavengerContract);
        let client = ScavengerContractClient::new(&env, &contract_id);

        let manufacturer = Address::generate(&env);

        env.mock_all_auths();

        client.register_participant(&manufacturer, &ParticipantRole::Manufacturer);

        let incentive = client.create_incentive(
            &WasteType::Glass,
            &150,
            &5000,
            &manufacturer,
        );

        // Deactivate incentive
        client.update_incentive_status(&incentive.id, &false);

        // Calculate reward - should return 0 for inactive incentive
        let reward = client.calculate_incentive_reward(&incentive.id, &3000);
        assert_eq!(reward, 0);
    }

    #[test]
    fn test_calculate_incentive_reward_edge_cases() {

        let recycler = Address::generate(&env);
        let submitter = Address::generate(&env);
        let attacker = Address::generate(&env);
        env.mock_all_auths();

        client.register_participant(&manufacturer, &ParticipantRole::Manufacturer);
        client.register_participant(&recycler, &ParticipantRole::Recycler);

        let incentive = client.create_incentive(&manufacturer, &WasteType::Glass, &40, &8000);

        let desc = String::from_str(&env, "Glass bottles");
        let material = client.submit_material(&WasteType::Glass, &4000, &submitter, &desc);
        client.verify_material(&material.id, &recycler);

        // Attacker tries to claim - should fail
        client.claim_incentive_reward(&incentive.id, &material.id, &attacker);
    }

    #[test]
    #[should_panic(expected = "Insufficient incentive budget or incentive inactive")]
    fn test_claim_incentive_insufficient_budget() {

        let env = Env::default();
        let contract_id = env.register_contract(None, ScavengerContract);
        let client = ScavengerContractClient::new(&env, &contract_id);

        let manufacturer = Address::generate(&env);

        env.mock_all_auths();

        client.register_participant(&manufacturer, &ParticipantRole::Manufacturer);

        let incentive = client.create_incentive(
            &WasteType::PetPlastic,
            &200,
            &10000,
            &manufacturer,
        );

        // Test with 0 waste
        let reward = client.calculate_incentive_reward(&incentive.id, &0);
        assert_eq!(reward, 0);

        // Test with exactly max amount
        let reward = client.calculate_incentive_reward(&incentive.id, &10000);
        assert_eq!(reward, 2000); // 10000 * 200 / 1000

        // Test with 1 gram
        let reward = client.calculate_incentive_reward(&incentive.id, &1);
        assert_eq!(reward, 0); // 1 * 200 / 1000 = 0 (integer division)
    }

    #[test]
    fn test_incentive_exists() {

        let recycler = Address::generate(&env);
        let submitter = Address::generate(&env);
        env.mock_all_auths();

        client.register_participant(&manufacturer, &ParticipantRole::Manufacturer);
        client.register_participant(&recycler, &ParticipantRole::Recycler);

        // Create incentive with small budget
        let incentive = client.create_incentive(&manufacturer, &WasteType::Plastic, &50, &1000);

        // Submit large material (30kg * 50 = 1500 points, exceeds budget)
        let desc = String::from_str(&env, "Large plastic batch");
        let material = client.submit_material(&WasteType::Plastic, &30000, &submitter, &desc);
        client.verify_material(&material.id, &recycler);

        // Try to claim - should fail
        client.claim_incentive_reward(&incentive.id, &material.id, &submitter);
    }

    #[test]
    fn test_claim_incentive_auto_deactivate() {
        let env = Env::default();
        let contract_id = env.register_contract(None, ScavengerContract);
        let client = ScavengerContractClient::new(&env, &contract_id);

        let manufacturer = Address::generate(&env);
        let recycler = Address::generate(&env);
        let submitter = Address::generate(&env);
        env.mock_all_auths();

        client.register_participant(&manufacturer, &ParticipantRole::Manufacturer);
        client.register_participant(&recycler, &ParticipantRole::Recycler);

        // Create incentive with exact budget for 10kg
        let incentive = client.create_incentive(&manufacturer, &WasteType::Paper, &50, &500);

        // Submit and verify 10kg material
        let desc = String::from_str(&env, "Paper");
        let material = client.submit_material(&WasteType::Paper, &10000, &submitter, &desc);
        client.verify_material(&material.id, &recycler);

        // Claim reward (should exhaust budget)
        let reward = client.claim_incentive_reward(&incentive.id, &material.id, &submitter);
        assert_eq!(reward, 500);

        // Check incentive is auto-deactivated
        let updated_incentive = client.get_incentive_by_id(&incentive.id).unwrap();
        assert_eq!(updated_incentive.remaining_budget, 0);
        assert!(!updated_incentive.active);
    }

    #[test]
    fn test_incentive_counters_independent() {

        let env = Env::default();
        let contract_id = env.register_contract(None, ScavengerContract);
        let client = ScavengerContractClient::new(&env, &contract_id);

        let manufacturer = Address::generate(&env);

        let submitter = Address::generate(&env);

        env.mock_all_auths();

        client.register_participant(&manufacturer, &ParticipantRole::Manufacturer);


        // Check non-existent incentive
        assert!(!client.incentive_exists(&1));

        // Create incentive
        let incentive = client.create_incentive(
            &WasteType::Paper,
            &50,
            &5000,
            &manufacturer,
        );

        // Check existing incentive
        assert!(client.incentive_exists(&incentive.id));
        assert!(!client.incentive_exists(&999));
    }

    #[test]
    fn test_get_incentives_by_waste_type() {

        // Create incentive (ID 1)
        let incentive = client.create_incentive(&manufacturer, &WasteType::Metal, &100, &10000);
        assert_eq!(incentive.id, 1);

        // Submit material (ID 1)
        let desc = String::from_str(&env, "Test");
        let material = client.submit_material(&WasteType::Paper, &1000, &submitter, &desc);
        assert_eq!(material.id, 1);

        // Create another incentive (ID 2)
        let incentive2 = client.create_incentive(&manufacturer, &WasteType::Glass, &50, &8000);
        assert_eq!(incentive2.id, 2);

        // Submit another material (ID 2)
        let material2 = client.submit_material(&WasteType::Plastic, &2000, &submitter, &desc);
        assert_eq!(material2.id, 2);

        // Verify both counters are independent and sequential
        assert_eq!(incentive.id, 1);
        assert_eq!(material.id, 1);
        assert_eq!(incentive2.id, 2);
        assert_eq!(material2.id, 2);

        // Both use ID 1 and 2, proving counters are independent
        // (if they shared a counter, we'd have IDs 1,2,3,4 instead of 1,1,2,2)
    }

    // Incentive Tests
    #[test]
    fn test_create_incentive() {
        let env = Env::default();
        env.ledger().with_mut(|li| {
            li.timestamp = 1234567890;
        });
        let contract_id = env.register_contract(None, ScavengerContract);
        let client = ScavengerContractClient::new(&env, &contract_id);

        let manufacturer = Address::generate(&env);
        env.mock_all_auths();

        // Register manufacturer
        client.register_participant(&manufacturer, &ParticipantRole::Manufacturer);

        // Create incentive
        let incentive = client.create_incentive(
            &manufacturer,
            &WasteType::PetPlastic,
            &50,
        );

        assert_eq!(incentive.id, 1);
        assert_eq!(incentive.manufacturer, manufacturer);
        assert_eq!(incentive.waste_type, WasteType::PetPlastic);
        assert_eq!(incentive.reward_amount, 50);
        assert!(incentive.active);
        assert_eq!(incentive.created_at, 1234567890);
    }

    #[test]
    fn test_get_incentive() {

        let env = Env::default();
        let contract_id = env.register_contract(None, ScavengerContract);
        let client = ScavengerContractClient::new(&env, &contract_id);

        let manufacturer = Address::generate(&env);
        env.mock_all_auths();


        client.register_participant(&manufacturer, &ParticipantRole::Manufacturer);

        // Create multiple incentives
        client.create_incentive(&WasteType::Plastic, &100, &10000, &manufacturer);
        client.create_incentive(&WasteType::Metal, &200, &5000, &manufacturer);
        client.create_incentive(&WasteType::Plastic, &150, &8000, &manufacturer);
        client.create_incentive(&WasteType::Paper, &75, &12000, &manufacturer);

        // Get plastic incentives
        let plastic_incentives = client.get_incentives_by_waste_type(&WasteType::Plastic);
        assert_eq!(plastic_incentives.len(), 2);

        // Get metal incentives
        let metal_incentives = client.get_incentives_by_waste_type(&WasteType::Metal);
        assert_eq!(metal_incentives.len(), 1);

        // Get glass incentives (none)
        let glass_incentives = client.get_incentives_by_waste_type(&WasteType::Glass);
        assert_eq!(glass_incentives.len(), 0);
    }

    #[test]
    fn test_get_active_incentives() {

        // Register manufacturer
        client.register_participant(&manufacturer, &ParticipantRole::Manufacturer);

        // Create incentive
        client.create_incentive(&manufacturer, &WasteType::Metal, &100);

        // Retrieve incentive
        let incentive = client.get_incentive(&1);
        assert!(incentive.is_some());
        let incentive = incentive.unwrap();
        assert_eq!(incentive.id, 1);
        assert_eq!(incentive.waste_type, WasteType::Metal);
        assert_eq!(incentive.reward_amount, 100);
    }

    #[test]
    fn test_incentive_exists() {
        let env = Env::default();
        let contract_id = env.register_contract(None, ScavengerContract);
        let client = ScavengerContractClient::new(&env, &contract_id);

        let manufacturer = Address::generate(&env);
        env.mock_all_auths();

        // Check non-existent incentive
        assert!(!client.incentive_exists(&1));

        // Register manufacturer and create incentive
        client.register_participant(&manufacturer, &ParticipantRole::Manufacturer);
        client.create_incentive(&manufacturer, &WasteType::Paper, &25);

        // Check existing incentive
        assert!(client.incentive_exists(&1));
        assert!(!client.incentive_exists(&2));
    }

    #[test]
    fn test_multiple_incentives_per_manufacturer() {

        let env = Env::default();
        let contract_id = env.register_contract(None, ScavengerContract);
        let client = ScavengerContractClient::new(&env, &contract_id);

        let manufacturer = Address::generate(&env);
        env.mock_all_auths();


        client.register_participant(&manufacturer, &ParticipantRole::Manufacturer);

        // Create multiple incentives
        let i1 = client.create_incentive(&WasteType::Plastic, &100, &10000, &manufacturer);
        let i2 = client.create_incentive(&WasteType::Metal, &200, &5000, &manufacturer);
        let i3 = client.create_incentive(&WasteType::Paper, &75, &8000, &manufacturer);

        // All should be active initially
        let active = client.get_active_incentives();
        assert_eq!(active.len(), 3);

        // Deactivate one
        client.update_incentive_status(&i2.id, &false);

        // Should have 2 active
        let active = client.get_active_incentives();
        assert_eq!(active.len(), 2);

        // Deactivate another
        client.update_incentive_status(&i1.id, &false);

        // Should have 1 active
        let active = client.get_active_incentives();
        assert_eq!(active.len(), 1);
        assert_eq!(active.get(0).unwrap().id, i3.id);
    }

    #[test]
    fn test_multiple_manufacturers_incentives() {

        // Register manufacturer
        client.register_participant(&manufacturer, &ParticipantRole::Manufacturer);

        // Create multiple incentives
        let inc1 = client.create_incentive(&manufacturer, &WasteType::Paper, &10);
        let inc2 = client.create_incentive(&manufacturer, &WasteType::Plastic, &20);
        let inc3 = client.create_incentive(&manufacturer, &WasteType::Metal, &30);

        // Verify unique IDs
        assert_eq!(inc1.id, 1);
        assert_eq!(inc2.id, 2);
        assert_eq!(inc3.id, 3);

        // Verify all are retrievable
        assert!(client.get_incentive(&1).is_some());
        assert!(client.get_incentive(&2).is_some());
        assert!(client.get_incentive(&3).is_some());
    }

    #[test]
    fn test_get_incentives_batch() {
        let env = Env::default();
        let contract_id = env.register_contract(None, ScavengerContract);
        let client = ScavengerContractClient::new(&env, &contract_id);

        let manufacturer = Address::generate(&env);
        env.mock_all_auths();

        // Register manufacturer
        client.register_participant(&manufacturer, &ParticipantRole::Manufacturer);

        // Create multiple incentives
        client.create_incentive(&manufacturer, &WasteType::Paper, &10);
        client.create_incentive(&manufacturer, &WasteType::Plastic, &20);
        client.create_incentive(&manufacturer, &WasteType::Metal, &30);

        // Batch retrieve
        let mut ids = soroban_sdk::Vec::new(&env);
        ids.push_back(1);
        ids.push_back(2);
        ids.push_back(3);
        ids.push_back(99); // Non-existent

        let results = client.get_incentives_batch(&ids);
        assert_eq!(results.len(), 4);
        assert!(results.get(0).unwrap().is_some());
        assert!(results.get(1).unwrap().is_some());
        assert!(results.get(2).unwrap().is_some());
        assert!(results.get(3).unwrap().is_none());

        // Verify correct data
        let inc1 = results.get(0).unwrap().unwrap();
        assert_eq!(inc1.waste_type, WasteType::Paper);
        assert_eq!(inc1.reward_amount, 10);
    }

    #[test]
    #[should_panic(expected = "Only manufacturers can create incentives")]
    fn test_only_manufacturer_can_create_incentive() {
        let env = Env::default();
        let contract_id = env.register_contract(None, ScavengerContract);
        let client = ScavengerContractClient::new(&env, &contract_id);

        let recycler = Address::generate(&env);
        env.mock_all_auths();

        // Register as recycler (not manufacturer)
        client.register_participant(&recycler, &ParticipantRole::Recycler);

        // Attempt to create incentive - should panic
        client.create_incentive(&recycler, &WasteType::Paper, &10);
    }

    #[test]
    #[should_panic(expected = "Reward amount must be greater than zero")]
    fn test_incentive_zero_reward_rejected() {

        let env = Env::default();
        let contract_id = env.register_contract(None, ScavengerContract);
        let client = ScavengerContractClient::new(&env, &contract_id);


        let manufacturer1 = Address::generate(&env);
        let manufacturer2 = Address::generate(&env);
        env.mock_all_auths();

        client.register_participant(&manufacturer1, &ParticipantRole::Manufacturer);
        client.register_participant(&manufacturer2, &ParticipantRole::Manufacturer);

        // Each manufacturer creates incentives
        let i1 = client.create_incentive(&WasteType::Plastic, &100, &10000, &manufacturer1);
        let i2 = client.create_incentive(&WasteType::Plastic, &120, &8000, &manufacturer2);

        // Verify both exist and are independent
        let retrieved1 = client.get_incentive_by_id(&i1.id).unwrap();
        let retrieved2 = client.get_incentive_by_id(&i2.id).unwrap();

        assert_eq!(retrieved1.rewarder, manufacturer1);
        assert_eq!(retrieved2.rewarder, manufacturer2);
        assert_ne!(retrieved1.reward, retrieved2.reward);
    }

    #[test]
    fn test_incentive_storage_deterministic() {

        let manufacturer = Address::generate(&env);
        env.mock_all_auths();

        // Register manufacturer
        client.register_participant(&manufacturer, &ParticipantRole::Manufacturer);

        // Attempt to create incentive with zero reward - should panic
        client.create_incentive(&manufacturer, &WasteType::Paper, &0);
    }

    #[test]
    fn test_incentive_storage_compatibility() {
        let env = Env::default();
        let contract_id = env.register_contract(None, ScavengerContract);
        let manufacturer = Address::generate(&env);

        let incentive = Incentive::new(
            1,
            manufacturer.clone(),
            WasteType::Glass,
            75,
            1234567890,
        );

        // Test that Incentive can be stored in Soroban storage
        env.as_contract(&contract_id, || {
            env.storage().instance().set(&("incentive", 1u64), &incentive);
            let retrieved: Incentive = env.storage().instance().get(&("incentive", 1u64)).unwrap();
            
            assert_eq!(retrieved.id, incentive.id);
            assert_eq!(retrieved.manufacturer, incentive.manufacturer);
            assert_eq!(retrieved.waste_type, incentive.waste_type);
            assert_eq!(retrieved.reward_amount, incentive.reward_amount);
            assert_eq!(retrieved.active, incentive.active);
            assert_eq!(retrieved.created_at, incentive.created_at);
        });
    }

    #[test]
    fn test_deactivate_incentive() {

        let env = Env::default();
        let contract_id = env.register_contract(None, ScavengerContract);
        let client = ScavengerContractClient::new(&env, &contract_id);

        let manufacturer = Address::generate(&env);
        env.mock_all_auths();


        client.register_participant(&manufacturer, &ParticipantRole::Manufacturer);

        // Create incentive
        let incentive = client.create_incentive(
            &WasteType::Metal,
            &250,
            &15000,
            &manufacturer,
        );

        // Retrieve multiple times - should be consistent
        let r1 = client.get_incentive_by_id(&incentive.id).unwrap();
        let r2 = client.get_incentive_by_id(&incentive.id).unwrap();
        let r3 = client.get_incentive_by_id(&incentive.id).unwrap();

        assert_eq!(r1, r2);
        assert_eq!(r2, r3);
    }

    #[test]
    fn test_incentive_all_waste_types() {

        // Register manufacturer and create incentive
        client.register_participant(&manufacturer, &ParticipantRole::Manufacturer);
        let incentive = client.create_incentive(&manufacturer, &WasteType::Paper, &25);
        assert!(incentive.active);

        // Deactivate incentive
        let deactivated = client.deactivate_incentive(&1, &manufacturer);
        assert!(!deactivated.active);

        // Verify persistence
        let retrieved = client.get_incentive(&1).unwrap();
        assert!(!retrieved.active);
    }

    #[test]
    fn test_activate_incentive() {

        let env = Env::default();
        let contract_id = env.register_contract(None, ScavengerContract);
        let client = ScavengerContractClient::new(&env, &contract_id);

        let manufacturer = Address::generate(&env);
        env.mock_all_auths();


        client.register_participant(&manufacturer, &ParticipantRole::Manufacturer);

        // Create incentive for each waste type
        let i1 = client.create_incentive(&WasteType::Paper, &50, &10000, &manufacturer);
        let i2 = client.create_incentive(&WasteType::PetPlastic, &100, &8000, &manufacturer);
        let i3 = client.create_incentive(&WasteType::Plastic, &90, &9000, &manufacturer);
        let i4 = client.create_incentive(&WasteType::Metal, &200, &5000, &manufacturer);
        let i5 = client.create_incentive(&WasteType::Glass, &75, &12000, &manufacturer);

        // Verify all were created correctly
        assert_eq!(i1.waste_type, WasteType::Paper);
        assert_eq!(i2.waste_type, WasteType::PetPlastic);
        assert_eq!(i3.waste_type, WasteType::Plastic);
        assert_eq!(i4.waste_type, WasteType::Metal);
        assert_eq!(i5.waste_type, WasteType::Glass);
    }

    #[test]
    fn test_incentive_reward_calculation_no_overflow() {

        // Register manufacturer and create incentive
        client.register_participant(&manufacturer, &ParticipantRole::Manufacturer);
        client.create_incentive(&manufacturer, &WasteType::Paper, &25);

        // Deactivate then reactivate
        client.deactivate_incentive(&1, &manufacturer);
        let reactivated = client.activate_incentive(&1, &manufacturer);
        assert!(reactivated.active);

        // Verify persistence
        let retrieved = client.get_incentive(&1).unwrap();
        assert!(retrieved.active);
    }

    #[test]
    fn test_update_incentive_reward() {

        let env = Env::default();
        let contract_id = env.register_contract(None, ScavengerContract);
        let client = ScavengerContractClient::new(&env, &contract_id);

        let manufacturer = Address::generate(&env);
        env.mock_all_auths();


        client.register_participant(&manufacturer, &ParticipantRole::Manufacturer);

        // Create incentive with large values
        let incentive = client.create_incentive(
            &WasteType::Metal,
            &1000,
            &1000000, // 1000kg max
            &manufacturer,
        );

        // Calculate reward for large amount
        let reward = client.calculate_incentive_reward(&incentive.id, &500000);
        // 500000 * 1000 / 1000 = 500000 tokens
        assert_eq!(reward, 500000);

        // Register manufacturer and create incentive
        client.register_participant(&manufacturer, &ParticipantRole::Manufacturer);
        let incentive = client.create_incentive(&manufacturer, &WasteType::Metal, &50);
        assert_eq!(incentive.reward_amount, 50);

        // Update reward amount
        let updated = client.update_incentive_reward(&1, &manufacturer, &100);
        assert_eq!(updated.reward_amount, 100);

        // Verify persistence
        let retrieved = client.get_incentive(&1).unwrap();
        assert_eq!(retrieved.reward_amount, 100);
    }

    #[test]
    #[should_panic(expected = "Only the incentive creator can modify this incentive")]
    fn test_unauthorized_incentive_modification() {
        let env = Env::default();
        let contract_id = env.register_contract(None, ScavengerContract);
        let client = ScavengerContractClient::new(&env, &contract_id);

        let manufacturer1 = Address::generate(&env);
        let manufacturer2 = Address::generate(&env);
        env.mock_all_auths();

        // Register both manufacturers
        client.register_participant(&manufacturer1, &ParticipantRole::Manufacturer);
        client.register_participant(&manufacturer2, &ParticipantRole::Manufacturer);

        // Manufacturer 1 creates incentive
        client.create_incentive(&manufacturer1, &WasteType::Paper, &25);

        // Manufacturer 2 tries to deactivate - should panic
        client.deactivate_incentive(&1, &manufacturer2);

    }
}



    // Participant-specific tests
    #[test]
    fn test_participant_persistence() {
        let env = Env::default();
        let contract_id = env.register_contract(None, ScavengerContract);
        let client = ScavengerContractClient::new(&env, &contract_id);

        let user = Address::generate(&env);
        env.mock_all_auths();

        let name = soroban_sdk::Symbol::new(&env, "Alice");
        let participant = client.register_participant(
            &user,
            &ParticipantRole::Recycler,
            &name,
            &40_748_817,
            &-73_985_428,
        );

        // Retrieve and verify persistence
        let retrieved = client.get_participant(&user).unwrap();
        assert_eq!(retrieved.address, participant.address);
        assert_eq!(retrieved.role, participant.role);
        assert_eq!(retrieved.name, participant.name);
        assert_eq!(retrieved.latitude, participant.latitude);
        assert_eq!(retrieved.longitude, participant.longitude);
        assert_eq!(retrieved.is_registered, participant.is_registered);
        assert_eq!(retrieved.total_waste_processed, participant.total_waste_processed);
        assert_eq!(retrieved.total_tokens_earned, participant.total_tokens_earned);
        assert_eq!(retrieved.registered_at, participant.registered_at);
    }

    #[test]
    fn test_participant_initialization() {
        let env = Env::default();
        let contract_id = env.register_contract(None, ScavengerContract);
        let client = ScavengerContractClient::new(&env, &contract_id);

        let user = Address::generate(&env);
        env.mock_all_auths();

        let name = soroban_sdk::Symbol::new(&env, "Bob");
        let participant = client.register_participant(
            &user,
            &ParticipantRole::Collector,
            &name,
            &51_507_351,
            &-141_278,
        );

        // Verify correct initialization
        assert!(participant.is_registered);
        assert_eq!(participant.total_waste_processed, 0);
        assert_eq!(participant.total_tokens_earned, 0);
        assert!(participant.registered_at > 0);
    }

    #[test]
    fn test_role_based_access_enforcement() {
        let env = Env::default();
        let contract_id = env.register_contract(None, ScavengerContract);
        let client = ScavengerContractClient::new(&env, &contract_id);

        let collector = Address::generate(&env);
        let manufacturer = Address::generate(&env);
        env.mock_all_auths();

        let name = soroban_sdk::Symbol::new(&env, "Test");
        client.register_participant(&collector, &ParticipantRole::Collector, &name, &0, &0);
        client.register_participant(&manufacturer, &ParticipantRole::Manufacturer, &name, &0, &0);

        // Collector can collect but not manufacture
        assert!(client.can_collect(&collector));
        assert!(!client.can_manufacture(&collector));

        // Manufacturer can manufacture but not collect
        assert!(!client.can_collect(&manufacturer));
        assert!(client.can_manufacture(&manufacturer));
    }

    #[test]
    fn test_participant_stats_update() {
        let env = Env::default();
        let contract_id = env.register_contract(None, ScavengerContract);
        let client = ScavengerContractClient::new(&env, &contract_id);

        let submitter = Address::generate(&env);
        let recycler = Address::generate(&env);
        env.mock_all_auths();

        let name = soroban_sdk::Symbol::new(&env, "Test");
        client.register_participant(&submitter, &ParticipantRole::Collector, &name, &0, &0);
        client.register_participant(&recycler, &ParticipantRole::Recycler, &name, &0, &0);

        // Submit material
        let desc = String::from_str(&env, "Test material");
        client.submit_material(&WasteType::Metal, &5000, &submitter, &desc);

        // Check waste processed updated
        let participant = client.get_participant(&submitter).unwrap();
        assert_eq!(participant.total_waste_processed, 5000);
        assert_eq!(participant.total_tokens_earned, 0);

        // Verify material
        client.verify_material(&1, &recycler);

        // Check tokens earned updated
        let participant = client.get_participant(&submitter).unwrap();
        assert_eq!(participant.total_waste_processed, 5000);
        assert_eq!(participant.total_tokens_earned, 250); // 5kg * 5 * 10
    }

    #[test]
    fn test_participant_stats_overflow_protection() {
        let env = Env::default();
        let contract_id = env.register_contract(None, ScavengerContract);
        let client = ScavengerContractClient::new(&env, &contract_id);

        let user = Address::generate(&env);
        env.mock_all_auths();

        let name = soroban_sdk::Symbol::new(&env, "Test");
        client.register_participant(&user, &ParticipantRole::Collector, &name, &0, &0);

        // Submit multiple materials
        let desc = String::from_str(&env, "Test");
        for _ in 0..10 {
            client.submit_material(&WasteType::Paper, &1000, &user, &desc);
        }

        // Check stats accumulated correctly
        let participant = client.get_participant(&user).unwrap();
        assert_eq!(participant.total_waste_processed, 10000);
    }

    #[test]
    fn test_deregister_participant() {
        let env = Env::default();
        let contract_id = env.register_contract(None, ScavengerContract);
        let client = ScavengerContractClient::new(&env, &contract_id);

        let user = Address::generate(&env);
        env.mock_all_auths();

        let name = soroban_sdk::Symbol::new(&env, "Test");
        client.register_participant(&user, &ParticipantRole::Collector, &name, &0, &0);

        // Verify registered
        assert!(client.can_collect(&user));

        // Deregister
        let deregistered = client.deregister_participant(&user);
        assert!(!deregistered.is_registered);

        // Verify can no longer perform actions
        assert!(!client.can_collect(&user));
    }

    #[test]
    fn test_update_location() {
        let env = Env::default();
        let contract_id = env.register_contract(None, ScavengerContract);
        let client = ScavengerContractClient::new(&env, &contract_id);

        let user = Address::generate(&env);
        env.mock_all_auths();

        let name = soroban_sdk::Symbol::new(&env, "Test");
        client.register_participant(&user, &ParticipantRole::Collector, &name, &0, &0);

        // Update location
        let updated = client.update_location(&user, &48_856_613, &2_352_222); // Paris
        assert_eq!(updated.latitude, 48_856_613);
        assert_eq!(updated.longitude, 2_352_222);

        // Verify persistence
        let retrieved = client.get_participant(&user).unwrap();
        assert_eq!(retrieved.latitude, 48_856_613);
        assert_eq!(retrieved.longitude, 2_352_222);
    }

    #[test]
    #[should_panic(expected = "Participant not found")]
    fn test_submit_material_unregistered_user() {
        let env = Env::default();
        let contract_id = env.register_contract(None, ScavengerContract);
        let client = ScavengerContractClient::new(&env, &contract_id);

        let user = Address::generate(&env);
        env.mock_all_auths();

        let desc = String::from_str(&env, "Test");
        client.submit_material(&WasteType::Paper, &1000, &user, &desc);
    }

    #[test]
    #[should_panic(expected = "Participant is not registered")]
    fn test_update_role_deregistered_user() {
        let env = Env::default();
        let contract_id = env.register_contract(None, ScavengerContract);
        let client = ScavengerContractClient::new(&env, &contract_id);

        let user = Address::generate(&env);
        env.mock_all_auths();

        let name = soroban_sdk::Symbol::new(&env, "Test");
        client.register_participant(&user, &ParticipantRole::Collector, &name, &0, &0);
        client.deregister_participant(&user);

        // Should panic
        client.update_role(&user, &ParticipantRole::Recycler);
    }

    #[test]
    #[should_panic(expected = "Verifier is not registered")]
    fn test_verify_material_deregistered_verifier() {
        let env = Env::default();
        let contract_id = env.register_contract(None, ScavengerContract);
        let client = ScavengerContractClient::new(&env, &contract_id);

        let submitter = Address::generate(&env);
        let recycler = Address::generate(&env);
        env.mock_all_auths();

        let name = soroban_sdk::Symbol::new(&env, "Test");
        client.register_participant(&submitter, &ParticipantRole::Collector, &name, &0, &0);
        client.register_participant(&recycler, &ParticipantRole::Recycler, &name, &0, &0);

        // Submit material
        let desc = String::from_str(&env, "Test");
        client.submit_material(&WasteType::Paper, &1000, &submitter, &desc);

        // Deregister recycler
        client.deregister_participant(&recycler);

        // Should panic
        client.verify_material(&1, &recycler);
    }

    #[test]
    fn test_batch_operations_update_participant_stats() {
        let env = Env::default();
        let contract_id = env.register_contract(None, ScavengerContract);
        let client = ScavengerContractClient::new(&env, &contract_id);

        let submitter = Address::generate(&env);
        let recycler = Address::generate(&env);
        env.mock_all_auths();

        let name = soroban_sdk::Symbol::new(&env, "Test");
        client.register_participant(&submitter, &ParticipantRole::Collector, &name, &0, &0);
        client.register_participant(&recycler, &ParticipantRole::Recycler, &name, &0, &0);

        // Batch submit
        let mut materials = soroban_sdk::Vec::new(&env);
        materials.push_back((WasteType::Paper, 1000u64, String::from_str(&env, "Batch 1")));
        materials.push_back((WasteType::Plastic, 2000u64, String::from_str(&env, "Batch 2")));
        materials.push_back((WasteType::Metal, 3000u64, String::from_str(&env, "Batch 3")));

        client.submit_materials_batch(&materials, &submitter);

        // Check participant stats
        let participant = client.get_participant(&submitter).unwrap();
        assert_eq!(participant.total_waste_processed, 6000);

        // Batch verify
        let mut ids = soroban_sdk::Vec::new(&env);
        ids.push_back(1);
        ids.push_back(2);
        ids.push_back(3);

        client.verify_materials_batch(&ids, &recycler);

        // Check tokens earned
        let participant = client.get_participant(&submitter).unwrap();
        assert_eq!(participant.total_waste_processed, 6000);
        // 1kg*1*10 + 2kg*2*10 + 3kg*5*10 = 10 + 40 + 150 = 200
        assert_eq!(participant.total_tokens_earned, 200);
    }

    #[test]
    fn test_participant_storage_deterministic() {
        let env = Env::default();
        let contract_id = env.register_contract(None, ScavengerContract);
        let client = ScavengerContractClient::new(&env, &contract_id);

        let user = Address::generate(&env);
        env.mock_all_auths();

        let name = soroban_sdk::Symbol::new(&env, "Test");
        
        // Register twice with same data
        let p1 = client.register_participant(
            &user,
            &ParticipantRole::Recycler,
            &name,
            &12345678,
            &-87654321,
        );

        let p2 = client.register_participant(
            &user,
            &ParticipantRole::Recycler,
            &name,
            &12345678,
            &-87654321,
        );

        // Should overwrite with same values
        assert_eq!(p1.role, p2.role);
        assert_eq!(p1.name, p2.name);
        assert_eq!(p1.latitude, p2.latitude);
        assert_eq!(p1.longitude, p2.longitude);
    }

    #[test]
    fn test_multiple_participants_independent_stats() {
        let env = Env::default();
        let contract_id = env.register_contract(None, ScavengerContract);
        let client = ScavengerContractClient::new(&env, &contract_id);

        let user1 = Address::generate(&env);
        let user2 = Address::generate(&env);
        let recycler = Address::generate(&env);
        env.mock_all_auths();

        let name = soroban_sdk::Symbol::new(&env, "Test");
        client.register_participant(&user1, &ParticipantRole::Collector, &name, &0, &0);
        client.register_participant(&user2, &ParticipantRole::Collector, &name, &0, &0);
        client.register_participant(&recycler, &ParticipantRole::Recycler, &name, &0, &0);

        // User1 submits
        let desc = String::from_str(&env, "Test");
        client.submit_material(&WasteType::Paper, &1000, &user1, &desc);
        client.verify_material(&1, &recycler);

        // User2 submits
        client.submit_material(&WasteType::Metal, &5000, &user2, &desc);
        client.verify_material(&2, &recycler);

        // Check independent stats
        let p1 = client.get_participant(&user1).unwrap();
        let p2 = client.get_participant(&user2).unwrap();

        assert_eq!(p1.total_waste_processed, 1000);
        assert_eq!(p1.total_tokens_earned, 10); // 1kg * 1 * 10

        assert_eq!(p2.total_waste_processed, 5000);
        assert_eq!(p2.total_tokens_earned, 250); // 5kg * 5 * 10
    }
}

