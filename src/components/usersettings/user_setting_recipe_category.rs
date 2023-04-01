use super::user_settings_page::UserSettingsPageMessage;
use crate::{components::SpriteSheetIcon, constants::GAME_DATA, data::*, USER_SETTINGS};
use wasm_bindgen::JsCast;
use web_sys::{EventTarget, HtmlInputElement};
use yew::prelude::*;

#[derive(Debug)]
pub struct UserSettingRecipeCategory;

#[derive(Debug, Clone, PartialEq, Properties)]
pub struct UserSettingRecipeCategoryProperties {
    pub category: String,
    pub choices: Vec<&'static AssemblingMachine>,
    pub callback: Callback<UserSettingsPageMessage>,
}

impl Component for UserSettingRecipeCategory {
    type Properties = UserSettingRecipeCategoryProperties;
    type Message = &'static AssemblingMachine;

    fn create(_ctx: &Context<Self>) -> Self {
        Self
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let props = ctx.props();
        props
            .callback
            .emit(UserSettingsPageMessage::ChangeAssembler(
                props.category.clone(),
                msg,
            ));
        false
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();
        let on_selected = ctx.link().batch_callback(|e: Event| {
            log::info!("change");
            let target: Option<EventTarget> = e.target();
            let input = target.and_then(|t| t.dyn_into::<HtmlInputElement>().ok());
            input.and_then(|i| GAME_DATA.assembling_machines.get(&i.value()))
        });
        html! {
            <li>
            <p> {props.category.clone()} </p>
            {
                for props.choices.iter().map(|am| {
                    html_nested! {
                        <label>
                            <input type="radio" name={format!("recipe-category-pref-{}", props.category)} checked={
                                USER_SETTINGS
                                    .read().ok().and_then(|us| {
                                        us.recipe_category_prefs
                                            .get(&props.category)
                                            .map(|amp| amp.name == am.name)
                                    })
                                .unwrap_or(false)} onchange={on_selected.clone()} value={am.name.clone()}/>
                            <SpriteSheetIcon name={am.name.clone()} prefix="assembling-machine"/>
                        </label>
                    }
                })
            }
            </li>
        }
    }
}
