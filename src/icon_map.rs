use hashbrown::HashMap;
use serde::Deserialize;
use std::ops::Deref;

#[derive(Debug, Deserialize, PartialEq)]
#[serde(transparent)]
pub struct IconMap {
    icon_map: HashMap<String, (usize, usize)>,
}

impl Deref for IconMap {
    type Target = HashMap<String, (usize, usize)>;

    fn deref(&self) -> &Self::Target {
        &self.icon_map
    }
}
