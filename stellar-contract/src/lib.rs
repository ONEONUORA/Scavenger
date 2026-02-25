#![no_std]

mod events;
mod types;



pub use types::{
    Material, ParticipantRole, RecyclingStats, TransferItemType, TransferRecord, TransferStatus,
    Waste, WasteTransfer, WasteType,
};


use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Address, Env, String, Symbol, Vec};

// Storage keys
const ADMIN: Symbol = symbol_short!("ADMIN");
const CHARITY: Symbol = symbol_short!("CHARITY");
const COLLECTOR_PCT: Symbol = symbol_short!("COL_PCT");
const OWNER_PCT: Symbol = symbol_short!("OWN_PCT");
const TOTAL_WEIGHT: Symbol = symbol_short!("TOT_WGT");
const TOTAL_TOKENS: Symbol = symbol_short!("TOT_TKN");

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
}

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

    /// Donate tokens to charity
    /// Records the donation and emits an event for tracking
    pub fn donate_to_charity(
        env: Env,
        donor: Address,
        amount: i128,
    ) {
        donor.require_auth();

        // Validate amount
        if amount <= 0 {
            panic!("Donation amount must be greater than zero");
        }

        // Get charity contract address
        let charity_contract = env.storage()
            .instance()
            .get::<Symbol, Address>(&CHARITY)
            .expect("Charity contract not set");

        // Emit donation event
        events::emit_donation_made(&env, &donor, amount, &charity_contract);
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
    /// Prevents duplicate registrations
    pub fn register_participant(
        env: Env,
        address: Address,
        role: ParticipantRole,
        name: soroban_sdk::Symbol,
        latitude: i128,
        longitude: i128,
    ) -> Participant {
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
            
            // Update global total tokens if tokens were earned
            if tokens_earned > 0 {
                Self::add_to_total_tokens(env, tokens_earned as u128);
            }
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

    /// Convert waste type enum to a human-readable string.
    pub fn get_waste_type_string(env: Env, waste_type: WasteType) -> String {
        String::from_str(&env, waste_type.as_str())
    }

    /// Convert participant role enum to a human-readable string.
    pub fn get_participant_role_string(env: Env, role: ParticipantRole) -> String {
        String::from_str(&env, role.as_str())
    }

    /// Validate whether a transfer route is allowed between two participants.
    /// Valid routes:
    /// - Recycler -> Collector
    /// - Recycler -> Manufacturer
    /// - Collector -> Manufacturer
    pub fn is_valid_transfer(env: Env, from: Address, to: Address) -> bool {
        let from_participant: Option<Participant> = env.storage().instance().get(&(from,));
        let to_participant: Option<Participant> = env.storage().instance().get(&(to,));

        let (Some(from_p), Some(to_p)) = (from_participant, to_participant) else {
            return false;
        };

        if !from_p.is_registered || !to_p.is_registered {
            return false;
        }

        matches!(
            (from_p.role, to_p.role),
            (ParticipantRole::Recycler, ParticipantRole::Collector)
                | (ParticipantRole::Recycler, ParticipantRole::Manufacturer)
                | (ParticipantRole::Collector, ParticipantRole::Manufacturer)
        )
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
    fn get_incentive_internal(env: &Env, incentive_id: u64) -> Option<Incentive> {
        let key = ("incentive", incentive_id);
        env.storage().instance().get(&key)
    }

    /// Retrieve an incentive by ID (internal compatibility alias).
    fn get_incentive(env: &Env, incentive_id: u64) -> Option<Incentive> {
        Self::get_incentive_internal(env, incentive_id)
    }

    /// Check if an incentive record exists
    pub fn incentive_exists(env: Env, incentive_id: u64) -> bool {
        let key = ("incentive", incentive_id);
        env.storage().instance().has(&key)
    }

    /// Get global total weight
    fn get_total_weight(env: &Env) -> u64 {
        env.storage().instance().get(&TOTAL_WEIGHT).unwrap_or(0)
    }

    /// Add to global total weight
    fn add_to_total_weight(env: &Env, weight: u64) {
        let current = Self::get_total_weight(env);
        let new_total = current.checked_add(weight).expect("Overflow in total weight");
        env.storage().instance().set(&TOTAL_WEIGHT, &new_total);
    }

    /// Get global total tokens earned
    fn get_total_tokens(env: &Env) -> u128 {
        env.storage().instance().get(&TOTAL_TOKENS).unwrap_or(0)
    }

    /// Add to global total tokens
    fn add_to_total_tokens(env: &Env, tokens: u128) {
        let current = Self::get_total_tokens(env);
        let new_total = current.checked_add(tokens).expect("Overflow in total tokens");
        env.storage().instance().set(&TOTAL_TOKENS, &new_total);
    }

    /// Calculate total weight for active waste entries in v2 storage.
    /// Iterates once across the waste ID range to keep reads linear and allocation-free.
    fn get_total_active_waste_weight(env: &Env) -> u64 {
        let mut total_weight: u64 = 0;
        let total_wastes = Self::get_waste_count(env);

        for waste_id in 1..=total_wastes {
            if let Some(waste) = env
                .storage()
                .instance()
                .get::<_, Waste>(&("waste_v2", waste_id as u128))
            {
                if waste.is_active {
                    let weight = u64::try_from(waste.weight)
                        .expect("Waste weight exceeds u64 range");
                    total_weight = total_weight
                        .checked_add(weight)
                        .expect("Overflow in active waste total weight");
                }
            }
        }

        total_weight
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

    /// Update an existing incentive's reward and maximum waste amount
    /// Only the rewarder can update their incentive
    /// Only active incentives can be updated
    pub fn update_incentive(
        env: Env,
        incentive_id: u64,
        new_reward: u128,
        new_max_waste_amount: u128,
    ) -> Incentive {
        // Step 1: Retrieve incentive (existence check)
        let mut incentive: Incentive = Self::get_incentive(&env, incentive_id)
            .expect("Incentive not found");

        // Step 2: Authorization check
        incentive.rewarder.require_auth();

        // Step 3: Active status check
        if !incentive.is_active {
            panic!("Incentive is not active");
        }

        // Step 4: Input validation
        if new_reward == 0 {
            panic!("Reward must be greater than zero");
        }
        if new_max_waste_amount == 0 {
            panic!("Max waste amount must be greater than zero");
        }

        // Step 5: Update fields (atomic)
        incentive.reward = new_reward;
        incentive.max_waste_amount = new_max_waste_amount;

        // Step 6: Persist to storage
        Self::set_incentive(&env, incentive_id, &incentive);

        // Step 7: Emit event
        env.events().publish(
            (symbol_short!("inc_upd"), incentive_id),
            (incentive.rewarder.clone(), new_reward, new_max_waste_amount)
        );

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
    }

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

        // Update global total weight
        Self::add_to_total_weight(&env, weight);

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

        if !waste.is_active {
            panic!("Cannot transfer deactivated waste");
        }

        if !Self::is_valid_transfer(env.clone(), from.clone(), to.clone()) {
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

        if !waste.is_active {
            panic!("Cannot confirm deactivated waste");
        }

        if waste.current_owner == confirmer {
            panic!("Owner cannot confirm own waste");
        }

        if waste.is_confirmed {
            panic!("Waste already confirmed");
        }

        waste.confirm(confirmer.clone());
        env.storage().instance().set(&("waste_v2", waste_id), &waste);

        events::emit_waste_confirmed(&env, waste_id, &confirmer);

        waste
    }

    /// Reset waste confirmation status
    /// Only the waste owner can reset the confirmation
    pub fn reset_waste_confirmation(
        env: Env,
        waste_id: u128,
        owner: Address,
    ) -> types::Waste {
        owner.require_auth();

        let mut waste: types::Waste = env
            .storage()
            .instance()
            .get(&("waste_v2", waste_id))
            .expect("Waste not found");

        if waste.current_owner != owner {
            panic!("Only owner can reset confirmation");
        }

        if !waste.is_confirmed {
            panic!("Waste is not confirmed");
        }

        waste.reset_confirmation();
        env.storage().instance().set(&("waste_v2", waste_id), &waste);

        env.events().publish(
            (soroban_sdk::symbol_short!("reset"), waste_id),
            (owner, env.ledger().timestamp()),
        );

        waste
    }

    /// Deactivate a waste record (admin only)
    /// Deactivated waste cannot be queried or reactivated
    pub fn deactivate_waste(
        env: Env,
        waste_id: u128,
        admin: Address,
    ) -> types::Waste {
        Self::require_admin(&env, &admin);

        let mut waste: types::Waste = env
            .storage()
            .instance()
            .get(&("waste_v2", waste_id))
            .expect("Waste not found");

        if !waste.is_active {
            panic!("Waste already deactivated");
        }

        waste.deactivate();
        env.storage().instance().set(&("waste_v2", waste_id), &waste);

        env.events().publish(
            (soroban_sdk::symbol_short!("deactive"), waste_id),
            (admin, env.ledger().timestamp()),
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

    /// Get supply chain statistics (total wastes, total weight, total tokens earned)
    /// Returns a tuple of (total_wastes_count, total_weight_grams, total_tokens_earned)
    pub fn get_supply_chain_stats(env: Env) -> (u64, u64, u128) {
        let total_wastes = Self::get_waste_count(&env);
        let total_weight = Self::get_total_active_waste_weight(&env);
        let total_tokens = Self::get_total_tokens(&env);
        
        (total_wastes, total_weight, total_tokens)
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

}
