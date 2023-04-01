mod raw_user_settings;
mod user_setting_recipe_category;
mod user_setting_resource_category;
mod user_settings_page;

pub use raw_user_settings::*;
pub use user_setting_recipe_category::*;
pub use user_setting_resource_category::*;
pub use user_settings_page::*;

use self::raw_user_settings::RawUserSettings;
use crate::{constants::GAME_DATA, data::*};
use gloo_storage::Storage;
use hashbrown::HashMap;

#[derive(Debug)]
pub struct UserSettings {
    recipe_category_prefs: HashMap<String, &'static AssemblingMachine>,
    resource_category_prefs: HashMap<String, &'static MiningDrill>,
}

impl UserSettings {
    pub fn assembling_machine(&self, category: &str) -> Option<&'static AssemblingMachine> {
        Some(*self.recipe_category_prefs.get(category)?)
    }

    pub fn mining_drill(&self, category: &str) -> Option<&'static MiningDrill> {
        Some(*self.resource_category_prefs.get(category)?)
    }

    pub fn change_recipe_category(&mut self, category: &str, machine: &'static AssemblingMachine) {
        log::info!(
            "Changed assembler for categpry {} to {}",
            category,
            machine.name
        );
        self.recipe_category_prefs
            .insert(category.to_string(), machine);
        self.write()
    }

    pub fn change_resource_category(&mut self, category: &str, machine: &'static MiningDrill) {
        log::info!(
            "Changed mining drill for category {} to {}",
            category,
            machine.name
        );
        self.resource_category_prefs
            .insert(category.to_string(), machine);
        self.write()
    }

    fn write(&self) {
        gloo_storage::LocalStorage::set("user_settings", RawUserSettings::from(self)).unwrap();
    }

    fn init() -> Self {
        log::info!("User settings init");
        let mut recipe_category_prefs = HashMap::new();
        for (recipe_category, assemblers) in GAME_DATA.recipe_categories_with_multiple_assemblers()
        {
            log::info!(
                "assembler for category {}: {}",
                recipe_category,
                assemblers[0].name
            );
            recipe_category_prefs.insert(recipe_category, assemblers[0]);
        }
        let mut resource_category_prefs = HashMap::new();
        for (resource_category, mining_drills) in
            GAME_DATA.resource_categories_with_multiple_mining_drills()
        {
            log::info!(
                "mining drill for category {}: {}",
                resource_category,
                mining_drills[0].name
            );
            resource_category_prefs.insert(resource_category, mining_drills[0]);
        }
        let result = Self {
            recipe_category_prefs,
            resource_category_prefs,
        };
        result.write();
        result
    }

    fn from_raw(raw_us: RawUserSettings) -> Self {
        log::info!("Loading user settings");
        Self {
            recipe_category_prefs: raw_us
                .recipe_category_prefs
                .into_iter()
                .map(|(cat, am)| (cat, GAME_DATA.assembling_machines.get(&am).unwrap()))
                .collect(),
            resource_category_prefs: raw_us
                .resource_category_prefs
                .into_iter()
                .map(|(cat, md)| (cat, GAME_DATA.mining_drills.get(&md).unwrap()))
                .collect(),
        }
    }

    pub fn create() -> Self {
        if let Ok(us) = gloo_storage::LocalStorage::get("user_settings") {
            Self::from_raw(us)
        } else {
            Self::init()
        }
    }
}
