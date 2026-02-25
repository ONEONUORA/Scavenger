use soroban_sdk::{contracttype, Address, String};

/// Participant role in the scavenger system
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Role {
    Recycler,
    Collector,
    Manufacturer,
}

impl Role {
    /// Check if role can manufacture products
    pub fn can_manufacture(&self) -> bool {
        matches!(self, Role::Manufacturer)
    }
}

/// Waste type in the recycling system
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WasteType {
    Paper = 0,
    PetPlastic = 1,
    Plastic = 2,
    Metal = 3,
    Glass = 4,
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

/// Incentive program created by manufacturers
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Incentive {
    /// Unique identifier for the incentive
    pub id: u64,
    /// Address of the manufacturer offering the incentive
    pub rewarder: Address,
    /// Type of waste this incentive targets
    pub waste_type: WasteType,
    /// Reward points per kilogram
    pub reward_points: u64,
    /// Total points budget allocated for this incentive
    pub total_budget: u64,
    /// Remaining points budget available
    pub remaining_budget: u64,
    /// Whether the incentive is currently active
    pub active: bool,
    /// Timestamp when the incentive was created
    pub created_at: u64,
}

impl Incentive {
    /// Creates a new Incentive instance
    pub fn new(
        id: u64,
        rewarder: Address,
        waste_type: WasteType,
        reward_points: u64,
        total_budget: u64,
        created_at: u64,
    ) -> Self {
        Self {
            id,
            rewarder,
            waste_type,
            reward_points,
            total_budget,
            remaining_budget: total_budget,
            active: true,
            created_at,
        }
    }
}

/// Waste material submission
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Material {
    /// Unique identifier for the material
    pub id: u64,
    /// Type of waste material
    pub waste_type: WasteType,
    /// Weight of the material in grams
    pub weight: u64,
    /// Address of the participant who submitted the material
    pub submitter: Address,
    /// Current owner of the material
    pub current_owner: Address,
    /// Timestamp when the material was submitted
    pub submitted_at: u64,
    /// Whether the material has been verified
    pub verified: bool,
    /// Whether the material is active (can be deactivated by admin)
    pub is_active: bool,
}

impl Material {
    /// Creates a new Material instance
    pub fn new(
        id: u64,
        waste_type: WasteType,
        weight: u64,
        submitter: Address,
        submitted_at: u64,
    ) -> Self {
        Self {
            id,
            waste_type,
            weight,
            submitter: submitter.clone(),
            current_owner: submitter,
            submitted_at,
            verified: false,
            is_active: true,
        }
    }
}

/// Waste transfer record
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WasteTransfer {
    /// ID of the waste being transferred
    pub waste_id: u64,
    /// Address of the sender
    pub from: Address,
    /// Address of the receiver
    pub to: Address,
    /// Timestamp of the transfer
    pub transferred_at: u64,
}

impl WasteTransfer {
    /// Creates a new WasteTransfer instance
    pub fn new(
        waste_id: u64,
        from: Address,
        to: Address,
        transferred_at: u64,
    ) -> Self {
        Self {
            waste_id,
            from,
            to,
            transferred_at,
        }
    }
}

/// Participant statistics
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParticipantStats {
    /// Participant address
    pub address: Address,
    /// Total tokens earned
    pub total_earned: i128,
    /// Number of materials submitted
    pub materials_submitted: u64,
    /// Number of transfers participated in
    pub transfers_count: u64,
}

impl ParticipantStats {
    /// Creates a new ParticipantStats instance
    pub fn new(address: Address) -> Self {
        Self {
            address,
            total_earned: 0,
            materials_submitted: 0,
            transfers_count: 0,
        }
    }
}
/// Global contract metrics
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GlobalMetrics {
    /// Total number of waste items logged in the system
    pub total_wastes_count: u64,
    /// Total amount of tokens earned across all participants
    pub total_tokens_earned: i128,
}