use soroban_sdk::{symbol_short, Address, Env, String, Symbol};

use crate::types::{Role, WasteType};

const PARTICIPANT_REGISTERED: Symbol = symbol_short!("reg");
const INCENTIVE_SET: Symbol = symbol_short!("inc_set");
const TOKENS_REWARDED: Symbol = symbol_short!("rewarded");

/// Emit event when a participant registers
pub fn emit_participant_registered(
    env: &Env,
    address: &Address,
    role: &Role,
    name: &String,
    latitude: i64,
    longitude: i64,
) {
    env.events().publish(
        (PARTICIPANT_REGISTERED, address),
        (role, name, latitude, longitude),
    );
}

/// Emit event when an incentive is created
pub fn emit_incentive_set(
    env: &Env,
    incentive_id: u64,
    rewarder: &Address,
    waste_type: WasteType,
    reward_points: u64,
    total_budget: u64,
) {
    env.events().publish(
        (INCENTIVE_SET, incentive_id),
        (rewarder, waste_type, reward_points, total_budget),
    );
}

/// Emit event when tokens are rewarded
pub fn emit_tokens_rewarded(
    env: &Env,
    waste_id: u64,
    recipient: &Address,
    amount: i128,
) {
    env.events().publish(
        (TOKENS_REWARDED, waste_id),
        (recipient, amount),
    );
}
