mod components;
mod constants;
mod data;
mod icon_map;
mod prototype_ref;

use components::*;
use data::GameData;
use gloo_net::http::Request;
use icon_map::IconMap;
use std::{ops::Deref, rc::Rc, sync::RwLock};
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Debug)]
pub struct WrappedUserSettings {
    user_settings: RwLock<UserSettings>,
}

impl PartialEq for WrappedUserSettings {
    fn eq(&self, other: &Self) -> bool {
        let lock = self.user_settings.read().unwrap();
        let other_lock = other.user_settings.read().unwrap();

        lock.eq(&other_lock)
    }
}

impl Deref for WrappedUserSettings {
    type Target = RwLock<UserSettings>;

    fn deref(&self) -> &Self::Target {
        &self.user_settings
    }
}

#[derive(Debug, Clone, Routable, PartialEq)]
enum Route {
    #[at("/settings")]
    Settings,
    #[at("/")]
    Home,
}

fn switch(route: Route) -> Html {
    match route {
        Route::Home => html! { <Calculator /> },
        Route::Settings => html! { <UserSettingsPage /> },
    }
}

#[derive(Debug, Default)]
pub struct AppRoot {
    game_data: Option<Rc<GameData>>,
    icon_map: Option<Rc<IconMap>>,
    user_settings: Option<Rc<WrappedUserSettings>>,
}

#[derive(Debug)]
pub enum AppRootMessage {
    ResetData,
    GameDataReady(Rc<GameData>),
    IconMapReady(Rc<IconMap>),
    UserSettingsReady(Rc<WrappedUserSettings>),
}

impl AppRoot {
    async fn fetch_game_data() -> AppRootMessage {
        match Request::get("/factorio-web-calculator/assets/generated/processed-data.json")
            .send()
            .await
        {
            Err(req_err) => {
                log::error!("Failed to request game data: {}", req_err);
                panic!("Failed to request game data: {}", req_err);
            }
            Ok(req) => match req.json().await {
                Err(parse_err) => {
                    log::error!("Failed to parse game data: {}", parse_err);
                    panic!("Failed to parse game data: {}", parse_err);
                }
                Ok(game_data) => AppRootMessage::GameDataReady(Rc::new(game_data)),
            },
        }
    }

    async fn fetch_icon_map() -> AppRootMessage {
        match Request::get("/factorio-web-calculator/assets/generated/spritesheet-mapping.json")
            .send()
            .await
        {
            Err(req_err) => {
                log::error!("Failed to request spritesheet icon mapping: {}", req_err);
                panic!("Failed to request spritesheet icon mapping: {}", req_err);
            }
            Ok(response) => match response.json().await {
                Err(parse_err) => {
                    log::error!("Failed to parse spritesheet mapping: {}", parse_err);
                    panic!("Failed to parse spritesheet mapping: {}", parse_err);
                }
                Ok(icon_map) => AppRootMessage::IconMapReady(Rc::new(icon_map)),
            },
        }
    }

    async fn init_user_settings(game_data: Rc<GameData>) -> AppRootMessage {
        let user_settings = UserSettings::create(game_data.deref());
        let wrapped_us = WrappedUserSettings {
            user_settings: RwLock::new(user_settings),
        };
        AppRootMessage::UserSettingsReady(Rc::new(wrapped_us))
    }
}

impl Component for AppRoot {
    type Message = AppRootMessage;

    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let scope = ctx.link();
        scope.send_message(AppRootMessage::ResetData);

        Self::default()
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let scope = ctx.link();
        match msg {
            AppRootMessage::ResetData => {
                *self = Self::default();

                scope.send_future(Self::fetch_game_data());
                scope.send_future(Self::fetch_icon_map());
            }
            AppRootMessage::GameDataReady(game_data) => {
                self.game_data = Some(Rc::clone(&game_data));
                scope.send_future(Self::init_user_settings(game_data));
            }
            AppRootMessage::IconMapReady(icon_map) => self.icon_map = Some(icon_map),
            AppRootMessage::UserSettingsReady(user_settings) => {
                self.user_settings = Some(user_settings)
            }
        }
        true
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        let game_data_context = self.game_data.clone();
        let icon_map_context = self.icon_map.clone();
        let user_settings_context = self.user_settings.clone();

        html! {
            <ContextProvider<Option<Rc<GameData>>> context = {game_data_context}>
            <ContextProvider<Option<Rc<IconMap>>> context = {icon_map_context}>
            <ContextProvider<Option<Rc<WrappedUserSettings>>> context = {user_settings_context}>
                <BrowserRouter>
                    <Switch<Route> render={switch} />
                </BrowserRouter>
            </ContextProvider<Option<Rc<WrappedUserSettings>>>>
            </ContextProvider<Option<Rc<IconMap>>>>
            </ContextProvider<Option<Rc<GameData>>>>
        }
    }
}

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::Renderer::<AppRoot>::new().render();
}
