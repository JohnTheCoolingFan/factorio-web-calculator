use factorio_web_calculator::data::{AssemblingMachine, GameData, MiningDrill};

#[derive(Debug, Clone)]
pub struct AssemblingMachineRef {
    name: String,
}

impl AssemblingMachineRef {
    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_full<'a>(&self, game_data: &'a GameData) -> Option<&'a AssemblingMachine> {
        game_data.assembling_machines.get(&self.name)
    }

    pub fn new(name: String) -> Self {
        Self { name }
    }
}

#[derive(Debug, Clone)]
pub struct MiningDrillRef {
    name: String,
}

impl MiningDrillRef {
    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_full<'a>(&self, game_data: &'a GameData) -> Option<&'a MiningDrill> {
        game_data.mining_drills.get(&self.name)
    }

    pub fn new(name: String) -> Self {
        Self { name }
    }
}
