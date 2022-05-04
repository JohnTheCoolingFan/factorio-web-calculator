mod data;

use data::*;
use thiserror::Error;
use std::collections::HashMap;
use wasm_bindgen::JsCast;
use yew::{events::Event, html::ChildrenRenderer};
use web_sys::{EventTarget, HtmlInputElement};
use yew::{virtual_dom::VChild, prelude::*};
use yew_router::prelude::*;
use once_cell::sync::Lazy;

const DEFAULT_ITEM: &str = "advanced-circuit";
const UNKNOWN_ITEM: &str = "item-unknown";
const ORIGINAL_SPRITESHEET_SIZE: usize = 960;
const ORIGINAL_ICON_SIZE: usize = 64;
const DOWNSCALE: usize = 2;
const SPRITESHEET_SIZE: usize = ORIGINAL_SPRITESHEET_SIZE / DOWNSCALE;
const ICON_SIZE: usize = ORIGINAL_ICON_SIZE / DOWNSCALE;

static ICON_MAP: Lazy<HashMap<String, (usize, usize)>> = Lazy::new(|| {
    let json_mapping = include_bytes!("../assets/generated/spritesheet-mapping.json");
    serde_json::from_slice(json_mapping).unwrap()
});

static GAME_DATA: Lazy<GameData> = Lazy::new(|| {
    let game_data_json = include_bytes!("../assets/generated/processed-data.json");
    serde_json::from_slice(game_data_json).unwrap()
});

#[derive(Debug)]
pub struct Calculator {
    pub targets: Vec<CalcTarget>,
    pub calculation: Result<Calculation, CalculationError>
}

#[derive(Debug, Clone, PartialEq)]
pub enum CalculatorMessage {
    RemoveItem(usize),
    AddItem(CalcTarget),
    ChangeItem(usize, String),
    ChangeRate(usize, CalcTargetRate)
}

impl Component for Calculator {
    type Message = CalculatorMessage;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            targets: vec![CalcTarget::default()],
            calculation: Ok(Calculation::default())
        }
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
        html! {
            <div id="calc">
                <p> { "This is a calculator" } </p>
                <p> { "Current targets:" } </p>
                <InputList>
                { for targets.iter().enumerate().map(|(i, t)| 
                    html_nested! { <InputItem
                        item={t.name.clone()}
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

#[derive(Debug, Error)]
pub enum CalculationError {
    #[error("Recipe or Resource for item {0} npt found")]
    RecipeOrResourceNotFound(String),
    #[error("Assembling machine for recipe {0} not found")]
    AssemblingMachineNotFound(String),
    #[error("Minming Drill for resource {0} not found")]
    MiningDrillNotFound(String)
}

#[derive(Debug, Clone, Default)]
pub struct Calculation {
    vector: HashMap<String, f32>,
    pub steps: Vec<CalcStep>
}

#[derive(Debug, Clone)]
pub struct CalcStep {
    factory: Factory<'static>,
    amount: f32
}

impl CalcStep {
    fn produced_per_sec(&self) -> Vec<(String, f32)> {
        self.factory.produced_per_sec().into_iter().map(|(name, amount)| (name, amount * self.amount)).collect()
    }

    fn consumed_per_sec(&self) -> Vec<(String, f32)> {
        self.factory.consumed_per_sec().into_iter().map(|(name, amount)| (name, amount * self.amount)).collect()
    }
}

#[derive(Debug, Clone)]
pub enum Factory<'a> {
    AssemblingMachine(&'a AssemblingMachine, &'a Recipe),
    MiningDrill(&'a MiningDrill, &'a Resource)
}

impl<'a> Factory<'a> {
    fn produced_per_sec(&self) -> Vec<(String, f32)> {
        match self {
            Factory::AssemblingMachine(am, re) => re
                .produces()
                .into_iter()
                .map(|(name, amount)| (name, (am.crafting_speed / re.energy_required()) * amount))
                .collect(),
            Factory::MiningDrill(md, re) => {
                let temp: Vec<(String, f32)> = (&re.results).into();
                temp.into_iter().map(|(name, amount)| (name, (md.mining_speed / re.mining_time) * amount)).collect()
            }
        }
    }

    fn consumed_per_sec(&self) -> Vec<(String, f32)> {
        match self {
            Factory::AssemblingMachine(a, r) => r.consumes().into_iter().map(|(name, amount)| (name, r.energy_required() / a.crafting_speed * amount)).collect(),
            Factory::MiningDrill(md, re) => {
                if let Some(fluid_requirement) = &re.fluid_requirement {
                    vec![(fluid_requirement.required_fluid.clone(), fluid_requirement.fluid_amount * (re.mining_time / md.mining_speed))]
                } else {
                    vec![]
                }
            }
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

#[derive(Debug, Clone)]
struct InputItem;

#[derive(Debug, Clone, PartialEq, Properties)]
struct InputItemProps {
    item: String,
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

        let on_item_selected = link.batch_callback(|e: Event| {
            let target: Option<EventTarget> = e.target();

            let input = target.and_then(|t| t.dyn_into::<HtmlInputElement>().ok());
            input.and_then(|i| Some(InputItemMessage::ItemSelected(i.value().parse().ok()?)))
        });

        html! {
            <li class="target">
                // Remove this item from the list
                <button class="remove-item" onclick={link.callback(|_| InputItemMessage::Remove)}> {"x"} </button>
                // Change this item's target
                <button class="target-item" onclick={link.callback(|_| InputItemMessage::OpenItem)}> <ItemIcon item={props.item.clone()}/> </button>
                // Input factories
                {"Factories: "}
                <input type="text" onchange={on_factories_change} value={props.factories.to_string()} />
                // Input Items Per Second
                {"items/s: "}
                <input type="text" onchange={on_ips_change} value={props.items_per_second.to_string()}/>
                // Input item manually
                {"item: "}
                <input type="text" onchange={on_item_selected} value={props.item.clone()}/>
            </li>
        }
    }
}

#[derive(Debug, Clone)]
pub struct ItemIcon;

#[derive(Debug, Clone, PartialEq, Properties)]
pub struct ItemIconProperties {
    item: String
}

impl Component for ItemIcon {
    type Message = ();
    type Properties = ItemIconProperties;

    fn create(ctx: &Context<Self>) -> Self {
        Self
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();
        let pos = ICON_MAP.get(&format!("item-{}", props.item)).or_else(|| ICON_MAP.get(&format!("item-{}", UNKNOWN_ITEM)))
            .unwrap_or(&(ORIGINAL_SPRITESHEET_SIZE - ORIGINAL_ICON_SIZE,
                    ORIGINAL_SPRITESHEET_SIZE - ORIGINAL_ICON_SIZE));
        html! {
            <img src="assets/empty.gif" style={format!("background-image: url(\"assets/generated/spritesheet.png\"); background-position-x: -{0}px; background-position-y: -{1}px; width: {2}px; height: {2}px; background-size: {3}px;", pos.0 / DOWNSCALE, pos.1 / DOWNSCALE, ICON_SIZE, SPRITESHEET_SIZE)}/>
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
                <button class="add-item" onclick={link.callback(|_| ())}> {"+"} </button>
            </li>
        }
    }
}

fn main() {
    yew::start_app::<Calculator>();
}
