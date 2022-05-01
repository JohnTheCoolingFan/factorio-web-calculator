use std::collections::HashMap;
use serde::{Deserialize, Serialize};
#[cfg(test)]
use serde_json::json;

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct GameData {
    pub items: HashMap<String, Item>,
    pub recipes: HashMap<String, Recipe>,
    pub assembling_machines: HashMap<String, AssemblingMachine>,
    pub item_groups: HashMap<String, ItemGroup>,
    pub item_subgroups: HashMap<String, ItemSubGroup>
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Item {
    #[serde(alias = "icons")]
    pub icon: Icon,
    pub name: String,
    #[serde(default = "default_subgroup")]
    pub subgroup: String,
}

fn default_subgroup() -> String {
    "other".into()
}

// TODO: more test cases
// TODO: move somewhere else
#[test]
fn test_item_deserialization() {
    // This doesn't
    let json1 = json!({
        "icons": [
            {
                "icon": "pathhh",
                "tint": {
                    "r": 0.5,
                    "g": 0.75,
                    "b": 0.14,
                    "a": 0.75
                }
            }
        ],
        "name": "blahblah"
    });
    let item1: Item = serde_json::from_value(json1).unwrap();

    // This fails
    let json2 = json!({
        "icons": [
            {
                "icon": "__base__/graphics/icons/pipe.png",
                "tint": {
                    "b": 1,
                    "g": 0.5,
                    "r": 0.5
                }
            }
        ],
        "name": "infinity-pipe",
        "subgroup": "other",
        "type": "item"
    });
    let item2: Item = serde_json::from_value(json2).unwrap();
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Icon {
    Simple(String),
    Icons(Vec<IconData>),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProcessedIcon {
    /// X coordinate on spreadsheet
    pub x: usize,
    /// Y coordinate on spreadsheet
    pub y: usize,
    /// name of item this icon belongs to
    pub name: String
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IconData {
    pub icon: String,
    pub tint: Option<TintColor>
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TintColor {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    #[serde(default = "default_alpha")]
    pub a: f32
}

fn default_alpha() -> f32 {
    1.0
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Recipe {
    pub name: String,
    pub category: String,
    pub energy_required: f32,
    pub ingredients: Vec<RecipeIngredient>,
    pub results: Vec<RecipeResult>
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecipeIngredient {
    pub ingredient_type: RecipeItemType,
    pub name: String,
    pub amount: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecipeResult {
    pub name: String,
    pub amount: f32,
    pub result_type: RecipeItemType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecipeItemType {
    Item,
    Fluid
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AssemblingMachine {
    #[serde(alias = "icons")]
    pub icon: Icon,
    pub name: String,
    pub crafting_categories: Vec<String>,
    pub crafting_speed: f32,
    #[serde(default = "Vec::new")]
    pub allowed_effects: Vec<EffectType>,
    pub module_specification: Option<ModuleSpec>
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModuleSpec {
    pub module_slots: usize
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Module {
    pub icon: String,
    pub name: String,
    pub category: String,
    pub tier: u8,
    pub effect: HashMap<EffectType, Effect>,
    pub limitation: Option<Vec<String>>
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EffectType {
    Speed,
    Consumption,
    Pollution,
    Productivity
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Effect {
    pub bonus: f32
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ItemGroup {
    pub name: String
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ItemSubGroup {
    pub name: String,
    pub group: String
}
