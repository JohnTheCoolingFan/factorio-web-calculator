mod components;
mod constants;
mod data;

use components::*;
use once_cell::sync::Lazy;
use std::sync::RwLock;
use yew::prelude::*;
use yew_router::prelude::*;

pub static USER_SETTINGS: Lazy<RwLock<UserSettings>> = Lazy::new(|| {
    let result = UserSettings::create();
    RwLock::new(result)
});

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

#[function_component(MainApp)]
fn app() -> Html {
    html! {
        <BrowserRouter>
            <Switch<Route> render={switch} />
        </BrowserRouter>
    }
}

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::Renderer::<MainApp>::new().render();
}
