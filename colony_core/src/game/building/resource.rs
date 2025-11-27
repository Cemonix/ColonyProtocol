use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ResourceType {
    Minerals,
    Gas,
    Energy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Resources {
    pub minerals: u32,
    pub gas: u32,
    pub energy: u32,
}

impl Resources {
    pub fn new(minerals: u32, gas: u32, energy: u32) -> Self {
        Self {
            minerals,
            gas,
            energy,
        }
    }

    pub fn zero() -> Self {
        Self::default()
    }

    pub fn can_afford(&self, cost: &Resources) -> bool {
        self.minerals >= cost.minerals && self.gas >= cost.gas && self.energy >= cost.energy
    }
    
    pub fn add(&self, other: &Resources) -> Self {
        Resources {
            minerals: self.minerals + other.minerals,
            gas: self.gas + other.gas,
            energy: self.energy + other.energy,
        }
    }

    pub fn subtract(&self, cost: &Resources) -> Option<Self> {
        if !self.can_afford(cost) {
            return None;
        }

        Some(Resources {
            minerals: self.minerals - cost.minerals,
            gas: self.gas - cost.gas,
            energy: self.energy - cost.energy,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resources_creation() {
        let resources = Resources::new(100, 50, 25);
        assert_eq!(resources.minerals, 100);
        assert_eq!(resources.gas, 50);
        assert_eq!(resources.energy, 25);
    }

    #[test]
    fn test_resources_zero() {
        let resources = Resources::zero();
        assert_eq!(resources.minerals, 0);
        assert_eq!(resources.gas, 0);
        assert_eq!(resources.energy, 0);
    }

    #[test]
    fn test_can_afford_sufficient_resources() {
        let resources = Resources::new(100, 50, 25);
        let cost = Resources::new(50, 25, 10);
        assert!(resources.can_afford(&cost));
    }

    #[test]
    fn test_can_afford_insufficient_resources() {
        let resources = Resources::new(100, 50, 25);
        let cost = Resources::new(150, 25, 10);
        assert!(!resources.can_afford(&cost));
    }

    #[test]
    fn test_subtract_success() {
        let resources = Resources::new(100, 50, 25);
        let cost = Resources::new(50, 25, 10);
        let result = resources.subtract(&cost).expect("Should have enough resources");

        assert_eq!(result.minerals, 50);
        assert_eq!(result.gas, 25);
        assert_eq!(result.energy, 15);
    }

    #[test]
    fn test_subtract_failure() {
        let resources = Resources::new(100, 50, 25);
        let cost = Resources::new(150, 25, 10);
        let result = resources.subtract(&cost);

        assert!(result.is_none());
    }

    #[test]
    fn test_add_resources() {
        let resources = Resources::new(100, 50, 25);
        let production = Resources::new(20, 10, 5);
        let result = resources.add(&production);

        assert_eq!(result.minerals, 120);
        assert_eq!(result.gas, 60);
        assert_eq!(result.energy, 30);
    }
}
