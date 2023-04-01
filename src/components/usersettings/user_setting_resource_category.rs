use super::user_settings_page::UserSettingsPageMessage;
use crate::{constants::GAME_DATA, data::*, SpriteSheetIcon, USER_SETTINGS};
use wasm_bindgen::JsCast;
use web_sys::{EventTarget, HtmlInputElement};
use yew::prelude::*;

#[derive(Debug)]
pub struct UserSettingResourceCategory;

#[derive(Debug, Clone, PartialEq, Properties)]
pub struct UserSettingResourceCategoryProperties {
    pub category: String,
    pub choices: Vec<&'static MiningDrill>,
    pub callback: Callback<UserSettingsPageMessage>,
}

impl Component for UserSettingResourceCategory {
    type Properties = UserSettingResourceCategoryProperties;
    type Message = &'static MiningDrill;

    fn create(_ctx: &Context<Self>) -> Self {
        Self
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let props = ctx.props();
        props
            .callback
            .emit(UserSettingsPageMessage::ChangeMiningDrill(
                props.category.clone(),
                msg,
            ));
        false
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();
        let on_selected = ctx.link().batch_callback(|e: Event| {
            let target: Option<EventTarget> = e.target();
            let input = target.and_then(|t| t.dyn_into::<HtmlInputElement>().ok());
            input.and_then(|i| GAME_DATA.mining_drills.get(&i.value()))
        });
        html! {
            <li>
            <p> {props.category.clone()} </p>
            {
                for props.choices.iter().map(|md| {
                    html_nested! {
                        <label>
                            <input type="radio" name={format!("resource-category-pref-{}", props.category)} checked={
                                USER_SETTINGS
                                    .read().ok().and_then(|us| {
                                        us.resource_category_prefs
                                            .get(&props.category)
                                            .map(|mdp| mdp.name == md.name)
                                    })
                                .unwrap_or(false)} onchange={on_selected.clone()} value={md.name.clone()}/>
                            <SpriteSheetIcon name={md.name.clone()} prefix="mining-drill"/>
                        </label>
                    }
                })
            }
            </li>
        }
    }
}
