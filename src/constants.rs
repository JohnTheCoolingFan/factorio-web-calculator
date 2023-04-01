use crate::data::GameData;
use hashbrown::HashMap;
use once_cell::sync::Lazy;

pub const DEFAULT_ITEM: &str = "electronic-circuit";
pub const UNKNOWN_ITEM: &str = "item-unknown";
pub const ORIGINAL_SPRITESHEET_SIZE: usize = 960;
pub const ORIGINAL_ICON_SIZE: usize = 64;
pub const DOWNSCALE: usize = 2;
pub const SPRITESHEET_SIZE: usize = ORIGINAL_SPRITESHEET_SIZE / DOWNSCALE;
pub const ICON_SIZE: usize = ORIGINAL_ICON_SIZE / DOWNSCALE;
pub const RECURSION_LIMIT: usize = 5000;
pub const VERY_SMALL: f64 = 1e-10;
pub const RECIPE_BLACKLIST: &[&str] = &[
    "coal-liquefaction",
    "kovarex-enrichment-process",
    "nuclear-fuel-reprocessing",
]; // allow_decomposition = false

pub static ICON_MAP: Lazy<HashMap<String, (usize, usize)>> = Lazy::new(|| {
    let json_mapping = include_bytes!("../assets/generated/spritesheet-mapping.json");
    serde_json::from_slice(json_mapping).unwrap()
});

pub static GAME_DATA: Lazy<GameData> = Lazy::new(|| {
    let game_data_json = include_bytes!("../assets/generated/processed-data.json");
    serde_json::from_slice(game_data_json).unwrap()
});
