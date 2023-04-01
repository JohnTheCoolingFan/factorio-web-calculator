use crate::{constants::GAME_DATA, data::*, Route, USER_SETTINGS};
use yew::prelude::*;
use yew_router::prelude::*;

use super::{
    user_setting_recipe_category::UserSettingRecipeCategory,
    user_setting_resource_category::UserSettingResourceCategory,
};

#[derive(Debug)]
pub struct UserSettingsPage;

#[derive(Debug, Clone, PartialEq)]
pub enum UserSettingsPageMessage {
    ChangeAssembler(String, &'static AssemblingMachine),
    ChangeMiningDrill(String, &'static MiningDrill),
}

impl Component for UserSettingsPage {
    type Message = UserSettingsPageMessage;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        log::info!("update callback");
        let mut user_settings = USER_SETTINGS.write().unwrap();
        match msg {
            UserSettingsPageMessage::ChangeAssembler(recipe_category, assembling_machine) => {
                user_settings.change_recipe_category(&recipe_category, assembling_machine)
            }
            UserSettingsPageMessage::ChangeMiningDrill(resource_category, mining_drill) => {
                user_settings.change_resource_category(&resource_category, mining_drill)
            }
        };
        false
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <div id="usersettings">
                <p><Link<Route> to={Route::Home}>{"Go back"}</Link<Route>></p>
                <div id="usersettings_assemblingmachine">
                    <p>{"Assembling machines and furnaces:"}</p>
                    <ul>
                    {
                        for GAME_DATA.recipe_categories_with_multiple_assemblers().iter().map(|v| {
                            html_nested! {
                                <UserSettingRecipeCategory category={v.0.clone()} callback={ctx.link().callback(|m| m)} choices={v.1.clone()} />
                            }
                        })
                    }
                    </ul>
                </div>
                <div id="usersettings_miningdrill">
                    <p>{"Mining drills:"}</p>
                    <ul>
                    {
                        for GAME_DATA.resource_categories_with_multiple_mining_drills().iter().map(|v| {
                            html_nested! {
                                <UserSettingResourceCategory category={v.0.clone()} callback={ctx.link().callback(|m| m)} choices={v.1.clone()} />
                            }
                        })
                    }
                    </ul>
                </div>
            </div>
        }
    }
}
