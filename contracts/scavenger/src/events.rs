use soroban_sdk::{symbol_short, Address, Env, String, Symbol};

use crate::types::Role;

const PARTICIPANT_REGISTERED: Symbol = symbol_short!("reg");

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
