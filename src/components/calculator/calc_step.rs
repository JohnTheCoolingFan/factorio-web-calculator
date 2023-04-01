use super::Factory;

#[derive(Debug, Clone, PartialEq)]
pub struct CalcStep {
    pub factory: Factory<'static>,
    pub amount: f64,
}

impl CalcStep {
    pub fn produced_per_sec(&self) -> Vec<(String, f64)> {
        self.factory
            .produced_per_sec()
            .into_iter()
            .map(|(name, amount)| (name, amount * self.amount))
            .collect()
    }

    pub fn consumed_per_sec(&self) -> Vec<(String, f64)> {
        self.factory
            .consumed_per_sec()
            .into_iter()
            .map(|(name, amount)| (name, amount * self.amount))
            .collect()
    }

    pub fn machine_name(&self) -> String {
        self.factory.name()
    }
}
