mod components;
mod constants;
mod data;
mod icon_map;

use components::*;
use factorio_web_calculator::data::GameData;
use gloo_net::http::Request;
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
}

#[derive(Debug)]
pub enum AppRootMessage {
    GameDataReady(Rc<GameData>),
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
}

impl Component for AppRoot {
    type Message = AppRootMessage;

    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let scope = ctx.link();
        scope.send_future(Self::fetch_game_data());

        Self { game_data: None }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        let game_data_context = self.game_data.clone();

        html! {
            <ContextProvider<Option<Rc<GameData>>> context = {game_data_context}>
                <BrowserRouter>
                    <Switch<Route> render={switch} />
                </BrowserRouter>
            </ContextProvider<Option<Rc<GameData>>>>
        }
    }
}

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::Renderer::<AppRoot>::new().render();
}
