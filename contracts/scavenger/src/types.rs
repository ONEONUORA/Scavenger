use soroban_sdk::{contracttype, Address, String};

/// Participant role in the scavenger system
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Role {
    Recycler,
    Collector,
    Manufacturer,
}

/// Participant information
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Participant {
    pub address: Address,
    pub role: Role,
    pub name: String,
    pub latitude: i64,
    pub longitude: i64,
    pub registered_at: u64,
}
