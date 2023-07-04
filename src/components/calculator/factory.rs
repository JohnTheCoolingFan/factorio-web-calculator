use std::hash::{Hash, Hasher};

use crate::{
    constants::{GAME_DATA, RECIPE_BLACKLIST},
    data::*,
    USER_SETTINGS,
};

use super::CalculationError;

#[derive(Debug, Clone)]
pub enum Factory<'a> {
    AssemblingMachine(&'a AssemblingMachine, &'a Recipe),
    MiningDrill(&'a MiningDrill, &'a Resource),
    OffshorePump(&'a OffshorePump),
}

impl PartialEq for Factory<'_> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::AssemblingMachine(am1, rec1), Self::AssemblingMachine(am2, rec2)) => {
                am1.name.eq(&am2.name) && rec1.name.eq(&rec2.name)
            }
            (Self::MiningDrill(md1, res1), Self::MiningDrill(md2, res2)) => {
                md1.name.eq(&md2.name) && res1.name.eq(&res2.name)
            }
            (Self::OffshorePump(op1), Self::OffshorePump(op2)) => op1.name.eq(&op2.name),
            _ => false,
        }
    }
}

impl Eq for Factory<'_> {}

impl Hash for Factory<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Self::AssemblingMachine(am, rec) => {
                state.write_u8(1);
                am.name.hash(state);
                rec.name.hash(state);
            }
            Self::MiningDrill(md, res) => {
                state.write_u8(2);
                md.name.hash(state);
                res.name.hash(state)
            }
            Self::OffshorePump(op) => {
                state.write_u8(3);
                op.name.hash(state)
            }
        }
    }
}

impl<'a> Factory<'a> {
    pub fn produced_per_sec(&self) -> Vec<(String, f64)> {
        match self {
            Factory::AssemblingMachine(am, re) => re
                .produces()
                .into_iter()
                .map(|(name, amount)| (name, (am.crafting_speed / re.energy_required()) * amount))
                .collect(),
            Factory::MiningDrill(md, re) => {
                let temp: Vec<(String, f64)> = (&re.results).into();
                temp.into_iter()
                    .map(|(name, amount)| (name, (md.mining_speed / re.mining_time) * amount))
                    .collect()
            }
            Factory::OffshorePump(op) => vec![(op.fluid.clone(), op.pumping_speed)],
        }
    }

    pub fn item_produced_per_sec(&self, item: &str) -> f64 {
        for product in self.produced_per_sec() {
            if product.0 == item {
                return product.1;
            }
        }
        0.0
    }

    pub fn item_produced_per_recipe(&self, item: &str) -> f64 {
        match self {
            Factory::AssemblingMachine(_, re) => {
                for product in &re.produces() {
                    if product.0 == item {
                        return product.1;
                    }
                }
                0.0
            }
            Factory::MiningDrill(_, re) => {
                let products: Vec<(String, f64)> = (&re.results).into();
                for product in &products {
                    if product.0 == item {
                        return product.1;
                    }
                }
                0.0
            }
            Factory::OffshorePump(_) => 1.0,
        }
    }

    pub fn consumed_per_sec(&self) -> Vec<(String, f64)> {
        match self {
            Factory::AssemblingMachine(a, r) => r
                .consumes()
                .into_iter()
                .map(|(name, amount)| (name, (a.crafting_speed / r.energy_required()) * amount))
                .collect(),
            Factory::MiningDrill(md, re) => {
                if let Some(fluid_requirement) = &re.fluid_requirement {
                    vec![(
                        fluid_requirement.required_fluid.clone(),
                        fluid_requirement.fluid_amount * (md.mining_speed / re.mining_time),
                    )]
                } else {
                    vec![]
                }
            }
            _ => vec![],
        }
    }

    pub fn icon_prefix(&self) -> &str {
        match self {
            Factory::AssemblingMachine(_, _) => "assembling-machine",
            Factory::MiningDrill(_, _) => "mining-drill",
            Factory::OffshorePump(_) => "offshore-pump",
        }
    }

    pub fn name(&self) -> String {
        match self {
            Factory::AssemblingMachine(am, _) => am.name.clone(),
            Factory::MiningDrill(md, _) => md.name.clone(),
            Factory::OffshorePump(op) => op.name.clone(),
        }
    }

    pub fn ips_for_item(item: &str) -> f64 {
        if let Ok(factory) = Self::for_item(item) {
            for (name, amount) in factory.produced_per_sec() {
                if name == item {
                    return amount;
                }
            }
        }
        1.0
    }

    pub fn crafting_speed(&self) -> f64 {
        match self {
            Factory::AssemblingMachine(am, _) => am.crafting_speed,
            Factory::MiningDrill(md, _) => md.mining_speed,
            Factory::OffshorePump(op) => op.pumping_speed,
        }
    }

    pub fn energy_required(&self) -> f64 {
        match self {
            Factory::AssemblingMachine(_, recipe) => recipe.energy_required(),
            Factory::MiningDrill(_, resource) => resource.mining_time,
            Factory::OffshorePump(_) => 1.0,
        }
    }

    pub fn for_item(item: &str) -> Result<Self, CalculationError> {
        if let Some(offshore_pump) = Self::find_offshore_pump_for_item(item) {
            Ok(Self::OffshorePump(offshore_pump))
        } else if let Some(resource) = Self::find_resource_for_item(item) {
            if let Some(mining_drill) = USER_SETTINGS
                .read()
                .ok()
                .and_then(|us| us.mining_drill(&resource.category))
                .or_else(|| Self::find_mining_drill_for_resource(&resource.category))
            {
                Ok(Self::MiningDrill(mining_drill, resource))
            } else {
                Err(CalculationError::MiningDrillNotFound(
                    resource.category.clone(),
                ))
            }
        } else if let Some(recipe) = Self::find_recipe_for_item(item) {
            if let Some(assembling_machine) = USER_SETTINGS
                .read()
                .ok()
                .and_then(|us| us.assembling_machine(&recipe.category))
                .or_else(|| Self::find_assembling_machine_for_recipe(&recipe.category))
            {
                Ok(Self::AssemblingMachine(assembling_machine, recipe))
            } else {
                Err(CalculationError::AssemblingMachineNotFound(
                    recipe.category.clone(),
                ))
            }
        } else {
            Err(CalculationError::RecipeOrResourceNotFound(item.into()))
        }
    }

    fn find_recipe_for_item(item: &str) -> Option<&'static Recipe> {
        for recipe in GAME_DATA.recipes.values() {
            if recipe.produces().iter().any(|(x, _)| x == item)
                && recipe.allow_decomposition()
                && !RECIPE_BLACKLIST.contains(&&*recipe.name)
            {
                log::info!("Found recipe {}", recipe.name);
                return Some(recipe);
            }
        }
        None
    }

    fn find_assembling_machine_for_recipe(
        recipe_category: &str,
    ) -> Option<&'static AssemblingMachine> {
        GAME_DATA
            .assembling_machines
            .values()
            .find(|&assembling_machine| {
                assembling_machine
                    .crafting_categories
                    .iter()
                    .any(|c| c == recipe_category)
            })
    }

    fn find_resource_for_item(item: &str) -> Option<&'static Resource> {
        for resource in GAME_DATA.resources.values() {
            let results: Vec<(String, f64)> = (&resource.results).into();
            if results.iter().any(|(x, _)| x == item) {
                return Some(resource);
            }
        }
        None
    }

    fn find_mining_drill_for_resource(resource_category: &str) -> Option<&'static MiningDrill> {
        GAME_DATA.mining_drills.values().find(|&mining_drill| {
            mining_drill
                .resource_categories
                .iter()
                .any(|c| c == resource_category)
        })
    }

    fn find_offshore_pump_for_item(item: &str) -> Option<&'static OffshorePump> {
        GAME_DATA
            .offshore_pumps
            .values()
            .find(|&offshore_pump| offshore_pump.fluid == item)
    }
}
