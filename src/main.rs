//mod data;

use std::collections::{HashMap, hash_map::Entry};
use wasm_bindgen::JsCast;
use yew::{events::Event, html::ChildrenRenderer};
use web_sys::{EventTarget, HtmlInputElement};
use yew::{virtual_dom::VChild, prelude::*};
use yew_router::prelude::*;
use serde::Deserialize;
use gloo_net::http::Request;
use web_sys::console;

const DEFAULT_ITEM: &str = "advanced-circuit";
const SPRITESHEET_SIZE: usize = 960;
const SPRITESHEET_DOWNSCALE: usize = 2;
const ICON_SIZE: usize = 64;

pub struct Calculator {
    pub targets: Vec<CalcTarget>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CalculatorMessage {
    RemoveItem(usize),
    AddItem(CalcTarget),
    ChangeItem(usize, String),
    ChangeRate(usize, CalcTargetRate)
}

#[derive(Debug, Clone, PartialEq, Properties)]
pub struct CalculatorProperties {
    icon_map: HashMap<String, (usize, usize)>
}

impl CalculatorProperties {
    fn make_item(&self, t: &CalcTarget) -> TargetItem {
        TargetItem{name: t.name.clone(), pos: *self.icon_map.get(&t.name).unwrap_or(&(0, 0))}
    }
}

impl Component for Calculator {
    type Message = CalculatorMessage;
    type Properties = CalculatorProperties;

    fn create(_ctx: &Context<Self>) -> Self {
        Self{targets: vec![CalcTarget::default()]}
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            CalculatorMessage::AddItem(target) => {
                self.targets.push(target);
                true
            },
            CalculatorMessage::RemoveItem(idx) => {
                self.targets.remove(idx);
                true
            },
            CalculatorMessage::ChangeItem(idx, name) => {
                let item = self.targets.get_mut(idx).unwrap();
                item.name = name;
                true
            }
            CalculatorMessage::ChangeRate(idx, rate) => {
                let item = self.targets.get_mut(idx).unwrap();
                item.rate = rate;
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let targets = &self.targets;
        let link = ctx.link();
        let props = ctx.props();
        html! {
            <div id="calc">
                <p> { "This is a calculator" } </p>
                <p> { "Current targets:" } </p>
                <InputList>
                { for targets.iter().enumerate().map(|(i, t)| 
                    html_nested! { <InputItem
                        item={props.make_item(t)}
                        factories={t.rate.as_factories(1.0)}
                        items_per_second={t.rate.as_ips(1.0)}
                        onchanged={link.callback(|m| m)}
                        index = {i} /> }
                ) }
                <AddItem onclick={link.callback(|m| m)}/>
                </InputList>
            </div>
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CalcTarget {
    pub name: String,
    pub rate: CalcTargetRate
}

impl Default for CalcTarget {
    fn default() -> Self {
        Self{name: DEFAULT_ITEM.into(), rate: CalcTargetRate::default()}
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum CalcTargetRate {
    Factories(f64),
    ItemsPerSecond(f64)
}

impl CalcTargetRate {
    pub fn as_factories(&self, factory_ips: f64) -> f64 {
        match self {
            Self::Factories(f) => *f,
            Self::ItemsPerSecond(i) => i / factory_ips
        }
    }

    pub fn as_ips(&self, factory_ips: f64) -> f64 {
        match self {
            Self::Factories(f) => f * factory_ips,
            Self::ItemsPerSecond(i) => *i
        }
    }
}

impl Default for CalcTargetRate {
    fn default() -> Self {
        Self::Factories(1.0)
    }
}

#[derive(Debug)]
struct InputList;

#[derive(Debug, PartialEq, Properties)]
struct InputListProperties {
    #[prop_or_default]
    children: ChildrenRenderer<InputListItem>,
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

#[derive(Debug, Clone, PartialEq)]
enum InputListItem {
    Input(VChild<InputItem>),
    Add(VChild<AddItem>)
}

impl From<VChild<InputItem>> for InputListItem {
    fn from(v: VChild<InputItem>) -> Self {
        Self::Input(v)
    }
}

impl From<VChild<AddItem>> for InputListItem {
    fn from(v: VChild<AddItem>) -> Self {
        Self::Add(v)
    }
}

impl From<InputListItem> for Html {
    fn from(val: InputListItem) -> Self {
        match val {
            InputListItem::Input(c) => c.into(),
            InputListItem::Add(c) => c.into()
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TargetItem {
    name: String,
    pos: (usize, usize)
}

#[derive(Debug, Clone)]
struct InputItem;

#[derive(Debug, Clone, PartialEq, Properties)]
struct InputItemProps {
    item: TargetItem,
    #[prop_or(1.0)]
    factories: f64,
    #[prop_or(1.0)]
    items_per_second: f64,
    onchanged: Callback<<Calculator as Component>::Message>,
    index: usize
}

#[derive(Debug, Clone)]
enum InputItemMessage {
    Remove,
    OpenItem,
    ItemSelected(String),
    Factories(f64),
    ItemsPerSecond(f64)
}

impl Component for InputItem {
    type Message = InputItemMessage;
    type Properties = InputItemProps;

    fn create(_ctx: &Context<Self>) -> Self {
        Self
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let props = ctx.props();
        let callback = &props.onchanged;
        match msg {
            InputItemMessage::ItemSelected(s) => {
                callback.emit(CalculatorMessage::ChangeItem(props.index, s));
            },
            InputItemMessage::Factories(a) => {
                callback.emit(CalculatorMessage::ChangeRate(props.index, CalcTargetRate::Factories(a)));
            },
            InputItemMessage::ItemsPerSecond(a) => {
                callback.emit(CalculatorMessage::ChangeRate(props.index, CalcTargetRate::ItemsPerSecond(a)));
            },
            InputItemMessage::Remove => {
                callback.emit(CalculatorMessage::RemoveItem(props.index))
            }
            _ => {}
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();
        let link = ctx.link();

        let on_factories_change = link.batch_callback(|e: Event| {
            let target: Option<EventTarget> = e.target();

            let input = target.and_then(|t| t.dyn_into::<HtmlInputElement>().ok());
            input.and_then(|i| Some(InputItemMessage::Factories(i.value().parse().ok()?)))
        });

        let on_ips_change = link.batch_callback(|e: Event| {
            let target: Option<EventTarget> = e.target();

            let input = target.and_then(|t| t.dyn_into::<HtmlInputElement>().ok());
            input.and_then(|i| Some(InputItemMessage::ItemsPerSecond(i.value().parse().ok()?)))
        });

        html! {
            <li class="target">
                // Remove this item from the list
                <button class="remove-item" onclick={link.callback(|_| InputItemMessage::Remove)}> {"x"} </button>
                // Change this item's target
                <button> <InputItemIcon item={props.item.clone()}/> </button>
                // Input factories
                {"Factories: "}
                <input type="text" onchange={on_factories_change} />
                {"items/s"}
                // Input Items Per Second
                <input type="text" onchange={on_ips_change} />
            </li>
        }
    }
}

#[derive(Debug, Clone)]
pub struct InputItemIcon;

#[derive(Debug, Clone, PartialEq, Properties)]
pub struct InputItemIconProperties {
    item: TargetItem
}

impl Component for InputItemIcon {
    type Message = ();
    type Properties = InputItemIconProperties;

    fn create(ctx: &Context<Self>) -> Self {
        Self
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();
        let pos = props.item.pos;
        html! {
            <img src="assets/generated/spritesheet.png" style={format!("object-fit: none; object-position: -{0}px -{1}px; width: {2}; height: {2}", pos.0, pos.1, ICON_SIZE)}/>
        }
    }
}

#[derive(Debug, Clone)]
pub struct AddItem;

#[derive(Debug, Clone, PartialEq, Properties)]
pub struct AddItemProperties {
    onclick: Callback<<Calculator as Component>::Message>
}

impl Component for AddItem {
    type Message = ();
    type Properties = AddItemProperties;

    fn create(_ctx: &Context<Self>) -> Self {
        Self
    }

    fn update(&mut self, ctx: &Context<Self>, _msg: Self::Message) -> bool {
        ctx.props().onclick.emit(CalculatorMessage::AddItem(CalcTarget::default()));
        false
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let link = ctx.link();
        html! {
            <li>
                <button onclick={link.callback(|_| ())}> {"+"} </button>
            </li>
        }
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let request = Request::new("assets/generated/spritesheet-mapping.json").send().await;

    match request {
        Ok(response) => {
            if response.ok() {
                console::log_1(&"Parsing mappings".into());
                let mappings: HashMap<String, (usize, usize)> = response.json().await.unwrap();
                yew::start_app_with_props::<Calculator>(CalculatorProperties{icon_map: mappings});
            } else {
                console::log_1(&format!("Failed to get mappings: http status code {}", response.status()).into())
            }
        },
        Err(why) => console::log_1(&format!("Failed to get mappings: {}", why).into())
    };
}
