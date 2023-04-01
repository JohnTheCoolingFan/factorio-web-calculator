use super::{CalcStep, CalcTarget, Factory};
use crate::constants::{RECURSION_LIMIT, VERY_SMALL};
use hashbrown::HashMap;
use thiserror::Error;

#[derive(Debug, Clone, Default)]
pub struct Calculation {
    vector: HashMap<String, f64>,
    pub steps: Vec<CalcStep>,
}

impl Calculation {
    pub fn solve(mut self, input: &[CalcTarget]) -> Result<Self, CalculationError> {
        for target in input {
            let name = &target.name.clone();
            let factory = Factory::for_item(name)?;
            let items_per_second = target.rate.as_ips(factory.item_produced_per_sec(name));
            self.vector.insert(name.clone(), -items_per_second);
        }

        let mut recursion_limit = RECURSION_LIMIT;
        while !self.is_solved() && recursion_limit > 0 {
            recursion_limit -= 1;
            let item = self.pick_item().ok_or(CalculationError::NoItemToPick)?;
            let factory = Factory::for_item(&item.0)?;
            let mut amount = (item.1 * factory.energy_required()) / factory.crafting_speed();
            amount /= factory.item_produced_per_recipe(&item.0);
            log::info!(
                "Amount will be divided by {}",
                factory.item_produced_per_recipe(&item.0)
            );
            let step = CalcStep { factory, amount };
            self.apply_step(step);
        }

        if recursion_limit == 0 {
            return Err(CalculationError::RecursionLimit);
        }

        Ok(self)
    }

    pub fn apply_step(&mut self, step: CalcStep) {
        log::info!("Applying step in amount {:.3}", step.amount);
        let produced = step.produced_per_sec();
        let consumed = step.consumed_per_sec();

        for (name, amount) in &produced {
            log::info!("ingredient {} produced in amount of {:.3}", name, amount);
            let val = self.vector.entry(name.clone()).or_insert(0.0);
            *val += amount;
        }

        for (name, amount) in &consumed {
            log::info!("ingredient {} consumed in amount of {:.3}", name, amount);
            let val = self.vector.entry(name.clone()).or_insert(0.0);
            *val -= amount;
        }
        self.steps.push(step)
    }

    fn is_solved(&self) -> bool {
        //self.vector.values().sum::<f64>() == 0.0
        self.vector
            .values()
            .all(|i| (*i >= 0.0) || (i.abs() < VERY_SMALL))
    }

    fn pick_item(&self) -> Option<(String, f64)> {
        log::info!("Picking an item");
        for (name, value) in &self.vector {
            log::info!("Trying {}, {:.3}", name, value);
            if (value < &0.0) && (value.abs() > VERY_SMALL) {
                log::info!("Picked!");
                return Some((name.clone(), -value));
            }
        }
        None
    }
}

#[derive(Debug, Error)]
pub enum CalculationError {
    #[error("Recipe or Resource for item {0} not found")]
    RecipeOrResourceNotFound(String),
    #[error("Assembling machine for recipe {0} not found")]
    AssemblingMachineNotFound(String),
    #[error("Mining Drill for resource {0} not found")]
    MiningDrillNotFound(String),
    #[error("Recursion limit")]
    RecursionLimit,
    #[error("No item to pick")]
    NoItemToPick,
}
