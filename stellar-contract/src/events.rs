use soroban_sdk::{symbol_short, Address, Env, Symbol};

use crate::types::WasteType;

const WASTE_REGISTERED: Symbol = symbol_short!("recycled");

/// Emit event when waste is registered
pub fn emit_waste_registered(
    env: &Env,
    waste_id: u128,
    recycler: &Address,
    waste_type: WasteType,
    weight: u128,
    latitude: i128,
    longitude: i128,
) {
    env.events().publish(
        (WASTE_REGISTERED, waste_id),
        (waste_type, weight, recycler, latitude, longitude),
    );
}
