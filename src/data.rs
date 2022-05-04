use std::collections::HashMap;
use image::Rgba;
use serde::{Deserialize, Serialize};
#[cfg(test)]
use serde_json::json;

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct GameData {
    pub items: HashMap<String, Item>,
    pub recipes: HashMap<String, Recipe>,
    pub assembling_machines: HashMap<String, AssemblingMachine>,
    pub item_groups: HashMap<String, ItemGroup>,
    pub item_subgroups: HashMap<String, ItemSubGroup>,
    pub mining_drills: HashMap<String, MiningDrill>,
    pub offshore_pumps: HashMap<String, OffshorePump>
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
    let _item1: Item = serde_json::from_value(json1).unwrap();

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
    let _item2: Item = serde_json::from_value(json2).unwrap();
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Icon {
    Simple(String),
    Icons(Vec<IconData>),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IconData {
    pub icon: String,
    #[serde(default = "default_tint")]
    pub tint: TintColor
}

fn default_tint() -> TintColor {
    TintColor{r: 0.0, g: 0.0, b: 0.0, a: 1.0}
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TintColor {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    #[serde(default = "default_alpha")]
    pub a: f32
}

impl From<TintColor> for Rgba<u8> {
    fn from(t: TintColor) -> Self {
        [
            (t.r * 255.0).round() as u8,
            (t.g * 255.0).round() as u8,
            (t.b * 255.0).round() as u8,
            (t.a * 255.0).round() as u8
        ].into()
    }
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
    #[serde(flatten)]
    pub result: RecipeResults
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RecipeResults {
    Single {
        result: String,
        #[serde(default = "default_result_count")]
        result_count: f32
    },
    Multiple{
        results: Vec<RecipeResult>
    }
}

const fn default_result_count() -> f32 {
    1.0
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecipeIngredient {
    pub ingredient_type: RecipeItemType,
    pub name: String,
    pub amount: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RecipeResult {
    SimpleItem {
        name: String,
        #[serde(default="default_result_count")]
        amount: f32
    },
    ComplexFixed{
        #[serde(rename = "type")]
        result_type: RecipeItemType,
        #[serde(flatten)]
        amount: RecipeAmount
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RecipeAmount {
    Number{
        amount: f32
    },
    MinMax {
        amount_min: f32,
        amount_max: f32,
    },
    Probability {
        amount: f32,
        probability: f32,
    },
    MinMaxProbability {
        amount_min: f32,
        amount_max: f32,
        probability: f32
    }
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
pub struct MiningDrill {
    #[serde(alias = "icons")]
    pub icon: Icon,
    pub name: String,
    pub mining_speed: f32,
    pub resource_categories: Vec<String>,
    #[serde(default = "Vec::new")]
    pub allowed_effects: Vec<EffectType>,
    pub module_specification: Option<ModuleSpec>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Resource {
    #[serde(alias = "icons")]
    pub icon: Icon,
    pub name: String,
    pub category: String,
    pub mining_time: f32,
    #[serde(flatten)]
    pub fluid_requirement: Option<FluidRequirement>,
    #[serde(flatten)]
    pub results: RecipeResults
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FluidRequirement {
    required_fluid: String,
    fluid_amount: f32
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OffshorePump {
    #[serde(alias = "icons")]
    icon: Icon,
    name: String,
    fluid: String,
    pumping_speed: f32
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
