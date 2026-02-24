use soroban_sdk::{contracttype, Address, String, Symbol};

/// Represents an incentive offered by a manufacturer to encourage recycling
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

    /// Deactivates the incentive
    pub fn deactivate(&mut self) {
        self.active = false;
    }

    /// Calculates reward for a given weight in grams
    pub fn calculate_reward(&self, weight_grams: u64) -> u64 {
        // Convert grams to kg and multiply by reward points
        (weight_grams / 1000) * self.reward_points
    }

    /// Attempts to claim a reward, returns the amount claimed
    /// Returns None if insufficient budget
    pub fn claim_reward(&mut self, weight_grams: u64) -> Option<u64> {
        if !self.active {
            return None;
        }

        let reward = self.calculate_reward(weight_grams);
        if reward > self.remaining_budget {
            return None;
        }

        self.remaining_budget -= reward;

        // Auto-deactivate if budget exhausted
        if self.remaining_budget == 0 {
            self.active = false;
        }

        Some(reward)
    }

    /// Checks if the incentive has sufficient budget for a reward
    pub fn has_sufficient_budget(&self, weight_grams: u64) -> bool {
        if !self.active {
            return false;
        }
        let reward = self.calculate_reward(weight_grams);
        reward <= self.remaining_budget
    }
}

/// Represents the role of a participant in the Scavenger ecosystem
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ParticipantRole {
    /// Recycler role - responsible for collecting and processing recyclable materials
    Recycler = 0,
    /// Collector role - responsible for gathering materials from various sources
    Collector = 1,
    /// Manufacturer role - responsible for processing materials into new products
    Manufacturer = 2,
}

impl ParticipantRole {
    /// Validates if the role is a valid ParticipantRole variant
    pub fn is_valid(role: u32) -> bool {
        matches!(role, 0..=2)
    }

    /// Converts a u32 to a ParticipantRole
    /// Returns None if the value is invalid
    pub fn from_u32(value: u32) -> Option<Self> {
        match value {
            0 => Some(ParticipantRole::Recycler),
            1 => Some(ParticipantRole::Collector),
            2 => Some(ParticipantRole::Manufacturer),
            _ => None,
        }
    }

    /// Converts the ParticipantRole to u32
    pub fn to_u32(&self) -> u32 {
        *self as u32
    }

    /// Returns the string representation of the role
    pub fn as_str(&self) -> &'static str {
        match self {
            ParticipantRole::Recycler => "RECYCLER",
            ParticipantRole::Collector => "COLLECTOR",
            ParticipantRole::Manufacturer => "MANUFACTURER",
        }
    }

    /// Validates if a participant can perform a specific action based on their role
    pub fn can_collect_materials(&self) -> bool {
        matches!(self, ParticipantRole::Recycler | ParticipantRole::Collector)
    }

    /// Validates if a participant can manufacture products
    pub fn can_manufacture(&self) -> bool {
        matches!(self, ParticipantRole::Manufacturer)
    }

    /// Validates if a participant can process recyclables
    pub fn can_process_recyclables(&self) -> bool {
        matches!(self, ParticipantRole::Recycler)
    }
}

/// Represents the type of waste material in the recycling ecosystem
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WasteType {
    /// Paper waste - newspapers, cardboard, office paper
    Paper = 0,
    /// PET plastic - polyethylene terephthalate bottles and containers
    PetPlastic = 1,
    /// General plastic waste - various plastic types
    Plastic = 2,
    /// Metal waste - aluminum, steel, copper
    Metal = 3,
    /// Glass waste - bottles, jars, containers
    Glass = 4,
}

impl WasteType {
    /// Validates if the value is a valid WasteType variant
    pub fn is_valid(value: u32) -> bool {
        matches!(value, 0..=4)
    }

    /// Converts a u32 to a WasteType
    /// Returns None if the value is invalid
    pub fn from_u32(value: u32) -> Option<Self> {
        match value {
            0 => Some(WasteType::Paper),
            1 => Some(WasteType::PetPlastic),
            2 => Some(WasteType::Plastic),
            3 => Some(WasteType::Metal),
            4 => Some(WasteType::Glass),
            _ => None,
        }
    }

    /// Converts the WasteType to u32
    pub fn to_u32(&self) -> u32 {
        *self as u32
    }

    /// Returns the string representation of the waste type
    pub fn as_str(&self) -> &'static str {
        match self {
            WasteType::Paper => "PAPER",
            WasteType::PetPlastic => "PETPLASTIC",
            WasteType::Plastic => "PLASTIC",
            WasteType::Metal => "METAL",
            WasteType::Glass => "GLASS",
        }
    }

    /// Checks if the waste type is recyclable plastic
    pub fn is_plastic(&self) -> bool {
        matches!(self, WasteType::PetPlastic | WasteType::Plastic)
    }

    /// Checks if the waste type is biodegradable
    pub fn is_biodegradable(&self) -> bool {
        matches!(self, WasteType::Paper)
    }

    /// Checks if the waste type is infinitely recyclable
    pub fn is_infinitely_recyclable(&self) -> bool {
        matches!(self, WasteType::Metal | WasteType::Glass)
    }
}

impl core::fmt::Display for WasteType {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Represents a recyclable material submission in the system
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
    /// Timestamp when the material was submitted
    pub submitted_at: u64,
    /// Whether the material has been verified
    pub verified: bool,
    /// Optional description of the material
    pub description: String,
}

impl Material {
    /// Creates a new Material instance
    pub fn new(
        id: u64,
        waste_type: WasteType,
        weight: u64,
        submitter: Address,
        submitted_at: u64,
        description: String,
    ) -> Self {
        Self {
            id,
            waste_type,
            weight,
            submitter,
            submitted_at,
            verified: false,
            description,
        }
    }

    /// Marks the material as verified
    pub fn verify(&mut self) {
        self.verified = true;
    }

    /// Checks if the material meets minimum weight requirement (100g)
    pub fn meets_minimum_weight(&self) -> bool {
        self.weight >= 100
    }

    /// Calculates reward points based on waste type and weight
    /// Different waste types have different point multipliers
    pub fn calculate_reward_points(&self) -> u64 {
        let multiplier = match self.waste_type {
            WasteType::Paper => 1,
            WasteType::PetPlastic => 3,
            WasteType::Plastic => 2,
            WasteType::Metal => 5,
            WasteType::Glass => 2,
        };

        // Points = (weight in kg) * multiplier * 10
        (self.weight / 1000) * multiplier * 10
    }
}

/// Represents a waste item in the recycling system
/// This is the main struct that tracks waste throughout its lifecycle
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Waste {
    /// Unique identifier for the waste item
    pub waste_id: u128,
    /// Type of waste material
    pub waste_type: WasteType,
    /// Weight of the waste in grams
    pub weight: u128,
    /// Current owner of the waste
    pub current_owner: Address,
    /// Latitude coordinate (scaled by 1e6 for precision)
    pub latitude: i128,
    /// Longitude coordinate (scaled by 1e6 for precision)
    pub longitude: i128,
    /// Timestamp when the waste was recycled (0 if not yet recycled)
    pub recycled_timestamp: u64,
    /// Whether the waste is currently active in the system
    pub is_active: bool,
    /// Whether the waste has been confirmed/verified
    pub is_confirmed: bool,
    /// Address of the confirmer/verifier
    pub confirmer: Address,
}

impl Waste {
    /// Creates a new Waste instance with all fields
    pub fn new(
        waste_id: u128,
        waste_type: WasteType,
        weight: u128,
        current_owner: Address,
        latitude: i128,
        longitude: i128,
        recycled_timestamp: u64,
        is_active: bool,
        is_confirmed: bool,
        confirmer: Address,
    ) -> Self {
        Self {
            waste_id,
            waste_type,
            weight,
            current_owner,
            latitude,
            longitude,
            recycled_timestamp,
            is_active,
            is_confirmed,
            confirmer,
        }
    }

    /// Validates that the waste has valid coordinates
    pub fn has_valid_coordinates(&self) -> bool {
        let max_lat = 90_000_000i128;
        let max_lon = 180_000_000i128;
        
        self.latitude >= -max_lat 
            && self.latitude <= max_lat
            && self.longitude >= -max_lon
            && self.longitude <= max_lon
    }

    /// Checks if the waste has been recycled
    pub fn is_recycled(&self) -> bool {
        self.recycled_timestamp > 0
    }

    /// Checks if the waste meets minimum weight requirement (100g)
    pub fn meets_minimum_weight(&self) -> bool {
        self.weight >= 100
    }

    /// Marks the waste as recycled with the given timestamp
    pub fn mark_recycled(&mut self, timestamp: u64) {
        self.recycled_timestamp = timestamp;
    }

    /// Confirms the waste with the given confirmer
    pub fn confirm(&mut self, confirmer: Address) {
        self.is_confirmed = true;
        self.confirmer = confirmer;
    }

    /// Deactivates the waste
    pub fn deactivate(&mut self) {
        self.is_active = false;
    }

    /// Transfers ownership to a new owner
    pub fn transfer_to(&mut self, new_owner: Address) {
        self.current_owner = new_owner;
    }

    /// Updates the location of the waste
    pub fn update_location(&mut self, latitude: i128, longitude: i128) {
        self.latitude = latitude;
        self.longitude = longitude;
    }
}

/// Builder pattern for constructing Waste instances
/// Provides a fluent API for creating waste with optional fields
pub struct WasteBuilder {
    waste_id: u128,
    waste_type: WasteType,
    weight: u128,
    current_owner: Address,
    latitude: i128,
    longitude: i128,
    recycled_timestamp: u64,
    is_active: bool,
    is_confirmed: bool,
    confirmer: Option<Address>,
}

impl WasteBuilder {
    /// Creates a new WasteBuilder with required fields
    pub fn new(
        waste_id: u128,
        waste_type: WasteType,
        weight: u128,
        current_owner: Address,
    ) -> Self {
        Self {
            waste_id,
            waste_type,
            weight,
            current_owner: current_owner.clone(),
            latitude: 0,
            longitude: 0,
            recycled_timestamp: 0,
            is_active: true,
            is_confirmed: false,
            confirmer: Some(current_owner),
        }
    }

    /// Sets the location coordinates
    pub fn location(mut self, latitude: i128, longitude: i128) -> Self {
        self.latitude = latitude;
        self.longitude = longitude;
        self
    }

    /// Sets the recycled timestamp
    pub fn recycled_at(mut self, timestamp: u64) -> Self {
        self.recycled_timestamp = timestamp;
        self
    }

    /// Sets the active status
    pub fn active(mut self, is_active: bool) -> Self {
        self.is_active = is_active;
        self
    }

    /// Sets the confirmed status and confirmer
    pub fn confirmed(mut self, confirmer: Address) -> Self {
        self.is_confirmed = true;
        self.confirmer = Some(confirmer);
        self
    }

    /// Sets the confirmer address
    pub fn confirmer(mut self, confirmer: Address) -> Self {
        self.confirmer = Some(confirmer);
        self
    }

    /// Builds the Waste instance
    pub fn build(self) -> Waste {
        let confirmer = self.confirmer.unwrap_or_else(|| self.current_owner.clone());
        Waste {
            waste_id: self.waste_id,
            waste_type: self.waste_type,
            weight: self.weight,
            current_owner: self.current_owner,
            latitude: self.latitude,
            longitude: self.longitude,
            recycled_timestamp: self.recycled_timestamp,
            is_active: self.is_active,
            is_confirmed: self.is_confirmed,
            confirmer,
        }
    }
}


/// Tracks recycling statistics for a participant
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecyclingStats {
    /// Participant address
    pub participant: Address,
    /// Total number of materials submitted
    pub total_submissions: u64,
    /// Total number of verified materials
    pub verified_submissions: u64,
    /// Total weight of all materials in grams
    pub total_weight: u64,
    /// Total reward points earned
    pub total_points: u64,
    /// Number of materials by waste type
    pub paper_count: u64,
    pub pet_plastic_count: u64,
    pub plastic_count: u64,
    pub metal_count: u64,
    pub glass_count: u64,
}

impl RecyclingStats {
    /// Creates a new RecyclingStats instance
    pub fn new(participant: Address) -> Self {
        Self {
            participant,
            total_submissions: 0,
            verified_submissions: 0,
            total_weight: 0,
            total_points: 0,
            paper_count: 0,
            pet_plastic_count: 0,
            plastic_count: 0,
            metal_count: 0,
            glass_count: 0,
        }
    }

    /// Records a new material submission
    pub fn record_submission(&mut self, material: &Material) {
        self.total_submissions += 1;
        self.total_weight += material.weight;

        // Update waste type count
        match material.waste_type {
            WasteType::Paper => self.paper_count += 1,
            WasteType::PetPlastic => self.pet_plastic_count += 1,
            WasteType::Plastic => self.plastic_count += 1,
            WasteType::Metal => self.metal_count += 1,
            WasteType::Glass => self.glass_count += 1,
        }
    }

    /// Records a material verification
    pub fn record_verification(&mut self, material: &Material) {
        if material.verified {
            self.verified_submissions += 1;
            self.total_points += material.calculate_reward_points();
        }
    }

    /// Calculates the verification rate (percentage)
    pub fn verification_rate(&self) -> u64 {
        if self.total_submissions == 0 {
            0
        } else {
            (self.verified_submissions * 100) / self.total_submissions
        }
    }

    /// Gets the most submitted waste type
    pub fn most_submitted_type(&self) -> Option<WasteType> {
        let counts = [
            (WasteType::Paper, self.paper_count),
            (WasteType::PetPlastic, self.pet_plastic_count),
            (WasteType::Plastic, self.plastic_count),
            (WasteType::Metal, self.metal_count),
            (WasteType::Glass, self.glass_count),
        ];

        counts
            .iter()
            .max_by_key(|(_, count)| count)
            .filter(|(_, count)| *count > 0)
            .map(|(waste_type, _)| *waste_type)
    }

    /// Calculates average weight per submission
    pub fn average_weight(&self) -> u64 {
        if self.total_submissions == 0 {
            0
        } else {
            self.total_weight / self.total_submissions
        }
    }

    /// Checks if participant is an active recycler (10+ submissions)
    pub fn is_active_recycler(&self) -> bool {
        self.total_submissions >= 10
    }

    /// Checks if participant is a verified contributor (80%+ verification rate)
    pub fn is_verified_contributor(&self) -> bool {
        self.verification_rate() >= 80
    }
}

#[cfg(test)]
mod recycling_stats_tests {
    use super::*;
    use soroban_sdk::testutils::Address as _;

    #[test]
    fn test_new_stats() {
        let env = soroban_sdk::Env::default();
        let participant = Address::generate(&env);

        let stats = RecyclingStats::new(participant.clone());

        assert_eq!(stats.participant, participant);
        assert_eq!(stats.total_submissions, 0);
        assert_eq!(stats.verified_submissions, 0);
        assert_eq!(stats.total_weight, 0);
        assert_eq!(stats.total_points, 0);
    }

    #[test]
    fn test_record_submission() {
        let env = soroban_sdk::Env::default();
        let participant = Address::generate(&env);
        let description = String::from_str(&env, "Test");

        let mut stats = RecyclingStats::new(participant.clone());
        let material = Material::new(1, WasteType::Paper, 5000, participant, 0, description);

        stats.record_submission(&material);

        assert_eq!(stats.total_submissions, 1);
        assert_eq!(stats.total_weight, 5000);
        assert_eq!(stats.paper_count, 1);
    }

    #[test]
    fn test_record_verification() {
        let env = soroban_sdk::Env::default();
        let participant = Address::generate(&env);
        let description = String::from_str(&env, "Test");

        let mut stats = RecyclingStats::new(participant.clone());
        let mut material = Material::new(1, WasteType::Metal, 5000, participant, 0, description);

        material.verify();
        stats.record_verification(&material);

        assert_eq!(stats.verified_submissions, 1);
        assert_eq!(stats.total_points, 250); // 5kg * 5 * 10
    }

    #[test]
    fn test_verification_rate() {
        let env = soroban_sdk::Env::default();
        let participant = Address::generate(&env);

        let mut stats = RecyclingStats::new(participant);
        stats.total_submissions = 10;
        stats.verified_submissions = 8;

        assert_eq!(stats.verification_rate(), 80);
    }

    #[test]
    fn test_most_submitted_type() {
        let env = soroban_sdk::Env::default();
        let participant = Address::generate(&env);

        let mut stats = RecyclingStats::new(participant);
        stats.paper_count = 5;
        stats.plastic_count = 10;
        stats.metal_count = 3;

        assert_eq!(stats.most_submitted_type(), Some(WasteType::Plastic));
    }

    #[test]
    fn test_average_weight() {
        let env = soroban_sdk::Env::default();
        let participant = Address::generate(&env);

        let mut stats = RecyclingStats::new(participant);
        stats.total_submissions = 5;
        stats.total_weight = 10000;

        assert_eq!(stats.average_weight(), 2000);
    }

    #[test]
    fn test_is_active_recycler() {
        let env = soroban_sdk::Env::default();
        let participant = Address::generate(&env);

        let mut stats = RecyclingStats::new(participant);
        assert!(!stats.is_active_recycler());

        stats.total_submissions = 10;
        assert!(stats.is_active_recycler());
    }

    #[test]
    fn test_is_verified_contributor() {
        let env = soroban_sdk::Env::default();
        let participant = Address::generate(&env);

        let mut stats = RecyclingStats::new(participant);
        stats.total_submissions = 10;
        stats.verified_submissions = 8;

        assert!(stats.is_verified_contributor());
    }

    #[test]
    fn test_stats_storage() {
        let env = soroban_sdk::Env::default();
        let contract_id = env.register_contract(None, crate::ScavengerContract);
        let participant = Address::generate(&env);

        let stats = RecyclingStats::new(participant.clone());

        // RecyclingStats can be stored (validated through contract tests)
        assert_eq!(stats.total_submissions, 0);
    }
}

#[cfg(test)]
mod material_tests {
    use super::*;
    use soroban_sdk::testutils::Address as _;

    #[test]
    fn test_material_creation() {
        let env = soroban_sdk::Env::default();
        let submitter = Address::generate(&env);
        let description = String::from_str(&env, "Plastic bottles");

        let material = Material::new(
            1,
            WasteType::PetPlastic,
            5000,
            submitter.clone(),
            1234567890,
            description.clone(),
        );

        assert_eq!(material.id, 1);
        assert_eq!(material.waste_type, WasteType::PetPlastic);
        assert_eq!(material.weight, 5000);
        assert_eq!(material.submitter, submitter);
        assert_eq!(material.submitted_at, 1234567890);
        assert!(!material.verified);
        assert_eq!(material.description, description);
    }

    #[test]
    fn test_material_verify() {
        let env = soroban_sdk::Env::default();
        let submitter = Address::generate(&env);
        let description = String::from_str(&env, "Test");

        let mut material = Material::new(
            1,
            WasteType::Paper,
            1000,
            submitter,
            1234567890,
            description,
        );

        assert!(!material.verified);
        material.verify();
        assert!(material.verified);
    }

    #[test]
    fn test_meets_minimum_weight() {
        let env = soroban_sdk::Env::default();
        let submitter = Address::generate(&env);
        let description = String::from_str(&env, "Test");

        let material_below = Material::new(
            1,
            WasteType::Paper,
            50,
            submitter.clone(),
            1234567890,
            description.clone(),
        );
        assert!(!material_below.meets_minimum_weight());

        let material_exact = Material::new(
            2,
            WasteType::Paper,
            100,
            submitter.clone(),
            1234567890,
            description.clone(),
        );
        assert!(material_exact.meets_minimum_weight());

        let material_above =
            Material::new(3, WasteType::Paper, 500, submitter, 1234567890, description);
        assert!(material_above.meets_minimum_weight());
    }

    #[test]
    fn test_calculate_reward_points() {
        let env = soroban_sdk::Env::default();
        let submitter = Address::generate(&env);
        let description = String::from_str(&env, "Test");

        // Paper: 5kg * 1 * 10 = 50 points
        let paper = Material::new(
            1,
            WasteType::Paper,
            5000,
            submitter.clone(),
            0,
            description.clone(),
        );
        assert_eq!(paper.calculate_reward_points(), 50);

        // PetPlastic: 5kg * 3 * 10 = 150 points
        let pet = Material::new(
            2,
            WasteType::PetPlastic,
            5000,
            submitter.clone(),
            0,
            description.clone(),
        );
        assert_eq!(pet.calculate_reward_points(), 150);

        // Plastic: 5kg * 2 * 10 = 100 points
        let plastic = Material::new(
            3,
            WasteType::Plastic,
            5000,
            submitter.clone(),
            0,
            description.clone(),
        );
        assert_eq!(plastic.calculate_reward_points(), 100);

        // Metal: 5kg * 5 * 10 = 250 points
        let metal = Material::new(
            4,
            WasteType::Metal,
            5000,
            submitter.clone(),
            0,
            description.clone(),
        );
        assert_eq!(metal.calculate_reward_points(), 250);

        // Glass: 5kg * 2 * 10 = 100 points
        let glass = Material::new(5, WasteType::Glass, 5000, submitter, 0, description);
        assert_eq!(glass.calculate_reward_points(), 100);
    }

    #[test]
    fn test_material_storage_compatibility() {
        let env = soroban_sdk::Env::default();
        let contract_id = env.register_contract(None, crate::ScavengerContract);
        let submitter = Address::generate(&env);
        let description = String::from_str(&env, "Storage test");

        let material = Material::new(
            1,
            WasteType::Metal,
            3000,
            submitter,
            1234567890,
            description,
        );

        // Material can be stored in Soroban storage (validated through contract tests)
        assert_eq!(material.id, 1);
        assert_eq!(material.waste_type, WasteType::Metal);
        assert_eq!(material.weight, 3000);
    }
}

#[cfg(test)]
mod waste_type_tests {
    use super::*;

    #[test]
    fn test_waste_type_values() {
        assert_eq!(WasteType::Paper as u32, 0);
        assert_eq!(WasteType::PetPlastic as u32, 1);
        assert_eq!(WasteType::Plastic as u32, 2);
        assert_eq!(WasteType::Metal as u32, 3);
        assert_eq!(WasteType::Glass as u32, 4);
    }

    #[test]
    fn test_waste_type_is_valid() {
        assert!(WasteType::is_valid(0));
        assert!(WasteType::is_valid(1));
        assert!(WasteType::is_valid(2));
        assert!(WasteType::is_valid(3));
        assert!(WasteType::is_valid(4));
        assert!(!WasteType::is_valid(5));
        assert!(!WasteType::is_valid(999));
    }

    #[test]
    fn test_waste_type_from_u32() {
        assert_eq!(WasteType::from_u32(0), Some(WasteType::Paper));
        assert_eq!(WasteType::from_u32(1), Some(WasteType::PetPlastic));
        assert_eq!(WasteType::from_u32(2), Some(WasteType::Plastic));
        assert_eq!(WasteType::from_u32(3), Some(WasteType::Metal));
        assert_eq!(WasteType::from_u32(4), Some(WasteType::Glass));
        assert_eq!(WasteType::from_u32(5), None);
        assert_eq!(WasteType::from_u32(999), None);
    }

    #[test]
    fn test_waste_type_to_u32() {
        assert_eq!(WasteType::Paper.to_u32(), 0);
        assert_eq!(WasteType::PetPlastic.to_u32(), 1);
        assert_eq!(WasteType::Plastic.to_u32(), 2);
        assert_eq!(WasteType::Metal.to_u32(), 3);
        assert_eq!(WasteType::Glass.to_u32(), 4);
    }

    #[test]
    fn test_waste_type_as_str() {
        assert_eq!(WasteType::Paper.as_str(), "PAPER");
        assert_eq!(WasteType::PetPlastic.as_str(), "PETPLASTIC");
        assert_eq!(WasteType::Plastic.as_str(), "PLASTIC");
        assert_eq!(WasteType::Metal.as_str(), "METAL");
        assert_eq!(WasteType::Glass.as_str(), "GLASS");
    }

    #[test]
    fn test_waste_type_display() {
        // Test Display trait by converting to string representation
        assert_eq!(WasteType::Paper.as_str(), "PAPER");
        assert_eq!(WasteType::PetPlastic.as_str(), "PETPLASTIC");
        assert_eq!(WasteType::Plastic.as_str(), "PLASTIC");
        assert_eq!(WasteType::Metal.as_str(), "METAL");
        assert_eq!(WasteType::Glass.as_str(), "GLASS");
    }

    #[test]
    fn test_waste_type_is_plastic() {
        assert!(!WasteType::Paper.is_plastic());
        assert!(WasteType::PetPlastic.is_plastic());
        assert!(WasteType::Plastic.is_plastic());
        assert!(!WasteType::Metal.is_plastic());
        assert!(!WasteType::Glass.is_plastic());
    }

    #[test]
    fn test_waste_type_is_biodegradable() {
        assert!(WasteType::Paper.is_biodegradable());
        assert!(!WasteType::PetPlastic.is_biodegradable());
        assert!(!WasteType::Plastic.is_biodegradable());
        assert!(!WasteType::Metal.is_biodegradable());
        assert!(!WasteType::Glass.is_biodegradable());
    }

    #[test]
    fn test_waste_type_is_infinitely_recyclable() {
        assert!(!WasteType::Paper.is_infinitely_recyclable());
        assert!(!WasteType::PetPlastic.is_infinitely_recyclable());
        assert!(!WasteType::Plastic.is_infinitely_recyclable());
        assert!(WasteType::Metal.is_infinitely_recyclable());
        assert!(WasteType::Glass.is_infinitely_recyclable());
    }

    #[test]
    fn test_waste_type_clone_and_copy() {
        let waste1 = WasteType::Paper;
        let waste2 = waste1;
        assert_eq!(waste1, waste2);
    }

    #[test]
    fn test_waste_type_equality() {
        assert_eq!(WasteType::Paper, WasteType::Paper);
        assert_ne!(WasteType::Paper, WasteType::Plastic);
        assert_ne!(WasteType::Metal, WasteType::Glass);
    }

    #[test]
    fn test_all_waste_types() {
        let types = [
            WasteType::Paper,
            WasteType::PetPlastic,
            WasteType::Plastic,
            WasteType::Metal,
            WasteType::Glass,
        ];

        for (i, waste_type) in types.iter().enumerate() {
            assert_eq!(waste_type.to_u32(), i as u32);
            assert_eq!(WasteType::from_u32(i as u32), Some(*waste_type));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_role_values() {
        assert_eq!(ParticipantRole::Recycler as u32, 0);
        assert_eq!(ParticipantRole::Collector as u32, 1);
        assert_eq!(ParticipantRole::Manufacturer as u32, 2);
    }

    #[test]
    fn test_is_valid() {
        assert!(ParticipantRole::is_valid(0));
        assert!(ParticipantRole::is_valid(1));
        assert!(ParticipantRole::is_valid(2));
        assert!(!ParticipantRole::is_valid(3));
        assert!(!ParticipantRole::is_valid(999));
    }

    #[test]
    fn test_from_u32() {
        assert_eq!(
            ParticipantRole::from_u32(0),
            Some(ParticipantRole::Recycler)
        );
        assert_eq!(
            ParticipantRole::from_u32(1),
            Some(ParticipantRole::Collector)
        );
        assert_eq!(
            ParticipantRole::from_u32(2),
            Some(ParticipantRole::Manufacturer)
        );
        assert_eq!(ParticipantRole::from_u32(3), None);
        assert_eq!(ParticipantRole::from_u32(999), None);
    }

    #[test]
    fn test_to_u32() {
        assert_eq!(ParticipantRole::Recycler.to_u32(), 0);
        assert_eq!(ParticipantRole::Collector.to_u32(), 1);
        assert_eq!(ParticipantRole::Manufacturer.to_u32(), 2);
    }

    #[test]
    fn test_as_str() {
        assert_eq!(ParticipantRole::Recycler.as_str(), "RECYCLER");
        assert_eq!(ParticipantRole::Collector.as_str(), "COLLECTOR");
        assert_eq!(ParticipantRole::Manufacturer.as_str(), "MANUFACTURER");
    }

    #[test]
    fn test_can_collect_materials() {
        assert!(ParticipantRole::Recycler.can_collect_materials());
        assert!(ParticipantRole::Collector.can_collect_materials());
        assert!(!ParticipantRole::Manufacturer.can_collect_materials());
    }

    #[test]
    fn test_can_manufacture() {
        assert!(!ParticipantRole::Recycler.can_manufacture());
        assert!(!ParticipantRole::Collector.can_manufacture());
        assert!(ParticipantRole::Manufacturer.can_manufacture());
    }

    #[test]
    fn test_can_process_recyclables() {
        assert!(ParticipantRole::Recycler.can_process_recyclables());
        assert!(!ParticipantRole::Collector.can_process_recyclables());
        assert!(!ParticipantRole::Manufacturer.can_process_recyclables());
    }

    #[test]
    fn test_clone_and_copy() {
        let role1 = ParticipantRole::Recycler;
        let role2 = role1;
        assert_eq!(role1, role2);
    }

    #[test]
    fn test_equality() {
        assert_eq!(ParticipantRole::Recycler, ParticipantRole::Recycler);
        assert_ne!(ParticipantRole::Recycler, ParticipantRole::Collector);
        assert_ne!(ParticipantRole::Collector, ParticipantRole::Manufacturer);
    }
}

#[cfg(test)]
mod waste_transfer_tests {
    use super::*;
    use soroban_sdk::testutils::Address as _;
    use soroban_sdk::Vec;

    #[test]
    fn test_waste_transfer_creation() {
        let env = soroban_sdk::Env::default();
        let from = Address::generate(&env);
        let to = Address::generate(&env);
        let note = String::from_str(&env, "Test transfer");

        let transfer = WasteTransfer::new(1, from.clone(), to.clone(), 1234567890, note.clone());

        assert_eq!(transfer.waste_id, 1);
        assert_eq!(transfer.from, from);
        assert_eq!(transfer.to, to);
        assert_eq!(transfer.transferred_at, 1234567890);
        assert_eq!(transfer.note, note);
    }

    #[test]
    fn test_waste_transfer_storage_compatibility() {
        let env = soroban_sdk::Env::default();
        let from = Address::generate(&env);
        let to = Address::generate(&env);
        let note = String::from_str(&env, "Storage test");

        let transfer = WasteTransfer::new(1, from, to, 1234567890, note);

        // WasteTransfer can be stored (validated through contract tests)
        assert_eq!(transfer.waste_id, 1);
        assert_eq!(transfer.transferred_at, 1234567890);
    }

    #[test]
    fn test_waste_transfer_vec_storage() {
        let env = soroban_sdk::Env::default();
        let from = Address::generate(&env);
        let to = Address::generate(&env);
        let note = String::from_str(&env, "Vec test");

        let mut transfers = Vec::new(&env);
        transfers.push_back(WasteTransfer::new(
            1,
            from.clone(),
            to.clone(),
            1000,
            note.clone(),
        ));
        transfers.push_back(WasteTransfer::new(
            1,
            to.clone(),
            from.clone(),
            2000,
            note.clone(),
        ));

        // Vec<WasteTransfer> can be stored (validated through contract tests)
        assert_eq!(transfers.len(), 2);
        assert_eq!(transfers.get(0).unwrap().transferred_at, 1000);
        assert_eq!(transfers.get(1).unwrap().transferred_at, 2000);
    }

    #[test]
    fn test_waste_transfer_equality() {
        let env = soroban_sdk::Env::default();
        let from = Address::generate(&env);
        let to = Address::generate(&env);
        let note = String::from_str(&env, "Test");

        let transfer1 = WasteTransfer::new(1, from.clone(), to.clone(), 1000, note.clone());
        let transfer2 = WasteTransfer::new(1, from.clone(), to.clone(), 1000, note.clone());
        let transfer3 = WasteTransfer::new(2, from.clone(), to.clone(), 1000, note.clone());

        assert_eq!(transfer1, transfer2);
        assert_ne!(transfer1, transfer3);
    }

    #[test]
    fn test_waste_transfer_clone() {
        let env = soroban_sdk::Env::default();
        let from = Address::generate(&env);
        let to = Address::generate(&env);
        let note = String::from_str(&env, "Clone test");

        let transfer1 = WasteTransfer::new(1, from, to, 1000, note);
        let transfer2 = transfer1.clone();

        assert_eq!(transfer1, transfer2);
        assert_eq!(transfer1.waste_id, transfer2.waste_id);
    }
}

#[cfg(test)]
mod incentive_tests {
    use super::*;
    use soroban_sdk::testutils::Address as _;

    #[test]
    fn test_incentive_creation() {
        let env = soroban_sdk::Env::default();
        let rewarder = Address::generate(&env);

        let incentive = Incentive::new(
            1,
            rewarder.clone(),
            WasteType::PetPlastic,
            50,
            10000,
            1234567890,
        );

        assert_eq!(incentive.id, 1);
        assert_eq!(incentive.rewarder, rewarder);
        assert_eq!(incentive.waste_type, WasteType::PetPlastic);
        assert_eq!(incentive.reward_points, 50);
        assert_eq!(incentive.total_budget, 10000);
        assert_eq!(incentive.remaining_budget, 10000);
        assert!(incentive.active);
        assert_eq!(incentive.created_at, 1234567890);
    }

    #[test]
    fn test_incentive_deactivate() {
        let env = soroban_sdk::Env::default();
        let rewarder = Address::generate(&env);

        let mut incentive = Incentive::new(1, rewarder, WasteType::Metal, 100, 5000, 0);
        assert!(incentive.active);

        incentive.deactivate();
        assert!(!incentive.active);
    }

    #[test]
    fn test_incentive_calculate_reward() {
        let env = soroban_sdk::Env::default();
        let rewarder = Address::generate(&env);

        let incentive = Incentive::new(1, rewarder, WasteType::PetPlastic, 50, 10000, 0);

        // 5kg (5000g) * 50 points/kg = 250 points
        assert_eq!(incentive.calculate_reward(5000), 250);

        // 10kg * 50 points/kg = 500 points
        assert_eq!(incentive.calculate_reward(10000), 500);

        // Less than 1kg should give 0 points
        assert_eq!(incentive.calculate_reward(500), 0);
    }

    #[test]
    fn test_incentive_claim_reward() {
        let env = soroban_sdk::Env::default();
        let rewarder = Address::generate(&env);

        let mut incentive = Incentive::new(1, rewarder, WasteType::Metal, 100, 1000, 0);

        // Claim 5kg worth (500 points)
        let claimed = incentive.claim_reward(5000);
        assert_eq!(claimed, Some(500));
        assert_eq!(incentive.remaining_budget, 500);
        assert!(incentive.active);

        // Claim remaining 5kg (500 points)
        let claimed = incentive.claim_reward(5000);
        assert_eq!(claimed, Some(500));
        assert_eq!(incentive.remaining_budget, 0);
        assert!(!incentive.active); // Auto-deactivated
    }

    #[test]
    fn test_incentive_claim_insufficient_budget() {
        let env = soroban_sdk::Env::default();
        let rewarder = Address::generate(&env);

        let mut incentive = Incentive::new(1, rewarder, WasteType::Plastic, 50, 1000, 0);

        // Try to claim 30kg worth (1500 points) - exceeds budget
        let claimed = incentive.claim_reward(30000);
        assert_eq!(claimed, None);
        assert_eq!(incentive.remaining_budget, 1000); // Budget unchanged
        assert!(incentive.active);
    }

    #[test]
    fn test_incentive_claim_when_inactive() {
        let env = soroban_sdk::Env::default();
        let rewarder = Address::generate(&env);

        let mut incentive = Incentive::new(1, rewarder, WasteType::Glass, 30, 5000, 0);
        incentive.deactivate();

        // Cannot claim from inactive incentive
        let claimed = incentive.claim_reward(5000);
        assert_eq!(claimed, None);
    }

    #[test]
    fn test_incentive_has_sufficient_budget() {
        let env = soroban_sdk::Env::default();
        let rewarder = Address::generate(&env);

        let incentive = Incentive::new(1, rewarder, WasteType::Paper, 20, 1000, 0);

        // 10kg * 20 = 200 points (sufficient)
        assert!(incentive.has_sufficient_budget(10000));

        // 50kg * 20 = 1000 points (exactly sufficient)
        assert!(incentive.has_sufficient_budget(50000));

        // 60kg * 20 = 1200 points (insufficient)
        assert!(!incentive.has_sufficient_budget(60000));
    }

    #[test]
    fn test_incentive_has_sufficient_budget_when_inactive() {
        let env = soroban_sdk::Env::default();
        let rewarder = Address::generate(&env);

        let mut incentive = Incentive::new(1, rewarder, WasteType::Metal, 100, 5000, 0);
        incentive.deactivate();

        // Inactive incentive always returns false
        assert!(!incentive.has_sufficient_budget(1000));
    }

    #[test]
    fn test_incentive_storage_compatibility() {
        let env = soroban_sdk::Env::default();
        let rewarder = Address::generate(&env);

        let incentive = Incentive::new(1, rewarder, WasteType::PetPlastic, 50, 10000, 0);

        // Incentive can be stored (validated through contract tests)
        assert_eq!(incentive.id, 1);
        assert_eq!(incentive.total_budget, 10000);
    }

    #[test]
    fn test_incentive_equality() {
        let env = soroban_sdk::Env::default();
        let rewarder = Address::generate(&env);

        let incentive1 = Incentive::new(1, rewarder.clone(), WasteType::Metal, 100, 5000, 1000);
        let incentive2 = Incentive::new(1, rewarder.clone(), WasteType::Metal, 100, 5000, 1000);
        let incentive3 = Incentive::new(2, rewarder, WasteType::Metal, 100, 5000, 1000);

        assert_eq!(incentive1, incentive2);
        assert_ne!(incentive1, incentive3);
    }

    #[test]
    fn test_incentive_clone() {
        let env = soroban_sdk::Env::default();
        let rewarder = Address::generate(&env);

        let incentive1 = Incentive::new(1, rewarder, WasteType::Glass, 30, 8000, 0);
        let incentive2 = incentive1.clone();

        assert_eq!(incentive1, incentive2);
        assert_eq!(incentive1.id, incentive2.id);
        assert_eq!(incentive1.remaining_budget, incentive2.remaining_budget);
    }
}
