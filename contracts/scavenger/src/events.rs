use soroban_sdk::{symbol_short, Address, Env, String, Symbol};

use crate::types::{Role, WasteType};

const PARTICIPANT_REGISTERED: Symbol = symbol_short!("reg");
const INCENTIVE_SET: Symbol = symbol_short!("inc_set");
const INCENTIVE_UPDATED: Symbol = symbol_short!("inc_upd");
const TOKENS_REWARDED: Symbol = symbol_short!("rewarded");
const WASTE_DEACTIVATED: Symbol = symbol_short!("wst_deact");
const WASTE_CONFIRMED: Symbol = symbol_short!("wst_conf");
const WASTE_CONFIRMATION_RESET: Symbol = symbol_short!("wst_rst");

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

/// Emit event when an incentive is updated
pub fn emit_incentive_updated(
    env: &Env,
    incentive_id: u64,
    rewarder: &Address,
    new_reward_points: u64,
    new_total_budget: u64,
) {
    env.events().publish(
        (INCENTIVE_UPDATED, incentive_id),
        (rewarder, new_reward_points, new_total_budget),
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

/// Emit event when waste is deactivated
pub fn emit_waste_deactivated(
    env: &Env,
    waste_id: u64,
    admin: &Address,
) {
    env.events().publish(
        (WASTE_DEACTIVATED, waste_id),
        admin,
    );
}

/// Emit event when waste is confirmed
pub fn emit_waste_confirmed(
    env: &Env,
    waste_id: u64,
    confirmer: &Address,
) {
    env.events().publish(
        (WASTE_CONFIRMED, waste_id),
        confirmer,
    );
}

/// Emit event when waste confirmation is reset
pub fn emit_waste_confirmation_reset(
    env: &Env,
    waste_id: u64,
    owner: &Address,
) {
    env.events().publish(
        (WASTE_CONFIRMATION_RESET, waste_id),
        owner,
    );
}
