#![no_std]

mod types;

pub use types::{ParticipantRole, WasteType};

use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, Map};

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
        
        // Test that WasteType can be stored and retrieved from storage
        let waste_types = [
            WasteType::Paper,
            WasteType::PetPlastic,
            WasteType::Plastic,
            WasteType::Metal,
            WasteType::Glass,
        ];

        for (i, waste_type) in waste_types.iter().enumerate() {
            let key = (i as u32,);
            env.storage().instance().set(&key, waste_type);
            let retrieved: WasteType = env.storage().instance().get(&key).unwrap();
            assert_eq!(retrieved, *waste_type);
        }
    }

    #[test]
    fn test_waste_type_serialization() {
        let env = Env::default();
        
        // Test all waste types can be serialized/deserialized
        let all_types = [
            WasteType::Paper,
            WasteType::PetPlastic,
            WasteType::Plastic,
            WasteType::Metal,
            WasteType::Glass,
        ];

        for waste_type in all_types.iter() {
            // Store in instance storage
            env.storage().instance().set(&("waste",), waste_type);
            let retrieved: WasteType = env.storage().instance().get(&("waste",)).unwrap();
            assert_eq!(retrieved, *waste_type);
            
            // Verify string representation
            assert!(!waste_type.as_str().is_empty());
        }
    }
}
