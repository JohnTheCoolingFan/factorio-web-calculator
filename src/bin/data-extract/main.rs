use factorio_web_calculator::data::*;

use std::{collections::HashMap, iter::Iterator, path::PathBuf, fs::{self, File}};
use serde_json::{Value, from_reader, from_value, json};
use clap::Parser;

#[derive(Debug, Parser)]
#[clap(about, long_about = None)]
struct CliParameters {
    /// Path to Factorio installation dir
    #[clap(short, long, parse(from_os_str), value_name = "FACTORIO_DIR")]
    factorio_dir: PathBuf,
    /// Output directory
    #[clap(short, long, parse(from_os_str), value_name = "OUT")]
    output_dir: PathBuf,
    /// Input file to read data from
    #[clap(short, long, parse(from_os_str), value_name = "IN")]
    input_file: PathBuf,
    /// Use expensive recipes instead of normal
    #[clap(short, long)]
    expensive: bool
}

fn main() {
    let params = CliParameters::parse();

    let difficulty = params.expensive.then(|| "expensive").unwrap_or("normal");
    
    if !params.factorio_dir.exists() {
        panic!("{} does not exist", params.factorio_dir.to_str().unwrap());
    }

    let core_path = params.factorio_dir.join("data/core");
    let base_path = params.factorio_dir.join("data/base");
    let output_file = params.output_dir.join("parsed-data.json");
    let out_dir = params.output_dir;

    println!("Parsing input data");
    let in_file = File::open(params.input_file).unwrap();
    let json_data: Value = from_reader(in_file).unwrap();

    println!("Processing items");
    let items: HashMap<String, Item> = json_data["item"]
        .as_object()
        .cloned()
        .unwrap()
        .into_iter()
        .map(|(k, v)| (k, from_value(v).unwrap()))
        .collect();

    println!("Processing assembling machines");
    let assembling_machines: HashMap<String, AssemblingMachine> = json_data["assembling-machine"]
        .as_object()
        .cloned()
        .unwrap()
        .into_iter()
        .map(|(k, v)| (k, from_value(v).unwrap()))
        .collect();
    println!("Processing furnaces");
    let furnaces: HashMap<String, AssemblingMachine> = from_value(json_data["furnace"].clone()).unwrap();
    println!("Merging furnaces and assembling machines");
    let assembling_machines: HashMap<String, AssemblingMachine> = {
        assembling_machines.into_iter().chain(furnaces.into_iter()).collect()
    };

    println!("Processing item groups and subgroups");
    let item_groups: HashMap<String, ItemGroup> = from_value(json_data["item-group"].clone()).unwrap();
    let item_subgroups: HashMap<String, ItemSubGroup> = from_value(json_data["item-subgroup"].clone()).unwrap();

    println!("Processing recipes");
    let mut recipes: HashMap<String, Recipe> = HashMap::new();
    for (name, value) in json_data["recipe"].as_object().unwrap() {
        let category = value["category"].as_str().unwrap_or("crafting").to_string();
        let name = value["name"].as_str().unwrap_or(name).to_string();
        let energy_required = value["energy_required"].as_f64().unwrap_or(0.5) as f32;

        // Results
        let (results, ingredients) = if value[difficulty] != json!(null) {
            (get_results(&value[difficulty]), get_ingredients(&value[difficulty]))
        } else {
            (get_results(value), get_ingredients(value))
        };

        recipes.insert(name.clone(), Recipe{name, category, energy_required, results, ingredients});
    }

    println!("Done loading and parsing data")
}

fn get_results(value: &Value) -> Vec<RecipeResult> {
    // Results
    if let Some(result) = value["result"].as_str() {
        let result_count = value["result_count"].as_u64().unwrap_or(1) as f32;
        vec![RecipeResult{result_type: RecipeItemType::Item, name: result.to_string(), amount: result_count}]
    } else {
        value["results"].as_array().unwrap().iter().map(|v| {
            if let Some(item_product) = v.as_array() {
                let name = item_product[0].as_str().unwrap().to_string();
                let amount = item_product[1].as_u64().unwrap() as f32;
                RecipeResult{result_type: RecipeItemType::Item, name, amount}
            } else {
                let result_type = match v["type"].as_str().unwrap_or("item") {
                    "item" => RecipeItemType::Item,
                    "fluid" => RecipeItemType::Fluid,
                    _ => RecipeItemType::Item
                };
                let mut amount: f32 = if let Some(amount) = v["amount"].as_u64() {
                    amount as f32
                } else {
                    let amount_min = v["amount_min"].as_u64().unwrap() as f32;
                    let amount_max = v["amount_max"].as_u64().unwrap() as f32;
                    (amount_min + amount_max) / 2.0
                };
                if let Some(probability) = v["probability"].as_f64() {
                    amount *= probability as f32;
                }
                let name = v["name"].as_str().unwrap().to_string();
                RecipeResult{result_type, amount, name}
            }
        }).collect()
    }
}

fn get_ingredients(value: &Value) -> Vec<RecipeIngredient> {
    value["ingredients"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| {
            if let Some(ingredient) = v.as_array() {
                let name = ingredient[0].as_str().unwrap().to_string();
                let amount = ingredient[1].as_u64().unwrap() as f32;
                RecipeIngredient{ingredient_type: RecipeItemType::Item, name, amount}
            } else {
                let ingredient_type = match v["type"].as_str().unwrap_or("item"){
                    "fluid" => RecipeItemType::Fluid,
                    _ => RecipeItemType::Item
                };
                let name = v["name"].as_str().unwrap().to_string();
                let amount = v["amount"].as_f64().unwrap() as f32;
                RecipeIngredient{ingredient_type, name, amount}
            }
        }).collect()
}
