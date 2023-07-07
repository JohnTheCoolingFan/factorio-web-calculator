mod components;
mod constants;
mod data;
mod icon_map;

use components::*;
use factorio_web_calculator::data::GameData;
use gloo_net::http::Request;
use icon_map::IconMap;
use once_cell::sync::Lazy;
use std::{rc::Rc, sync::RwLock};
use yew::prelude::*;
use yew_router::prelude::*;

pub static USER_SETTINGS: Lazy<RwLock<UserSettings>> = Lazy::new(|| {
    let result = UserSettings::create();
    RwLock::new(result)
});

// todo: move game data to AppRoot and do the same with user settings and icon mapping

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

#[derive(Debug)]
pub struct AppRoot {
    game_data: Option<Rc<GameData>>,
    icon_map: Option<Rc<IconMap>>,
}

#[derive(Debug)]
pub enum AppRootMessage {
    GameDataReady(Rc<GameData>),
    IconMapREady(Rc<IconMap>),
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
                Ok(icon_map) => AppRootMessage::IconMapREady(Rc::new(icon_map)),
            },
        }
    }
}

impl Component for AppRoot {
    type Message = AppRootMessage;

    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let scope = ctx.link();
        scope.send_future(Self::fetch_game_data());
        scope.send_future(Self::fetch_icon_map());

        Self {
            game_data: None,
            icon_map: None,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            AppRootMessage::GameDataReady(game_data) => self.game_data = Some(game_data),
            AppRootMessage::IconMapREady(icon_map) => self.icon_map = Some(icon_map),
        }
        true
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        let game_data_context = self.game_data.clone();
        let icon_map_context = self.icon_map.clone();

        html! {
            <ContextProvider<Option<Rc<GameData>>> context = {game_data_context}>
            <ContextProvider<Option<Rc<IconMap>>> context = {icon_map_context}>
                <BrowserRouter>
                    <Switch<Route> render={switch} />
                </BrowserRouter>
            </ContextProvider<Option<Rc<IconMap>>>>
            </ContextProvider<Option<Rc<GameData>>>>
        }
    }
}

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::Renderer::<AppRoot>::new().render();
}
