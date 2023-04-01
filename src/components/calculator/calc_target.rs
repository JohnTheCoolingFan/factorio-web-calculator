use super::CalcTargetRate;
use crate::constants::DEFAULT_ITEM;

#[derive(Debug, Clone, PartialEq)]
pub struct CalcTarget {
    pub name: String,
    pub rate: CalcTargetRate,
}

impl Default for CalcTarget {
    fn default() -> Self {
        Self {
            name: DEFAULT_ITEM.into(),
            rate: CalcTargetRate::default(),
        }
    }
}
