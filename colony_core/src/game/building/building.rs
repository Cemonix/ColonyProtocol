use std::sync::Arc;

use super::building_config::BuildingDefinition;
use super::resource::Resources;

#[derive(Debug, Clone)]
pub struct Building {
    definition: Arc<BuildingDefinition>,
    current_level: u8,
    current_hp: u32,
    turns_until_complete: u8,
}

impl Building {
    pub fn new_unbuilt(definition: Arc<BuildingDefinition>) -> Self {
        Self {
            definition,
            current_level: 0,
            current_hp: 0,
            turns_until_complete: 0,
        }
    }

    pub fn new_at_level(definition: Arc<BuildingDefinition>, level: u8) -> Option<Self> {
        if level == 0 || level > definition.max_level {
            return None;
        }

        let max_hp = definition.hitpoints_for_level(level)?;

        Some(Self {
            definition,
            current_level: level,
            current_hp: max_hp,
            turns_until_complete: 0,
        })
    }

    pub fn start_build(&mut self) -> Option<Resources> {
        if self.current_level > 0 {
            return None;
        }

        let cost = self.definition.cost_for_level(1)?.clone();
        let build_time = self.definition.build_time_for_level(1)?;

        self.current_level = 1;
        self.current_hp = 0;
        self.turns_until_complete = build_time;

        Some(cost)
    }

    pub fn start_upgrade(&mut self) -> Option<Resources> {
        if self.current_level == 0 {
            return None;
        }

        if self.current_level >= self.definition.max_level {
            return None;
        }

        if !self.is_complete() {
            return None;
        }

        let next_level = self.current_level + 1;
        let cost = self.definition.cost_for_level(next_level)?.clone();
        let build_time = self.definition.build_time_for_level(next_level)?;

        self.current_level = next_level;
        self.current_hp = 0;
        self.turns_until_complete = build_time;

        Some(cost)
    }

    pub fn tick(&mut self) -> bool {
        if self.turns_until_complete > 0 {
            self.turns_until_complete -= 1;

            if self.turns_until_complete == 0 {
                if let Some(max_hp) = self.definition.hitpoints_for_level(self.current_level) {
                    self.current_hp = max_hp;
                    return true;
                }
            }
        }
        false
    }

    pub fn cancel_build(&mut self) -> bool {
        if self.turns_until_complete == 0 {
            return false;
        }

        if self.current_level == 1 {
            self.current_level = 0;
            self.current_hp = 0;
            self.turns_until_complete = 0;
        } else {
            self.current_level -= 1;
            if let Some(max_hp) = self.definition.hitpoints_for_level(self.current_level) {
                self.current_hp = max_hp;
            }
            self.turns_until_complete = 0;
        }

        true
    }

    pub fn take_damage(&mut self, damage: u32) -> bool {
        if self.current_hp > damage {
            self.current_hp -= damage;
            false
        } else {
            self.current_hp = 0;
            true
        }
    }

    pub fn repair(&mut self) {
        if let Some(max_hp) = self.definition.hitpoints_for_level(self.current_level) {
            self.current_hp = max_hp;
        }
    }

    pub fn is_complete(&self) -> bool {
        self.turns_until_complete == 0 && self.current_level > 0
    }

    pub fn is_building(&self) -> bool {
        self.turns_until_complete > 0
    }

    pub fn is_built(&self) -> bool {
        self.current_level > 0
    }

    pub fn is_destroyed(&self) -> bool {
        self.current_level > 0 && self.current_hp == 0 && !self.is_building()
    }

    pub fn level(&self) -> u8 {
        self.current_level
    }

    pub fn current_hp(&self) -> u32 {
        self.current_hp
    }

    pub fn max_hp(&self) -> Option<u32> {
        self.definition.hitpoints_for_level(self.current_level)
    }

    pub fn turns_until_complete(&self) -> u8 {
        self.turns_until_complete
    }

    pub fn id(&self) -> &str {
        &self.definition.id
    }

    pub fn name(&self) -> &str {
        &self.definition.name
    }

    pub fn description(&self) -> &str {
        &self.definition.description
    }

    pub fn max_level(&self) -> u8 {
        self.definition.max_level
    }

    pub fn production(&self) -> Option<Resources> {
        if !self.is_complete() {
            return None;
        }
        self.definition.production_for_level(self.current_level).copied()
    }

    pub fn storage_capacity(&self) -> Option<Resources> {
        if !self.is_complete() {
            return None;
        }
        self.definition.storage_capacity_for_level(self.current_level).copied()
    }

    pub fn energy_consumption(&self) -> Option<u32> {
        if !self.is_complete() {
            return None;
        }
        self.definition.energy_consumption_for_level(self.current_level)
    }

    pub fn upgrade_cost(&self) -> Option<Resources> {
        if self.current_level >= self.definition.max_level {
            return None;
        }
        self.definition.cost_for_level(self.current_level + 1).cloned()
    }

    pub fn definition(&self) -> &Arc<BuildingDefinition> {
        &self.definition
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::building::BuildingRegistry;

    fn get_test_definition() -> Arc<BuildingDefinition> {
        let json = r#"
        {
            "buildings": [
                {
                    "id": "test_building",
                    "name": "Test Building",
                    "description": "A test building",
                    "max_level": 3,
                    "costs": [
                        {"minerals": 100, "gas": 0, "energy": 0},
                        {"minerals": 200, "gas": 0, "energy": 0},
                        {"minerals": 300, "gas": 0, "energy": 0}
                    ],
                    "build_time": [2, 3, 4],
                    "energy_consumption": [10, 20, 30],
                    "hitpoints": [1000, 1500, 2000],
                    "production": {
                        "minerals": [10, 20, 30],
                        "gas": [5, 10, 15],
                        "energy": [10, 20, 30]
                    },
                    "storage_capacity": {
                        "minerals": [100, 200, 300],
                        "gas": [50, 100, 150],
                        "energy": [0, 0, 0]
                    },
                    "prerequisites": []
                }
            ]
        }
        "#;

        let registry = BuildingRegistry::load_from_string(json).unwrap();
        registry.get("test_building").unwrap()
    }

    #[test]
    fn test_new_unbuilt() {
        let def = get_test_definition();
        let building = Building::new_unbuilt(def);

        assert_eq!(building.level(), 0);
        assert_eq!(building.current_hp(), 0);
        assert!(!building.is_built());
        assert!(!building.is_complete());
    }

    #[test]
    fn test_new_at_level() {
        let def = get_test_definition();
        let building = Building::new_at_level(def, 2).unwrap();

        assert_eq!(building.level(), 2);
        assert_eq!(building.current_hp(), 1500);
        assert!(building.is_built());
        assert!(building.is_complete());
    }

    #[test]
    fn test_start_build() {
        let def = get_test_definition();
        let mut building = Building::new_unbuilt(def);

        let cost = building.start_build().expect("Should be able to start building");
        assert_eq!(cost.minerals, 100);
        assert_eq!(building.level(), 1);
        assert_eq!(building.turns_until_complete(), 2);
        assert!(!building.is_complete());
        assert!(building.is_building());
    }

    #[test]
    fn test_construction_tick() {
        let def = get_test_definition();
        let mut building = Building::new_unbuilt(def);

        building.start_build();
        assert_eq!(building.turns_until_complete(), 2);

        let completed = building.tick();
        assert!(!completed);
        assert_eq!(building.turns_until_complete(), 1);

        let completed = building.tick();
        assert!(completed);
        assert_eq!(building.turns_until_complete(), 0);
        assert_eq!(building.current_hp(), 1000);
        assert!(building.is_complete());
    }

    #[test]
    fn test_upgrade() {
        let def = get_test_definition();
        let mut building = Building::new_at_level(def, 1).unwrap();

        let cost = building.start_upgrade().expect("Should be able to upgrade");
        assert_eq!(cost.minerals, 200);
        assert_eq!(building.level(), 2);
        assert!(!building.is_complete());

        for _ in 0..3 {
            building.tick();
        }

        assert!(building.is_complete());
        assert_eq!(building.current_hp(), 1500);
    }

    #[test]
    fn test_cannot_upgrade_at_max_level() {
        let def = get_test_definition();
        let mut building = Building::new_at_level(def, 3).unwrap();

        let result = building.start_upgrade();
        assert!(result.is_none());
    }

    #[test]
    fn test_cancel_build() {
        let def = get_test_definition();
        let mut building = Building::new_unbuilt(def);

        building.start_build();
        assert!(building.is_building());

        let cancelled = building.cancel_build();
        assert!(cancelled);
        assert_eq!(building.level(), 0);
        assert!(!building.is_built());
    }

    #[test]
    fn test_cancel_upgrade() {
        let def = get_test_definition();
        let mut building = Building::new_at_level(def, 1).unwrap();

        building.start_upgrade();
        assert_eq!(building.level(), 2);

        let cancelled = building.cancel_build();
        assert!(cancelled);
        assert_eq!(building.level(), 1);
        assert_eq!(building.current_hp(), 1000);
        assert!(building.is_complete());
    }

    #[test]
    fn test_take_damage() {
        let def = get_test_definition();
        let mut building = Building::new_at_level(def, 1).unwrap();

        assert_eq!(building.current_hp(), 1000);

        let destroyed = building.take_damage(300);
        assert!(!destroyed);
        assert_eq!(building.current_hp(), 700);

        let destroyed = building.take_damage(800);
        assert!(destroyed);
        assert_eq!(building.current_hp(), 0);
        assert!(building.is_destroyed());
    }

    #[test]
    fn test_repair() {
        let def = get_test_definition();
        let mut building = Building::new_at_level(def, 1).unwrap();

        building.take_damage(500);
        assert_eq!(building.current_hp(), 500);

        building.repair();
        assert_eq!(building.current_hp(), 1000);
    }

    #[test]
    fn test_production_only_when_complete() {
        let def = get_test_definition();
        let mut building = Building::new_unbuilt(def);

        assert!(building.production().is_none());

        building.start_build();
        assert!(building.production().is_none());

        building.tick();
        building.tick();

        let production = building.production().expect("Should produce when complete");
        assert_eq!(production.minerals, 10);
    }

    #[test]
    fn test_energy_consumption_only_when_complete() {
        let def = get_test_definition();
        let mut building = Building::new_unbuilt(def);

        assert!(building.energy_consumption().is_none());

        building.start_build();
        building.tick();
        building.tick();

        let energy = building.energy_consumption().expect("Should consume energy when complete");
        assert_eq!(energy, 10);
    }
}
