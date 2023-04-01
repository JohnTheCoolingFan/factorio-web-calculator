mod add_item;
mod item;
mod list_item;

pub use add_item::*;
pub use item::*;
pub use list_item::*;

use yew::{html::ChildrenRenderer, prelude::*};

#[derive(Debug)]
pub struct InputList;

#[derive(Debug, PartialEq, Properties)]
pub struct InputListProperties {
    #[prop_or_default]
    pub children: ChildrenRenderer<InputListItem>,
}

impl Component for InputList {
    type Message = ();
    type Properties = InputListProperties;

    fn create(_ctx: &Context<Self>) -> Self {
        Self
    }

    fn update(&mut self, _ctx: &Context<Self>, _msg: Self::Message) -> bool {
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <ul>
                { for ctx.props().children.iter() }
            </ul>
        }
    }
}
