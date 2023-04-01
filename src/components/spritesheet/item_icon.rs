use yew::prelude::*;

use super::SpriteSheetIcon;

#[derive(Debug, Clone)]
pub struct ItemIcon;

#[derive(Debug, Clone, PartialEq, Eq, Properties)]
pub struct ItemIconProperties {
    pub item: String,
}

impl Component for ItemIcon {
    type Message = ();
    type Properties = ItemIconProperties;

    fn create(_ctx: &Context<Self>) -> Self {
        Self
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();
        html! {
            <SpriteSheetIcon prefix={"item"} name={props.item.clone()} />
        }
    }
}
