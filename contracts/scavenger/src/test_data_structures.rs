#![cfg(test)]

use soroban_sdk::{testutils::Address as _, Address, Env, String, symbol_short};
use crate::types::{Role, WasteType, Participant, Incentive, Material, WasteTransfer};
use crate::ScavengerContract;

#[test]
fn test_waste_type_enum() {
    let env = Env::default();
    let contract_id = env.register_contract(None, ScavengerContract);
    
    let types = [
        WasteType::Paper,
        WasteType::PetPlastic,
        WasteType::Plastic,
        WasteType::Metal,
        WasteType::Glass,
    ];

    env.as_contract(&contract_id, || {
        for (i, waste_type) in types.iter().enumerate() {
            let key = (symbol_short!("WT"), i as u32);
            env.storage().temporary().set(&key, waste_type);
            
            let retrieved: WasteType = env.storage().temporary().get(&key).unwrap();
            assert_eq!(*waste_type, retrieved);
        }
    });
}

#[test]
fn test_role_enum() {
    let env = Env::default();
    let contract_id = env.register_contract(None, ScavengerContract);
    
    let roles = [
        Role::Recycler,
        Role::Collector,
        Role::Manufacturer,
    ];

    env.as_contract(&contract_id, || {
        for (i, role) in roles.iter().enumerate() {
            let key = (symbol_short!("ROLE"), i as u32);
            env.storage().temporary().set(&key, role);
            
            let retrieved: Role = env.storage().temporary().get(&key).unwrap();
            assert_eq!(*role, retrieved);
        }
    });

    // Test helper method
    assert!(!Role::Recycler.can_manufacture());
    assert!(!Role::Collector.can_manufacture());
    assert!(Role::Manufacturer.can_manufacture());
}

#[test]
fn test_participant_serialization() {
    let env = Env::default();
    let contract_id = env.register_contract(None, ScavengerContract);
    let address = Address::generate(&env);
    let name = String::from_str(&env, "Green Earth Recycling");
    
    let participant = Participant {
        address: address.clone(),
        role: Role::Recycler,
        name: name.clone(),
        latitude: -123456789,
        longitude: 987654321,
        registered_at: 1700000000,
    };

    env.as_contract(&contract_id, || {
        let key = symbol_short!("PART");
        env.storage().temporary().set(&key, &participant);
        
        let retrieved: Participant = env.storage().temporary().get(&key).unwrap();
        
        assert_eq!(participant.address, retrieved.address);
        assert_eq!(participant.role, retrieved.role);
        assert_eq!(participant.name, retrieved.name);
        assert_eq!(participant.latitude, retrieved.latitude);
        assert_eq!(participant.longitude, retrieved.longitude);
        assert_eq!(participant.registered_at, retrieved.registered_at);
    });
}

#[test]
fn test_incentive_serialization() {
    let env = Env::default();
    let contract_id = env.register_contract(None, ScavengerContract);
    let rewarder = Address::generate(&env);
    
    let incentive = Incentive {
        id: 101,
        rewarder: rewarder.clone(),
        waste_type: WasteType::Metal,
        reward_points: 500,
        total_budget: 1000000,
        remaining_budget: 750000,
        active: true,
        created_at: 1700000000,
    };

    env.as_contract(&contract_id, || {
        let key = symbol_short!("INC");
        env.storage().temporary().set(&key, &incentive);
        
        let retrieved: Incentive = env.storage().temporary().get(&key).unwrap();
        
        assert_eq!(incentive, retrieved);
        assert_eq!(retrieved.id, 101);
        assert_eq!(retrieved.reward_points, 500);
    });
}

#[test]
fn test_material_serialization() {
    let env = Env::default();
    let contract_id = env.register_contract(None, ScavengerContract);
    let submitter = Address::generate(&env);
    let confirmer = Address::generate(&env);
    
    let material = Material {
        id: 50,
        waste_type: WasteType::Glass,
        weight: 2500, // 2.5kg
        submitter: submitter.clone(),
        current_owner: confirmer.clone(),
        submitted_at: 1700000000,
        verified: true,
        is_active: true,
        is_confirmed: true,
        confirmer: confirmer.clone(),
    };

    env.as_contract(&contract_id, || {
        let key = symbol_short!("MAT");
        env.storage().temporary().set(&key, &material);
        
        let retrieved: Material = env.storage().temporary().get(&key).unwrap();
        
        assert_eq!(material, retrieved);
        assert_eq!(retrieved.weight, 2500);
        assert!(retrieved.verified);
    });
}

#[test]
fn test_waste_transfer_serialization() {
    let env = Env::default();
    let contract_id = env.register_contract(None, ScavengerContract);
    let from = Address::generate(&env);
    let to = Address::generate(&env);
    
    let transfer = WasteTransfer {
        waste_id: 50,
        from: from.clone(),
        to: to.clone(),
        transferred_at: 1700000500,
    };

    env.as_contract(&contract_id, || {
        let key = symbol_short!("TRANS");
        env.storage().temporary().set(&key, &transfer);
        
        let retrieved: WasteTransfer = env.storage().temporary().get(&key).unwrap();
        
        assert_eq!(transfer, retrieved);
        assert_eq!(retrieved.waste_id, 50);
        assert_eq!(retrieved.from, from);
        assert_eq!(retrieved.to, to);
    });
}

#[test]
fn test_serialization_edge_cases() {
    let env = Env::default();
    let contract_id = env.register_contract(None, ScavengerContract);
    
    // Empty name in Participant
    let address = Address::generate(&env);
    let empty_name = String::from_str(&env, "");
    let p_empty = Participant {
        address: address.clone(),
        role: Role::Collector,
        name: empty_name,
        latitude: 0,
        longitude: 0,
        registered_at: 0,
    };
    
    env.as_contract(&contract_id, || {
        let key_p = symbol_short!("P_EDGE");
        env.storage().temporary().set(&key_p, &p_empty);
        let r_p: Participant = env.storage().temporary().get(&key_p).unwrap();
        assert_eq!(r_p.name.len(), 0);

        // Max values in Incentive
        let incentive_max = Incentive {
            id: u64::MAX,
            rewarder: address.clone(),
            waste_type: WasteType::Paper,
            reward_points: u64::MAX,
            total_budget: u64::MAX,
            remaining_budget: u64::MAX,
            active: false,
            created_at: u64::MAX,
        };
        
        let key_i = symbol_short!("I_MAX");
        env.storage().temporary().set(&key_i, &incentive_max);
        let r_i: Incentive = env.storage().temporary().get(&key_i).unwrap();
        assert_eq!(r_i.id, u64::MAX);
        assert_eq!(r_i.reward_points, u64::MAX);
    });
}

#[test]
fn test_participant_stats_serialization() {
    let env = Env::default();
    let contract_id = env.register_contract(None, ScavengerContract);
    let address = Address::generate(&env);
    
    let stats = crate::types::ParticipantStats {
        address: address.clone(),
        total_earned: 1000000,
        materials_submitted: 50,
        transfers_count: 25,
    };

    env.as_contract(&contract_id, || {
        let key = symbol_short!("STATS");
        env.storage().temporary().set(&key, &stats);
        
        let retrieved: crate::types::ParticipantStats = env.storage().temporary().get(&key).unwrap();
        
        assert_eq!(stats, retrieved);
        assert_eq!(retrieved.total_earned, 1000000);
    });
}

#[test]
fn test_global_metrics_serialization() {
    let env = Env::default();
    let contract_id = env.register_contract(None, ScavengerContract);
    
    let metrics = crate::types::GlobalMetrics {
        total_wastes_count: 5000,
        total_tokens_earned: 1000000000,
    };

    env.as_contract(&contract_id, || {
        let key = symbol_short!("METRICS");
        env.storage().temporary().set(&key, &metrics);
        
        let retrieved: crate::types::GlobalMetrics = env.storage().temporary().get(&key).unwrap();
        
        assert_eq!(metrics, retrieved);
        assert_eq!(retrieved.total_wastes_count, 5000);
    });
}
