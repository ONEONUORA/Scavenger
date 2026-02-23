use soroban_sdk::{symbol_short, Address, Env, Symbol};

// Storage keys
const ADMIN: Symbol = symbol_short!("ADMIN");
const TOKEN_ADDR: Symbol = symbol_short!("TOKEN");
const CHARITY: Symbol = symbol_short!("CHARITY");
const COLLECTOR_PCT: Symbol = symbol_short!("COL_PCT");
const OWNER_PCT: Symbol = symbol_short!("OWN_PCT");
const TOTAL_EARNED: Symbol = symbol_short!("EARNED");
const INITIALIZED: Symbol = symbol_short!("INIT");
const COUNTER: Symbol = symbol_short!("COUNTER");

pub struct Storage;

impl Storage {
    // Initialization flag
    pub fn is_initialized(env: &Env) -> bool {
        env.storage().instance().has(&INITIALIZED)
    }

    pub fn set_initialized(env: &Env) {
        env.storage().instance().set(&INITIALIZED, &true);
    }

    // Admin functions
    pub fn get_admin(env: &Env) -> Option<Address> {
        env.storage().instance().get(&ADMIN)
    }

    pub fn set_admin(env: &Env, admin: &Address) {
        env.storage().instance().set(&ADMIN, admin);
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

    // Counter functions
    pub fn get_counter(env: &Env) -> u64 {
        env.storage().instance().get(&COUNTER).unwrap_or(0)
    }

    pub fn set_counter(env: &Env, value: u64) {
        env.storage().instance().set(&COUNTER, &value);
    }
}
