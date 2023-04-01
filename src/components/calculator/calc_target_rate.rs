#[derive(Debug, Clone, PartialEq)]
pub enum CalcTargetRate {
    Factories(f64),
    ItemsPerSecond(f64),
}

impl CalcTargetRate {
    pub fn as_factories(&self, factory_ips: f64) -> f64 {
        match self {
            Self::Factories(f) => *f,
            Self::ItemsPerSecond(i) => i / factory_ips,
        }
    }

    pub fn as_ips(&self, factory_ips: f64) -> f64 {
        match self {
            Self::Factories(f) => f * factory_ips,
            Self::ItemsPerSecond(i) => *i,
        }
    }
}

impl Default for CalcTargetRate {
    fn default() -> Self {
        Self::Factories(1.0)
    }
}
