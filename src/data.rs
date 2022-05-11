use std::collections::{HashMap, HashSet};
use image::Rgba;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct GameData {
    pub items: HashMap<String, Item>,
    pub recipes: HashMap<String, Recipe>,
    pub assembling_machines: HashMap<String, AssemblingMachine>,
    pub item_groups: HashMap<String, ItemGroup>,
    pub item_subgroups: HashMap<String, ItemSubGroup>,
    pub mining_drills: HashMap<String, MiningDrill>,
    pub offshore_pumps: HashMap<String, OffshorePump>,
    pub resources: HashMap<String, Resource>
}

impl GameData {
    pub fn recipe_categories_with_multiple_assemblers(&self) -> HashMap<String, Vec<&AssemblingMachine>> {
        let mut categories: HashSet<String> = HashSet::new();
        let mut result = HashMap::new();
        for recipe in self.recipes.values() {
            categories.insert(recipe.category.clone());
        }
        for category in categories {
            let entry = result.entry(category.clone()).or_insert_with(Vec::new);
            for assembling_machine in self.assembling_machines.values() {
                if assembling_machine.crafting_categories.len() > 1 && assembling_machine.crafting_categories.contains(&category) {
                    entry.push(assembling_machine);
                }
            }
        }
        result.retain(|_, v| v.len() > 1);
        result
            .iter_mut()
            .map(|(_, am_vec)| am_vec
                .sort_by_key(|am| &am.name)).for_each(drop);
        result
    }

    pub fn resource_categories_with_multiple_mining_drills(&self) -> HashMap<String, Vec<&MiningDrill>> {
        let mut categories: HashSet<String> = HashSet::new();
        let mut result = HashMap::new();
        for resource in self.resources.values() {
            categories.insert(resource.category.clone());
        }
        for category in categories {
            let entry = result.entry(category.clone()).or_insert_with(Vec::new);
            for mining_drill in self.mining_drills.values() {
                if mining_drill.resource_categories.len() > 1 && mining_drill.resource_categories.contains(&category) {
                    entry.push(mining_drill);
                }
            }
        }
        result.retain(|_, v| v.len() > 1);
        result
            .iter_mut()
            .map(|(_, md_vec)| md_vec
                .sort_by_key(|md| &md.name)).for_each(drop);
        result
    }
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
    TintColor{r: 1.0, g: 1.0, b: 1.0, a: 1.0}
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TintColor {
    pub r: f64,
    pub g: f64,
    pub b: f64,
    #[serde(default = "default_alpha")]
    pub a: f64
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

fn default_alpha() -> f64 {
    1.0
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Recipe {
    pub name: String,
    #[serde(default = "default_recipe_category")]
    pub category: String,
    #[serde(flatten)]
    pub recipe_data: RecipeBody
}

impl Recipe {
    fn get_recipe_data(&self) -> &RecipeData {
        match &self.recipe_data {
            RecipeBody::Simple { data } => data,
            RecipeBody::NormalAndExpensive { normal, expensive: _ } => normal
        }
    }

    pub fn produces(&self) -> Vec<(String, f64)> {
        (&self.get_recipe_data().results).into()
    }

    pub fn consumes(&self) -> Vec<(String, f64)> {
        self.get_recipe_data().ingredients.iter().map(Into::into).collect()
    }

    pub fn energy_required(&self) -> f64 {
        self.get_recipe_data().energy_required
    }

    pub fn allow_decomposition(&self) -> bool {
        self.get_recipe_data().allow_decomposition
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RecipeBody {
    Simple{
        #[serde(flatten)]
        data: RecipeData
    },
    NormalAndExpensive {
        normal: RecipeData,
        expensive: RecipeData
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecipeData {
    #[serde(default = "default_energy_required")]
    pub energy_required: f64,
    pub ingredients: Vec<RecipeIngredient>,
    #[serde(flatten)]
    pub results: RecipeResults,
    #[serde(default = "default_allow_decomposition")]
    pub allow_decomposition: bool
}

fn default_recipe_category() -> String { "crafting".into() }
const fn default_energy_required() -> f64 { 0.5 }
const fn default_allow_decomposition() -> bool { true }

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RecipeResults {
    Single {
        result: String,
        #[serde(default = "default_result_count")]
        result_count: f64
    },
    Multiple{
        results: Vec<RecipeResult>
    }
}

impl From<&RecipeResults> for Vec<(String, f64)> {
    fn from(results: &RecipeResults) -> Self {
        match results {
            RecipeResults::Single { result, result_count } => vec![(result.clone(), *result_count)],
            RecipeResults::Multiple { results } => results.iter().map(|rr| rr.into()).collect()
        }
    }
}

const fn default_result_count() -> f64 { 1.0 }

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RecipeIngredient {
    Struct(RecipeIngredientStruct),
    Tuple(RecipeIngredientTuple)
}

impl From<&RecipeIngredient> for (String, f64) {
    fn from(ri: &RecipeIngredient) -> Self {
        match &ri {
            RecipeIngredient::Struct(sri) => (sri.name.clone(), (&sri.amount).into()),
            RecipeIngredient::Tuple(tri) => (tri.0.clone(), tri.1)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecipeIngredientStruct {
    name: String,
    #[serde(flatten)]
    amount: RecipeAmount,
    #[serde(rename = "type")]
    #[serde(default = "default_recipe_ingredient_type")]
    ingr_type: RecipeItemType,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecipeIngredientTuple(String, f64);

const fn default_recipe_ingredient_type() -> RecipeItemType { RecipeItemType::Item }

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RecipeResult {
    Tuple(RecipeResultTuple),
    Struct(RecipeResultStruct)
}

impl From<&RecipeResult> for (String, f64) {
    fn from(rr: &RecipeResult) -> Self {
        match rr {
            RecipeResult::Tuple(trr) => (trr.0.clone(), trr.1),
            RecipeResult::Struct(srr) => (srr.name.clone(), (&srr.amount).into())
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecipeResultTuple(String, f64);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecipeResultStruct {
    name: String,
    #[serde(rename = "type")]
    #[serde(default = "default_recipe_ingredient_type")]
    result_type: RecipeItemType,
    #[serde(flatten)]
    amount: RecipeAmount,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RecipeAmount {
    NamedNumber{
        amount: f64
    },
    MinMax {
        amount_min: f64,
        amount_max: f64,
    },
    Probability {
        amount: f64,
        probability: f64,
    },
    MinMaxProbability {
        amount_min: f64,
        amount_max: f64,
        probability: f64
    }
}

impl From<&RecipeAmount> for f64 {
    fn from(ra: &RecipeAmount) -> f64 {
        match &ra {
            RecipeAmount::NamedNumber{amount} => *amount,
            RecipeAmount::MinMax{amount_min, amount_max} => (amount_min + amount_max) / 2.0,
            RecipeAmount::Probability{amount, probability} => amount * probability,
            RecipeAmount::MinMaxProbability{amount_min, amount_max, probability} => (amount_min + amount_max) / 2.0 * probability
        }
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
    pub crafting_speed: f64,
    #[serde(default = "Vec::new")]
    pub allowed_effects: Vec<EffectType>,
    pub module_specification: Option<ModuleSpec>
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MiningDrill {
    #[serde(alias = "icons")]
    pub icon: Icon,
    pub name: String,
    pub mining_speed: f64,
    pub resource_categories: Vec<String>,
    //pub allowed_effects: Option<EffectType>, // Exported data is broken a bit
    pub module_specification: Option<ModuleSpec>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Resource {
    #[serde(alias = "icons")]
    pub icon: Icon,
    pub name: String,
    pub category: String,
    pub mining_time: f64,
    #[serde(flatten)]
    pub fluid_requirement: Option<FluidRequirement>,
    #[serde(flatten)]
    pub results: RecipeResults
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FluidRequirement {
    pub required_fluid: String,
    pub fluid_amount: f64
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OffshorePump {
    #[serde(alias = "icons")]
    pub icon: Icon,
    pub name: String,
    pub fluid: String,
    pub pumping_speed: f64
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
    pub bonus: f64
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
