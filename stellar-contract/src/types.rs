use soroban_sdk::{contracttype, Env};

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
        matches!(role, 0 | 1 | 2)
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
        assert_eq!(ParticipantRole::from_u32(0), Some(ParticipantRole::Recycler));
        assert_eq!(ParticipantRole::from_u32(1), Some(ParticipantRole::Collector));
        assert_eq!(ParticipantRole::from_u32(2), Some(ParticipantRole::Manufacturer));
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
