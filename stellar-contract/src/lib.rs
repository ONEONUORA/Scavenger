#![no_std]

mod types;

pub use types::{Incentive, Material, ParticipantRole, RecyclingStats, WasteType};

use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, String};

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
    /// Register a new participant with a specific role
    pub fn register_participant(
        env: Env,
        address: Address,
        role: ParticipantRole,
    ) -> Participant {
        address.require_auth();

        let participant = Participant {
            address: address.clone(),
            role,
            registered_at: env.ledger().timestamp(),
        };

        // Store participant in contract storage
        let key = (address.clone(),);
        env.storage().instance().set(&key, &participant);

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
    fn get_incentive_count(env: &Env) -> u64 {
        env.storage().instance().get(&("incentive_count",)).unwrap_or(0)
    }

    /// Increment and return the next incentive ID
    fn next_incentive_id(env: &Env) -> u64 {
        let count = Self::get_incentive_count(env);
        let next_id = count + 1;
        env.storage().instance().set(&("incentive_count",), &next_id);
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

    /// Get participant information
    pub fn get_participant(env: Env, address: Address) -> Option<Participant> {
        let key = (address,);
        env.storage().instance().get(&key)
    }

    /// Update participant role
    pub fn update_role(env: Env, address: Address, new_role: ParticipantRole) -> Participant {
        address.require_auth();

        let key = (address.clone(),);
        let mut participant: Participant = env
            .storage()
            .instance()
            .get(&key)
            .expect("Participant not found");

        participant.role = new_role;
        env.storage().instance().set(&key, &participant);

        participant
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

    /// Validate if a participant can manufacture
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

        // Store waste using the new storage system
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
    pub fn get_wastes_batch(env: Env, waste_ids: soroban_sdk::Vec<u64>) -> soroban_sdk::Vec<Option<Material>> {
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
        let mut material: Material = Self::get_waste(&env, material_id)
            .expect("Material not found");

        material.verify();
        Self::set_waste(&env, material_id, &material);

        // Update submitter stats
        let mut stats: RecyclingStats = env
            .storage()
            .instance()
            .get(&("stats", material.submitter.clone()))
            .unwrap_or_else(|| RecyclingStats::new(material.submitter.clone()));
        
        stats.record_verification(&material);
        env.storage().instance().set(&("stats", material.submitter.clone()), &stats);

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
                env.storage().instance().set(&("stats", material.submitter.clone()), &stats);

                results.push_back(material);
            }
        }

        results
    }

    /// Get recycling statistics for a participant
    pub fn get_stats(env: Env, participant: Address) -> Option<RecyclingStats> {
        env.storage().instance().get(&("stats", participant))
    }

    /// Migration helper: Get all participants (for data migration)
    /// Returns a vector of all registered participant addresses
    pub fn get_all_participant_addresses(env: Env) -> soroban_sdk::Vec<Address> {
        // Note: In production, you'd want to maintain a separate index
        // This is a placeholder for migration purposes
        soroban_sdk::Vec::new(&env)
    }

    /// Migration helper: Batch update participant roles
    /// Useful for role migrations or corrections
    pub fn batch_update_roles(
        env: Env,
        updates: soroban_sdk::Vec<(Address, ParticipantRole)>,
    ) -> soroban_sdk::Vec<Participant> {
        let mut results = soroban_sdk::Vec::new(&env);

        for update in updates.iter() {
            let (address, new_role) = update;
            address.require_auth();

            let key = (address.clone(),);
            if let Some(mut participant) = env.storage().instance().get::<_, Participant>(&key) {
                participant.role = new_role;
                env.storage().instance().set(&key, &participant);
                results.push_back(participant);
            }
        }

        results
    }

    /// Migration helper: Export participant data
    /// Returns participant data for backup/migration purposes
    pub fn export_participant(env: Env, address: Address) -> Option<(Address, ParticipantRole, u64)> {
        let key = (address.clone(),);
        env.storage().instance().get::<_, Participant>(&key).map(|p| {
            (p.address, p.role, p.registered_at)
        })
    }

    /// Migration helper: Import participant data
    /// Useful for restoring from backup or migrating from another contract
    pub fn import_participant(
        env: Env,
        address: Address,
        role: ParticipantRole,
        registered_at: u64,
    ) -> Participant {
        address.require_auth();

        let participant = Participant {
            address: address.clone(),
            role,
            registered_at,
        };

        let key = (address,);
        env.storage().instance().set(&key, &participant);

        participant
    }

    /// Verify participant data integrity
    /// Returns true if participant data is valid and consistent
    pub fn verify_participant_integrity(env: Env, address: Address) -> bool {
        let key = (address.clone(),);
        if let Some(participant) = env.storage().instance().get::<_, Participant>(&key) {
            // Verify data consistency
            participant.address == address
                && participant.registered_at > 0
                && ParticipantRole::is_valid(participant.role.to_u32())
        } else {
            false
        }
    }

    /// Create a new incentive program
    /// Only manufacturers can create incentives
    pub fn create_incentive(
        env: Env,
        manufacturer: Address,
        waste_type: WasteType,
        reward_amount: u64,
    ) -> Incentive {
        manufacturer.require_auth();

        // Verify manufacturer has Manufacturer role
        let manufacturer_key = (manufacturer.clone(),);
        let participant: Participant = env
            .storage()
            .instance()
            .get(&manufacturer_key)
            .expect("Manufacturer not registered");

        if !participant.role.can_manufacture() {
            panic!("Only manufacturers can create incentives");
        }

        // Validate reward amount
        if reward_amount == 0 {
            panic!("Reward amount must be greater than zero");
        }

        // Generate next incentive ID
        let incentive_id = Self::next_incentive_id(&env);

        // Create incentive
        let incentive = Incentive::new(
            incentive_id,
            manufacturer,
            waste_type,
            reward_amount,
            env.ledger().timestamp(),
        );

        // Store incentive
        Self::set_incentive(&env, incentive_id, &incentive);

        incentive
    }

    /// Get incentive by ID
    pub fn get_incentive(env: Env, incentive_id: u64) -> Option<Incentive> {
        Self::get_incentive_internal(&env, incentive_id)
    }

    /// Check if an incentive exists
    pub fn incentive_exists(env: Env, incentive_id: u64) -> bool {
        let key = ("incentive", incentive_id);
        env.storage().instance().has(&key)
    }

    /// Get multiple incentives by IDs (batch retrieval)
    pub fn get_incentives_batch(
        env: Env,
        incentive_ids: soroban_sdk::Vec<u64>,
    ) -> soroban_sdk::Vec<Option<Incentive>> {
        let mut results = soroban_sdk::Vec::new(&env);
        
        for incentive_id in incentive_ids.iter() {
            results.push_back(Self::get_incentive_internal(&env, incentive_id));
        }
        
        results
    }

    /// Deactivate an incentive
    /// Only the manufacturer who created the incentive can deactivate it
    pub fn deactivate_incentive(
        env: Env,
        incentive_id: u64,
        manufacturer: Address,
    ) -> Incentive {
        manufacturer.require_auth();

        // Retrieve incentive
        let mut incentive: Incentive = Self::get_incentive_internal(&env, incentive_id)
            .expect("Incentive not found");

        // Verify manufacturer matches
        if incentive.manufacturer != manufacturer {
            panic!("Only the incentive creator can modify this incentive");
        }

        // Deactivate
        incentive.deactivate();
        Self::set_incentive(&env, incentive_id, &incentive);

        incentive
    }

    /// Activate an incentive
    /// Only the manufacturer who created the incentive can activate it
    pub fn activate_incentive(
        env: Env,
        incentive_id: u64,
        manufacturer: Address,
    ) -> Incentive {
        manufacturer.require_auth();

        // Retrieve incentive
        let mut incentive: Incentive = Self::get_incentive_internal(&env, incentive_id)
            .expect("Incentive not found");

        // Verify manufacturer matches
        if incentive.manufacturer != manufacturer {
            panic!("Only the incentive creator can modify this incentive");
        }

        // Activate
        incentive.activate();
        Self::set_incentive(&env, incentive_id, &incentive);

        incentive
    }

    /// Update incentive reward amount
    /// Only the manufacturer who created the incentive can update it
    pub fn update_incentive_reward(
        env: Env,
        incentive_id: u64,
        manufacturer: Address,
        new_reward_amount: u64,
    ) -> Incentive {
        manufacturer.require_auth();

        // Validate new reward amount
        if new_reward_amount == 0 {
            panic!("Reward amount must be greater than zero");
        }

        // Retrieve incentive
        let mut incentive: Incentive = Self::get_incentive_internal(&env, incentive_id)
            .expect("Incentive not found");

        // Verify manufacturer matches
        if incentive.manufacturer != manufacturer {
            panic!("Only the incentive creator can modify this incentive");
        }

        // Update reward amount
        incentive.reward_amount = new_reward_amount;
        Self::set_incentive(&env, incentive_id, &incentive);

        incentive
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::{testutils::{Address as _, Ledger}, Address, Env};

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

        let participant = client.register_participant(&user, &ParticipantRole::Recycler);

        assert_eq!(participant.address, user);
        assert_eq!(participant.role, ParticipantRole::Recycler);
        assert!(participant.registered_at > 0);
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
        let env = Env::default();
        let contract_id = env.register_contract(None, ScavengerContract);
        
        // Test that WasteType can be stored and retrieved from storage
        let waste_types = [
            WasteType::Paper,
            WasteType::PetPlastic,
            WasteType::Plastic,
            WasteType::Metal,
            WasteType::Glass,
        ];

        for (i, waste_type) in waste_types.iter().enumerate() {
            env.as_contract(&contract_id, || {
                let key = (i as u32,);
                env.storage().instance().set(&key, waste_type);
                let retrieved: WasteType = env.storage().instance().get(&key).unwrap();
                assert_eq!(retrieved, *waste_type);
            });
        }
    }

    #[test]
    fn test_waste_type_serialization() {
        let env = Env::default();
        let contract_id = env.register_contract(None, ScavengerContract);
        
        // Test all waste types can be serialized/deserialized
        let all_types = [
            WasteType::Paper,
            WasteType::PetPlastic,
            WasteType::Plastic,
            WasteType::Metal,
            WasteType::Glass,
        ];

        for waste_type in all_types.iter() {
            env.as_contract(&contract_id, || {
                // Store in instance storage
                env.storage().instance().set(&("waste",), waste_type);
                let retrieved: WasteType = env.storage().instance().get(&("waste",)).unwrap();
                assert_eq!(retrieved, *waste_type);
                
                // Verify string representation
                assert!(!waste_type.as_str().is_empty());
            });
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
        let material = client.submit_material(
            &WasteType::PetPlastic,
            &5000,
            &user,
            &description,
        );

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
        materials.push_back((
            WasteType::Metal,
            3000u64,
            String::from_str(&env, "Batch 2"),
        ));

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
        let id1 = env.as_contract(&contract_id, || {
            ScavengerContract::next_incentive_id(&env)
        });
        assert_eq!(id1, 1);
        
        // Test second increment
        let id2 = env.as_contract(&contract_id, || {
            ScavengerContract::next_incentive_id(&env)
        });
        assert_eq!(id2, 2);
    }

    #[test]
    fn test_incentive_id_counter_increments_correctly() {
        let env = Env::default();
        let contract_id = env.register_contract(None, ScavengerContract);
        
        // Generate multiple IDs
        let id1 = env.as_contract(&contract_id, || {
            ScavengerContract::next_incentive_id(&env)
        });
        let id2 = env.as_contract(&contract_id, || {
            ScavengerContract::next_incentive_id(&env)
        });
        let id3 = env.as_contract(&contract_id, || {
            ScavengerContract::next_incentive_id(&env)
        });
        let id4 = env.as_contract(&contract_id, || {
            ScavengerContract::next_incentive_id(&env)
        });
        let id5 = env.as_contract(&contract_id, || {
            ScavengerContract::next_incentive_id(&env)
        });
        
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
        let id1 = env.as_contract(&contract_id, || {
            ScavengerContract::next_incentive_id(&env)
        });
        let id2 = env.as_contract(&contract_id, || {
            ScavengerContract::next_incentive_id(&env)
        });
        let id3 = env.as_contract(&contract_id, || {
            ScavengerContract::next_incentive_id(&env)
        });
        
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
        let id3 = env.as_contract(&contract_id, || {
            ScavengerContract::next_incentive_id(&env)
        });
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
        let i1 = env.as_contract(&contract_id, || {
            ScavengerContract::next_incentive_id(&env)
        });
        let i2 = env.as_contract(&contract_id, || {
            ScavengerContract::next_incentive_id(&env)
        });
        
        // Generate more waste IDs
        let m3 = client.submit_material(&WasteType::Metal, &3000, &user, &desc);
        
        // Generate more incentive IDs
        let i3 = env.as_contract(&contract_id, || {
            ScavengerContract::next_incentive_id(&env)
        });
        
        // Verify waste IDs are sequential
        assert_eq!(m1.id, 1);
        assert_eq!(m2.id, 2);
        assert_eq!(m3.id, 3);
        
        // Verify incentive IDs are sequential
        assert_eq!(i1, 1);
        assert_eq!(i2, 2);
        assert_eq!(i3, 3);
        
        // Verify counters are independent
        let waste_count = env.as_contract(&contract_id, || {
            ScavengerContract::get_waste_count(&env)
        });
        let incentive_count = env.as_contract(&contract_id, || {
            ScavengerContract::get_incentive_count(&env)
        });
        assert_eq!(waste_count, 3);
        assert_eq!(incentive_count, 3);
    }

    // Participant Serialization and Migration Tests
    #[test]
    fn test_participant_persistence() {
        let env = Env::default();
        env.ledger().with_mut(|li| {
            li.timestamp = 1234567890;
        });
        let contract_id = env.register_contract(None, ScavengerContract);
        let client = ScavengerContractClient::new(&env, &contract_id);

        let user = Address::generate(&env);
        env.mock_all_auths();

        // Register participant
        let participant = client.register_participant(&user, &ParticipantRole::Collector);
        
        // Retrieve participant - data should persist
        let retrieved = client.get_participant(&user);
        assert!(retrieved.is_some());
        let retrieved = retrieved.unwrap();
        
        assert_eq!(retrieved.address, participant.address);
        assert_eq!(retrieved.role, participant.role);
        assert_eq!(retrieved.registered_at, participant.registered_at);
    }

    #[test]
    fn test_participant_data_integrity() {
        let env = Env::default();
        env.ledger().with_mut(|li| {
            li.timestamp = 1234567890;
        });
        let contract_id = env.register_contract(None, ScavengerContract);
        let client = ScavengerContractClient::new(&env, &contract_id);

        let user = Address::generate(&env);
        env.mock_all_auths();

        // Register and verify integrity
        client.register_participant(&user, &ParticipantRole::Recycler);
        
        let is_valid = client.verify_participant_integrity(&user);
        assert!(is_valid);
    }

    #[test]
    fn test_participant_export_import() {
        let env = Env::default();
        env.ledger().with_mut(|li| {
            li.timestamp = 1234567890;
        });
        let contract_id = env.register_contract(None, ScavengerContract);
        let client = ScavengerContractClient::new(&env, &contract_id);

        let user = Address::generate(&env);
        env.mock_all_auths();

        // Register participant
        client.register_participant(&user, &ParticipantRole::Manufacturer);
        
        // Export participant data
        let exported = client.export_participant(&user);
        assert!(exported.is_some());
        let (addr, role, timestamp) = exported.unwrap();
        
        assert_eq!(addr, user);
        assert_eq!(role, ParticipantRole::Manufacturer);
        assert_eq!(timestamp, 1234567890);
        
        // Import to new address
        let new_user = Address::generate(&env);
        let imported = client.import_participant(&new_user, &role, &timestamp);
        
        assert_eq!(imported.address, new_user);
        assert_eq!(imported.role, role);
        assert_eq!(imported.registered_at, timestamp);
    }

    #[test]
    fn test_batch_update_roles() {
        let env = Env::default();
        env.ledger().with_mut(|li| {
            li.timestamp = 1234567890;
        });
        let contract_id = env.register_contract(None, ScavengerContract);
        let client = ScavengerContractClient::new(&env, &contract_id);

        let user1 = Address::generate(&env);
        let user2 = Address::generate(&env);
        let user3 = Address::generate(&env);
        env.mock_all_auths();

        // Register participants
        client.register_participant(&user1, &ParticipantRole::Collector);
        client.register_participant(&user2, &ParticipantRole::Collector);
        client.register_participant(&user3, &ParticipantRole::Collector);

        // Batch update roles
        let mut updates = soroban_sdk::Vec::new(&env);
        updates.push_back((user1.clone(), ParticipantRole::Recycler));
        updates.push_back((user2.clone(), ParticipantRole::Manufacturer));
        updates.push_back((user3.clone(), ParticipantRole::Recycler));

        let results = client.batch_update_roles(&updates);
        
        assert_eq!(results.len(), 3);
        assert_eq!(results.get(0).unwrap().role, ParticipantRole::Recycler);
        assert_eq!(results.get(1).unwrap().role, ParticipantRole::Manufacturer);
        assert_eq!(results.get(2).unwrap().role, ParticipantRole::Recycler);

        // Verify updates persisted
        assert_eq!(client.get_participant(&user1).unwrap().role, ParticipantRole::Recycler);
        assert_eq!(client.get_participant(&user2).unwrap().role, ParticipantRole::Manufacturer);
        assert_eq!(client.get_participant(&user3).unwrap().role, ParticipantRole::Recycler);
    }

    #[test]
    fn test_participant_with_stats_consistency() {
        let env = Env::default();
        env.ledger().with_mut(|li| {
            li.timestamp = 1234567890;
        });
        let contract_id = env.register_contract(None, ScavengerContract);
        let client = ScavengerContractClient::new(&env, &contract_id);

        let user = Address::generate(&env);
        env.mock_all_auths();

        // Register participant
        client.register_participant(&user, &ParticipantRole::Collector);
        
        // Submit materials
        let desc = String::from_str(&env, "Test");
        client.submit_material(&WasteType::Paper, &1000, &user, &desc);
        client.submit_material(&WasteType::Plastic, &2000, &user, &desc);

        // Verify participant data persists alongside stats
        let participant = client.get_participant(&user);
        let stats = client.get_stats(&user);
        
        assert!(participant.is_some());
        assert!(stats.is_some());
        
        let participant = participant.unwrap();
        let stats = stats.unwrap();
        
        assert_eq!(participant.address, user);
        assert_eq!(stats.participant, user);
        assert_eq!(stats.total_submissions, 2);
    }

    #[test]
    fn test_participant_role_update_preserves_data() {
        let env = Env::default();
        env.ledger().with_mut(|li| {
            li.timestamp = 1234567890;
        });
        let contract_id = env.register_contract(None, ScavengerContract);
        let client = ScavengerContractClient::new(&env, &contract_id);

        let user = Address::generate(&env);
        env.mock_all_auths();

        // Register participant
        let original = client.register_participant(&user, &ParticipantRole::Collector);
        let original_timestamp = original.registered_at;
        
        // Update role
        let updated = client.update_role(&user, &ParticipantRole::Recycler);
        
        // Verify role changed but other data preserved
        assert_eq!(updated.role, ParticipantRole::Recycler);
        assert_eq!(updated.address, user);
        assert_eq!(updated.registered_at, original_timestamp);
    }

    #[test]
    fn test_participant_serialization_all_roles() {
        let env = Env::default();
        env.ledger().with_mut(|li| {
            li.timestamp = 1234567890;
        });
        let contract_id = env.register_contract(None, ScavengerContract);
        let client = ScavengerContractClient::new(&env, &contract_id);

        let recycler = Address::generate(&env);
        let collector = Address::generate(&env);
        let manufacturer = Address::generate(&env);
        env.mock_all_auths();

        // Register all role types
        client.register_participant(&recycler, &ParticipantRole::Recycler);
        client.register_participant(&collector, &ParticipantRole::Collector);
        client.register_participant(&manufacturer, &ParticipantRole::Manufacturer);

        // Verify all can be retrieved (serialization works)
        let p1 = client.get_participant(&recycler);
        let p2 = client.get_participant(&collector);
        let p3 = client.get_participant(&manufacturer);

        assert!(p1.is_some());
        assert!(p2.is_some());
        assert!(p3.is_some());

        assert_eq!(p1.unwrap().role, ParticipantRole::Recycler);
        assert_eq!(p2.unwrap().role, ParticipantRole::Collector);
        assert_eq!(p3.unwrap().role, ParticipantRole::Manufacturer);
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
