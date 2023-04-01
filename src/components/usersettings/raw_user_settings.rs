use hashbrown::HashMap;
use serde::{Deserialize, Serialize};

use super::UserSettings;

#[derive(Debug, Serialize, Deserialize)]
pub struct RawUserSettings {
    pub recipe_category_prefs: HashMap<String, String>,
    pub resource_category_prefs: HashMap<String, String>,
}

impl From<&UserSettings> for RawUserSettings {
    fn from(us: &UserSettings) -> Self {
        Self {
            recipe_category_prefs: us
                .recipe_category_prefs
                .iter()
                .map(|(cat, am)| (cat.clone(), am.name.clone()))
                .collect(),
            resource_category_prefs: us
                .resource_category_prefs
                .iter()
                .map(|(cat, md)| (cat.clone(), md.name.clone()))
                .collect(),
        }
    }
}
