use yew::{prelude::*, virtual_dom::VChild};

use super::{AddItem, InputItem};

#[derive(Debug, Clone, PartialEq)]
pub enum InputListItem {
    Input(VChild<InputItem>),
    Add(VChild<AddItem>),
}

impl From<VChild<InputItem>> for InputListItem {
    fn from(v: VChild<InputItem>) -> Self {
        Self::Input(v)
    }
}

impl From<VChild<AddItem>> for InputListItem {
    fn from(v: VChild<AddItem>) -> Self {
        Self::Add(v)
    }
}

impl From<InputListItem> for Html {
    fn from(val: InputListItem) -> Self {
        match val {
            InputListItem::Input(c) => c.into(),
            InputListItem::Add(c) => c.into(),
        }
    }
}
