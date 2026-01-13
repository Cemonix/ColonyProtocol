use core::fmt;
use std::ops::{Add, AddAssign, Sub, SubAssign};

#[derive(serde::Deserialize)]
pub enum ResourceType {
    Minerals(u32),
    Gas(u32),
    Energy(u32)
}

#[derive(serde::Deserialize, Default, Clone, Debug, PartialEq, PartialOrd)]
pub struct Resources {
    pub minerals: u32,
    pub gas: u32,
    pub energy: u32
}

impl Resources {
    pub fn has_enough(&self, cost: &Resources) -> bool {
        self.minerals >= cost.minerals 
        && self.gas >= cost.gas 
        && self.energy >= cost.energy
    }
}

impl Add for Resources {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            minerals: self.minerals.saturating_add(other.minerals),
            gas: self.gas.saturating_add(other.gas),
            energy: self.energy.saturating_add(other.energy),
        }
    }
}

impl AddAssign for Resources {
    fn add_assign(&mut self, other: Self) {
        self.minerals = self.minerals.saturating_add(other.minerals);
        self.gas = self.gas.saturating_add(other.gas);
        self.energy = self.energy.saturating_add(other.energy);
    }
}

impl AddAssign<&Resources> for Resources {
    fn add_assign(&mut self, other: &Resources) {
        self.minerals = self.minerals.saturating_add(other.minerals);
        self.gas = self.gas.saturating_add(other.gas);
        self.energy = self.energy.saturating_add(other.energy);
    }
}

impl Sub for Resources {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self {
            minerals: self.minerals.saturating_sub(other.minerals),
            gas: self.gas.saturating_sub(other.gas),
            energy: self.energy.saturating_sub(other.energy),
        }
    }
}

impl SubAssign for Resources {
    fn sub_assign(&mut self, other: Self) {
        self.minerals = self.minerals.saturating_sub(other.minerals);
        self.gas = self.gas.saturating_sub(other.gas);
        self.energy = self.energy.saturating_sub(other.energy);
    }
}

impl SubAssign<&Resources> for Resources {
    fn sub_assign(&mut self, other: &Resources) {
        self.minerals = self.minerals.saturating_sub(other.minerals);
        self.gas = self.gas.saturating_sub(other.gas);
        self.energy = self.energy.saturating_sub(other.energy);
    }
}

impl fmt::Display for Resources {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f, "Resources {{ minerals: {}, gas: {}, energy: {} }}", self.minerals, self.gas, self.energy
        )
    }
}