mod item_icon;

pub use item_icon::*;

use crate::constants::{
    DOWNSCALE, ICON_MAP, ICON_SIZE, ORIGINAL_ICON_SIZE, ORIGINAL_SPRITESHEET_SIZE,
    SPRITESHEET_SIZE, UNKNOWN_ITEM,
};
use yew::prelude::*;

#[derive(Debug)]
pub struct SpriteSheetIcon;

#[derive(Debug, Clone, PartialEq, Eq, Properties)]
pub struct SpriteSheetIconProperties {
    pub prefix: String,
    pub name: String,
}

impl Component for SpriteSheetIcon {
    type Message = ();
    type Properties = SpriteSheetIconProperties;

    fn create(_ctx: &Context<Self>) -> Self {
        Self
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();
        let pos = ICON_MAP
            .get(&format!("{}-{}", props.prefix, props.name))
            .or_else(|| ICON_MAP.get(&format!("item-{}", UNKNOWN_ITEM)))
            .unwrap_or(&(
                ORIGINAL_SPRITESHEET_SIZE - ORIGINAL_ICON_SIZE,
                ORIGINAL_SPRITESHEET_SIZE - ORIGINAL_ICON_SIZE,
            ));
        html! {
            <img src="assets/empty.gif" title={props.name.clone()} alt={props.name.clone()} style={ format!("background-image: url(\"assets/generated/spritesheet.png\"); background-position-x: -{0}px; background-position-y: -{1}px; width: {2}px; height: {2}px; background-size: {3}px", pos.0 / DOWNSCALE, pos.1 / DOWNSCALE, ICON_SIZE, SPRITESHEET_SIZE) }/>
        }
    }
}
