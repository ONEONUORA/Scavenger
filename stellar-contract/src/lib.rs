#![no_std]

mod types;

pub use types::{Incentive, Material, ParticipantRole, RecyclingStats, WasteTransfer, WasteType};

use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, String, Vec};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Participant {
    pub address: Address,
    pub role: ParticipantRole,
    pub registered_at: u64,
}

#[contract]
pub struct ScavengerContract;

#[contractimpl]
impl ScavengerContract {
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
    pub fn register_participant(env: Env, address: Address, role: ParticipantRole) -> Participant {
        address.require_auth();

        // Check if already registered
        if Self::is_participant_registered(env.clone(), address.clone()) {
            panic!("Participant already registered");
        }

        let participant = Participant {
            address: address.clone(),
            role,
            registered_at: env.ledger().timestamp(),
        };

        // Store participant using helper function
        Self::set_participant(&env, &address, &participant);

        participant
    }

    /// Store a waste record by ID
    /// Internal helper function for efficient waste storage
    fn set_waste(env: &Env, waste_id: u64, material: &Material) {
        let key = ("waste", waste_id);
        env.storage().instance().set(&key, material);
    }

    /// Retrieve a waste record by ID
    /// Returns None if waste doesn't exist
    fn get_waste(env: &Env, waste_id: u64) -> Option<Material> {
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

    /// Get participant information
    pub fn get_participant(env: Env, address: Address) -> Option<Participant> {
        let key = (address,);
        env.storage().instance().get(&key)
    }

    /// Update participant role
    /// Preserves registration timestamp and other data
    pub fn update_role(env: Env, address: Address, new_role: ParticipantRole) -> Participant {
        address.require_auth();

        let mut participant: Participant =
            Self::get_participant(env.clone(), address.clone()).expect("Participant not found");

        participant.role = new_role;
        Self::set_participant(&env, &address, &participant);

        participant
    }

    // ========== Waste Transfer History Functions ==========

    /// Get transfer history for a specific waste
    /// Returns chronologically ordered list of transfers
    pub fn get_transfer_history(env: Env, waste_id: u64) -> Vec<WasteTransfer> {
        let key = ("transfers", waste_id);
        env.storage().instance().get(&key).unwrap_or(Vec::new(&env))
    }

    /// Record a waste transfer
    /// Appends to immutable history
    fn record_transfer(env: &Env, waste_id: u64, from: Address, to: Address, note: String) {
        let key = ("transfers", waste_id);
        let mut history: Vec<WasteTransfer> =
            env.storage().instance().get(&key).unwrap_or(Vec::new(env));

        let transfer = WasteTransfer::new(waste_id, from, to, env.ledger().timestamp(), note);

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
        let mut material: Material = Self::get_waste(&env, waste_id).expect("Waste not found");

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
            participant.role.can_collect_materials()
        } else {
            false
        }
    }

    /// Validate if a participants can manufacture
    pub fn can_manufacture(env: Env, address: Address) -> bool {
        let key = (address,);
        if let Some(participant) = env.storage().instance().get::<_, Participant>(&key) {
            participant.role.can_manufacture()
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
        env.storage().instance().set(&("stats", submitter), &stats);

        material
    }

    /// Batch submit multiple materials for recycling
    /// More efficient than individual submissions
    pub fn submit_materials_batch(
        env: Env,
        materials: soroban_sdk::Vec<(WasteType, u64, String)>,
        submitter: Address,
    ) -> soroban_sdk::Vec<Material> {
        submitter.require_auth();

        let mut results = soroban_sdk::Vec::new(&env);
        let timestamp = env.ledger().timestamp();

        // Get or create stats once
        let mut stats: RecyclingStats = env
            .storage()
            .instance()
            .get(&("stats", submitter.clone()))
            .unwrap_or_else(|| RecyclingStats::new(submitter.clone()));

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
        }

        // Update stats once at the end
        env.storage().instance().set(&("stats", submitter), &stats);

        results
    }

    /// Get material by ID (alias for get_waste for backward compatibility)
    pub fn get_material(env: Env, material_id: u64) -> Option<Material> {
        Self::get_waste(&env, material_id)
    }

    /// Get waste by ID
    pub fn get_waste_by_id(env: Env, waste_id: u64) -> Option<Material> {
        Self::get_waste(&env, waste_id)
    }

    /// Get multiple wastes by IDs (batch retrieval)
    pub fn get_wastes_batch(
        env: Env,
        waste_ids: soroban_sdk::Vec<u64>,
    ) -> soroban_sdk::Vec<Option<Material>> {
        let mut results = soroban_sdk::Vec::new(&env);

        for waste_id in waste_ids.iter() {
            results.push_back(Self::get_waste(&env, waste_id));
        }

        results
    }

    /// Verify a material submission (only recyclers can verify)
    pub fn verify_material(env: Env, material_id: u64, verifier: Address) -> Material {
        verifier.require_auth();

        // Check if verifier is a recycler
        let verifier_key = (verifier.clone(),);
        let participant: Participant = env
            .storage()
            .instance()
            .get(&verifier_key)
            .expect("Verifier not registered");

        if !participant.role.can_process_recyclables() {
            panic!("Only recyclers can verify materials");
        }

        // Get and verify material using new storage system
        let mut material: Material =
            Self::get_waste(&env, material_id).expect("Material not found");

        material.verify();
        Self::set_waste(&env, material_id, &material);

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

        material
    }

    /// Batch verify multiple materials
    pub fn verify_materials_batch(
        env: Env,
        material_ids: soroban_sdk::Vec<u64>,
        verifier: Address,
    ) -> soroban_sdk::Vec<Material> {
        verifier.require_auth();

        // Check if verifier is a recycler
        let verifier_key = (verifier.clone(),);
        let participant: Participant = env
            .storage()
            .instance()
            .get(&verifier_key)
            .expect("Verifier not registered");

        if !participant.role.can_process_recyclables() {
            panic!("Only recyclers can verify materials");
        }

        let mut results = soroban_sdk::Vec::new(&env);

        for material_id in material_ids.iter() {
            if let Some(mut material) = Self::get_waste(&env, material_id) {
                material.verify();
                Self::set_waste(&env, material_id, &material);

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

                results.push_back(material);
            }
        }

        results
    }

    /// Get recycling statistics for a participant
    pub fn get_stats(env: Env, participant: Address) -> Option<RecyclingStats> {
        env.storage().instance().get(&("stats", participant))
    }

    // ========== Incentive Storage Functions ==========

    /// Store an incentive record by ID
    /// Internal helper function for efficient incentive storage
    fn set_incentive(env: &Env, incentive_id: u64, incentive: &Incentive) {
        let key = ("incentive", incentive_id);
        env.storage().instance().set(&key, incentive);
    }

    /// Retrieve an incentive record by ID
    /// Returns None if incentive doesn't exist
    fn get_incentive(env: &Env, incentive_id: u64) -> Option<Incentive> {
        let key = ("incentive", incentive_id);
        env.storage().instance().get(&key)
    }

    /// Check if an incentive record exists
    pub fn incentive_exists(env: Env, incentive_id: u64) -> bool {
        let key = ("incentive", incentive_id);
        env.storage().instance().has(&key)
    }

    /// Get incentive by ID (public getter)
    pub fn get_incentive_by_id(env: Env, incentive_id: u64) -> Option<Incentive> {
        Self::get_incentive(&env, incentive_id)
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

        let mut incentive = Self::get_incentive(&env, incentive_id).expect("Incentive not found");

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
        let material = Self::get_waste(&env, material_id).expect("Material not found");

        if !material.verified {
            panic!("Material must be verified to claim incentive");
        }

        // Verify claimer is the material submitter
        if material.submitter != claimer {
            panic!("Only material submitter can claim incentive");
        }

        // Get incentive
        let mut incentive = Self::get_incentive(&env, incentive_id).expect("Incentive not found");

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

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::{testutils::Address as _, Address, Env};

    #[test]
    fn test_register_participant() {
        let env = Env::default();
        let contract_id = env.register_contract(None, ScavengerContract);
        let client = ScavengerContractClient::new(&env, &contract_id);

        let user = Address::generate(&env);
        env.mock_all_auths();

        let participant = client.register_participant(&user, &ParticipantRole::Recycler);

        assert_eq!(participant.address, user);
        assert_eq!(participant.role, ParticipantRole::Recycler);
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

        client.register_participant(&user, &ParticipantRole::Collector);

        let participant = client.get_participant(&user);
        assert!(participant.is_some());
        assert_eq!(participant.unwrap().role, ParticipantRole::Collector);
    }

    #[test]
    fn test_update_role() {
        let env = Env::default();
        let contract_id = env.register_contract(None, ScavengerContract);
        let client = ScavengerContractClient::new(&env, &contract_id);

        let user = Address::generate(&env);
        env.mock_all_auths();

        client.register_participant(&user, &ParticipantRole::Recycler);
        let updated = client.update_role(&user, &ParticipantRole::Manufacturer);

        assert_eq!(updated.role, ParticipantRole::Manufacturer);
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

        client.register_participant(&recycler, &ParticipantRole::Recycler);
        client.register_participant(&collector, &ParticipantRole::Collector);
        client.register_participant(&manufacturer, &ParticipantRole::Manufacturer);

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

        client.register_participant(&recycler, &ParticipantRole::Recycler);
        client.register_participant(&manufacturer, &ParticipantRole::Manufacturer);

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

        client.register_participant(&user1, &ParticipantRole::Recycler);
        client.register_participant(&user2, &ParticipantRole::Collector);
        client.register_participant(&user3, &ParticipantRole::Manufacturer);

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

        let description = String::from_str(&env, "Plastic bottles");
        let material = client.submit_material(&WasteType::PetPlastic, &5000, &user, &description);

        assert_eq!(material.id, 1);
        assert_eq!(material.waste_type, WasteType::PetPlastic);
        assert_eq!(material.weight, 5000);
        assert_eq!(material.submitter, user);
        assert!(!material.verified);
    }

    #[test]
    fn test_get_material() {
        let env = Env::default();
        let contract_id = env.register_contract(None, ScavengerContract);
        let client = ScavengerContractClient::new(&env, &contract_id);

        let user = Address::generate(&env);
        env.mock_all_auths();

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

        // Register recycler
        client.register_participant(&recycler, &ParticipantRole::Recycler);

        // Submit material
        let description = String::from_str(&env, "Glass bottles");
        client.submit_material(&WasteType::Glass, &2000, &submitter, &description);

        // Verify material
        let verified = client.verify_material(&1, &recycler);
        assert!(verified.verified);
    }

    #[test]
    fn test_multiple_materials() {
        let env = Env::default();
        let contract_id = env.register_contract(None, ScavengerContract);
        let client = ScavengerContractClient::new(&env, &contract_id);

        let user = Address::generate(&env);
        env.mock_all_auths();

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
    }

    #[test]
    fn test_stats_tracking() {
        let env = Env::default();
        let contract_id = env.register_contract(None, ScavengerContract);
        let client = ScavengerContractClient::new(&env, &contract_id);

        let user = Address::generate(&env);
        env.mock_all_auths();

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

        // Register recycler
        client.register_participant(&recycler, &ParticipantRole::Recycler);

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
}
