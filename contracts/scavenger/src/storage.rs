use soroban_sdk::{symbol_short, Address, Env, Symbol, Vec};

use crate::types::{Incentive, Material, Participant, ParticipantStats, WasteTransfer, WasteType};

// Storage keys
const ADMIN: Symbol = symbol_short!("ADMIN");
const TOKEN_ADDR: Symbol = symbol_short!("TOKEN");
const CHARITY: Symbol = symbol_short!("CHARITY");
const COLLECTOR_PCT: Symbol = symbol_short!("COL_PCT");
const OWNER_PCT: Symbol = symbol_short!("OWN_PCT");
const TOTAL_EARNED: Symbol = symbol_short!("EARNED");
const INCENTIVE_COUNTER: Symbol = symbol_short!("INC_CNT");

pub struct Storage;

impl Storage {
    // Admin functions
    pub fn get_admin(env: &Env) -> Option<Address> {
        env.storage().instance().get(&ADMIN)
    }

    pub fn set_admin(env: &Env, admin: &Address) {
        env.storage().instance().set(&ADMIN, admin);
    }

    pub fn has_admin(env: &Env) -> bool {
        env.storage().instance().has(&ADMIN)
    }

    // Token address functions
    pub fn get_token_address(env: &Env) -> Option<Address> {
        env.storage().instance().get(&TOKEN_ADDR)
    }

    pub fn set_token_address(env: &Env, address: &Address) {
        env.storage().instance().set(&TOKEN_ADDR, address);
    }

    // Charity address functions
    pub fn get_charity_address(env: &Env) -> Option<Address> {
        env.storage().instance().get(&CHARITY)
    }

    pub fn set_charity_address(env: &Env, address: &Address) {
        env.storage().instance().set(&CHARITY, address);
    }

    // Collector percentage functions
    pub fn get_collector_percentage(env: &Env) -> Option<u32> {
        env.storage().instance().get(&COLLECTOR_PCT)
    }

    pub fn set_collector_percentage(env: &Env, percentage: u32) {
        env.storage().instance().set(&COLLECTOR_PCT, &percentage);
    }

    // Owner percentage functions
    pub fn get_owner_percentage(env: &Env) -> Option<u32> {
        env.storage().instance().get(&OWNER_PCT)
    }

    pub fn set_owner_percentage(env: &Env, percentage: u32) {
        env.storage().instance().set(&OWNER_PCT, &percentage);
    }

    // Total tokens earned functions
    pub fn get_total_earned(env: &Env) -> i128 {
        env.storage().instance().get(&TOTAL_EARNED).unwrap_or(0)
    }

    pub fn set_total_earned(env: &Env, amount: i128) {
        env.storage().instance().set(&TOTAL_EARNED, &amount);
    }

    pub fn add_to_total_earned(env: &Env, amount: i128) {
        let current = Self::get_total_earned(env);
        Self::set_total_earned(env, current + amount);
    }

    // Participant functions
    pub fn get_participant(env: &Env, address: &Address) -> Option<Participant> {
        let key = (symbol_short!("PART"), address);
        env.storage().instance().get(&key)
    }

    pub fn set_participant(env: &Env, address: &Address, participant: &Participant) {
        let key = (symbol_short!("PART"), address);
        env.storage().instance().set(&key, participant);
    }

    pub fn is_participant_registered(env: &Env, address: &Address) -> bool {
        let key = (symbol_short!("PART"), address);
        env.storage().instance().has(&key)
    }

    // Incentive counter functions
    pub fn next_incentive_id(env: &Env) -> u64 {
        let current: u64 = env.storage().instance().get(&INCENTIVE_COUNTER).unwrap_or(0);
        let next = current + 1;
        env.storage().instance().set(&INCENTIVE_COUNTER, &next);
        next
    }

    // Incentive storage functions
    pub fn set_incentive(env: &Env, incentive_id: u64, incentive: &Incentive) {
        let key = (symbol_short!("INC"), incentive_id);
        env.storage().instance().set(&key, incentive);
    }

    pub fn get_incentive(env: &Env, incentive_id: u64) -> Option<Incentive> {
        let key = (symbol_short!("INC"), incentive_id);
        env.storage().instance().get(&key)
    }

    pub fn incentive_exists(env: &Env, incentive_id: u64) -> bool {
        let key = (symbol_short!("INC"), incentive_id);
        env.storage().instance().has(&key)
    }

    // Rewarder incentives map (manufacturer -> Vec<incentive_ids>)
    pub fn add_incentive_to_rewarder(env: &Env, rewarder: &Address, incentive_id: u64) {
        let key = (symbol_short!("REW_INC"), rewarder);
        let mut incentives: Vec<u64> = env.storage().instance().get(&key).unwrap_or(Vec::new(env));
        incentives.push_back(incentive_id);
        env.storage().instance().set(&key, &incentives);
    }

    pub fn get_incentives_by_rewarder(env: &Env, rewarder: &Address) -> Vec<u64> {
        let key = (symbol_short!("REW_INC"), rewarder);
        env.storage().instance().get(&key).unwrap_or(Vec::new(env))
    }

    // General incentives map (waste_type -> Vec<incentive_ids>)
    pub fn add_incentive_to_waste_type(env: &Env, waste_type: WasteType, incentive_id: u64) {
        let key = (symbol_short!("GEN_INC"), waste_type);
        let mut incentives: Vec<u64> = env.storage().instance().get(&key).unwrap_or(Vec::new(env));
        incentives.push_back(incentive_id);
        env.storage().instance().set(&key, &incentives);
    }

    pub fn get_incentives_by_waste_type(env: &Env, waste_type: WasteType) -> Vec<u64> {
        let key = (symbol_short!("GEN_INC"), waste_type);
        env.storage().instance().get(&key).unwrap_or(Vec::new(env))
    }

    // Material storage functions
    pub fn next_material_id(env: &Env) -> u64 {
        let key = symbol_short!("MAT_CNT");
        let current: u64 = env.storage().instance().get(&key).unwrap_or(0);
        let next = current + 1;
        env.storage().instance().set(&key, &next);
        next
    }

    pub fn set_material(env: &Env, material_id: u64, material: &Material) {
        let key = (symbol_short!("MAT"), material_id);
        env.storage().instance().set(&key, material);
    }

    pub fn get_material(env: &Env, material_id: u64) -> Option<Material> {
        let key = (symbol_short!("MAT"), material_id);
        env.storage().instance().get(&key)
    }

    // Waste transfer history functions
    pub fn add_transfer(env: &Env, waste_id: u64, transfer: &WasteTransfer) {
        let key = (symbol_short!("TRANS"), waste_id);
        let mut transfers: Vec<WasteTransfer> = env.storage().instance().get(&key).unwrap_or(Vec::new(env));
        transfers.push_back(transfer.clone());
        env.storage().instance().set(&key, &transfers);
    }

    pub fn get_transfer_history(env: &Env, waste_id: u64) -> Vec<WasteTransfer> {
        let key = (symbol_short!("TRANS"), waste_id);
        env.storage().instance().get(&key).unwrap_or(Vec::new(env))
    }

    // Participant statistics functions
    pub fn get_stats(env: &Env, address: &Address) -> ParticipantStats {
        let key = (symbol_short!("STATS"), address);
        env.storage().instance().get(&key).unwrap_or_else(|| ParticipantStats::new(address.clone()))
    }

    pub fn set_stats(env: &Env, address: &Address, stats: &ParticipantStats) {
        let key = (symbol_short!("STATS"), address);
        env.storage().instance().set(&key, stats);
    }

    pub fn add_earnings(env: &Env, address: &Address, amount: i128) {
        let mut stats = Self::get_stats(env, address);
        stats.total_earned += amount;
        Self::set_stats(env, address, &stats);
    }
}
