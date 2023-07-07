mod raw_user_settings;
mod user_setting_recipe_category;
mod user_setting_resource_category;
mod user_settings_page;

pub use raw_user_settings::*;
pub use user_setting_recipe_category::*;
pub use user_setting_resource_category::*;
pub use user_settings_page::*;

use self::raw_user_settings::RawUserSettings;
use crate::{
    constants::GAME_DATA,
    data::*,
    prototype_ref::{AssemblingMachineRef, MiningDrillRef},
};
use gloo_storage::Storage;
use hashbrown::HashMap;

#[derive(Debug, PartialEq)]
pub struct UserSettings {
    recipe_category_prefs: HashMap<String, AssemblingMachineRef>,
    resource_category_prefs: HashMap<String, MiningDrillRef>,
}

impl UserSettings {
    pub fn assembling_machine(&self, category: &str) -> Option<&AssemblingMachineRef> {
        self.recipe_category_prefs.get(category)
    }

    pub fn mining_drill(&self, category: &str) -> Option<&MiningDrillRef> {
        self.resource_category_prefs.get(category)
    }

    pub fn change_recipe_category(&mut self, category: &str, machine: AssemblingMachineRef) {
        log::info!(
            "Changed assembler for category {} to {}",
            category,
            machine.get_name()
        );
        self.recipe_category_prefs
            .insert(category.to_string(), machine);
        self.write()
    }

    pub fn change_resource_category(&mut self, category: &str, machine: MiningDrillRef) {
        log::info!(
            "Changed mining drill for category {} to {}",
            category,
            machine.get_name()
        );
        self.resource_category_prefs
            .insert(category.to_string(), machine);
        self.write()
    }

    fn write(&self) {
        gloo_storage::LocalStorage::set("user_settings", RawUserSettings::from(self)).unwrap();
    }

    fn init(game_data: &GameData) -> Self {
        log::info!("User settings init");
        let mut recipe_category_prefs = HashMap::new();
        for (recipe_category, assemblers) in game_data.recipe_categories_with_multiple_assemblers()
        {
            log::info!(
                "assembler for category {}: {}",
                recipe_category,
                assemblers[0].name
            );
            recipe_category_prefs.insert(
                recipe_category,
                AssemblingMachineRef::new(assemblers[0].name.clone()),
            );
        }
        let mut resource_category_prefs = HashMap::new();
        for (resource_category, mining_drills) in
            game_data.resource_categories_with_multiple_mining_drills()
        {
            log::info!(
                "mining drill for category {}: {}",
                resource_category,
                mining_drills[0].name
            );
            resource_category_prefs.insert(
                resource_category,
                MiningDrillRef::new(mining_drills[0].name.clone()),
            );
        }
        let result = Self {
            recipe_category_prefs,
            resource_category_prefs,
        };
        result.write();
        result
    }

    fn from_raw(raw_us: RawUserSettings, game_data: &GameData) -> Self {
        log::info!("Loading user settings");
        Self {
            recipe_category_prefs: raw_us
                .recipe_category_prefs
                .into_iter()
                .map(|(cat, am)| {
                    let am_ref = if let Some(am_gd) = game_data.assembling_machines.get(&am) {
                        AssemblingMachineRef::new(am_gd.name.clone())
                    } else {
                        log::warn!("Assembling machine {} not found in game data", am);
                        AssemblingMachineRef::new(am)
                    };
                    (cat, am_ref)
                })
                .collect(),
            resource_category_prefs: raw_us
                .resource_category_prefs
                .into_iter()
                .map(|(cat, md)| {
                    let md_ref = if let Some(md_gd) = game_data.mining_drills.get(&md) {
                        MiningDrillRef::new(md_gd.name.clone())
                    } else {
                        log::warn!("Mining drill {} not found in game data", md);
                        MiningDrillRef::new(md)
                    };
                    (cat, md_ref)
                })
                .collect(),
        }
    }

    pub fn create(game_data: &GameData) -> Self {
        if let Ok(us) = gloo_storage::LocalStorage::get("user_settings") {
            Self::from_raw(us, game_data)
        } else {
            Self::init(game_data)
        }
    }
}
