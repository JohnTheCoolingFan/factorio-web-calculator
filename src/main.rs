use std::collections::{HashMap, hash_map::Entry};

use wasm_bindgen::JsCast;
use yew::events::Event;
use web_sys::{EventTarget, HtmlInputElement};
use yew::prelude::*;
use yew_router::prelude::*;
use serde::Deserialize;

const DEFAULT_ITEM: &str = "electronic_circuit";

struct Calculator {
    targets: HashMap<String, CalcTarget>,
}


#[derive(Debug, Clone, PartialEq)]
enum CalculatorMessage {
    RemoveItem(String),
    AddItem(String, CalcTarget),
    ChangeItem(String, String),
    ChangeRate(String, CalcTargetRate)
}

impl Component for Calculator {
    type Message = CalculatorMessage;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        let item_name = DEFAULT_ITEM.to_string();
        Self{targets: HashMap::from([(item_name, CalcTarget{rate: CalcTargetRate::Factories(1.0)})])}
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            CalculatorMessage::AddItem(name, target) => {
                self.targets.insert(name, target);
                true
            },
            CalculatorMessage::RemoveItem(name) => {
                self.targets.retain(|k, _| k != &name);
                true
            },
            CalculatorMessage::ChangeItem(from, to) => {
                if from != to {
                    let target = self.targets.remove(&from);
                    target.and_then(|t| self.targets.insert(to, t)).is_some()
                } else {
                    false
                }
            }
            CalculatorMessage::ChangeRate(name, rate) => {
                if let Some(target) = self.targets.get_mut(&name) {
                    target.rate = rate;
                    true
                } else {
                    false
                }
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
                <p><button onclick={link.callback(|_| CalculatorMessage::AddItem(DEFAULT_ITEM.to_string(),
                        CalcTarget::default()))}> {"+"} </button></p>
                <InputList>
                { for targets.iter().enumerate().map(|(i, t)| 
                    html_nested! { <InputItem
                        item={t.0.clone()}
                        factories={t.1.rate.as_factories(1.0)}
                        items_per_second={t.1.rate.as_ips(1.0)}
                        onchanged={link.callback(|m| m)}
                        index = {i} /> }
                ) }
                </InputList>
            </div>
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
struct CalcTarget {
    rate: CalcTargetRate
}

#[derive(Debug, Clone, PartialEq)]
enum CalcTargetRate {
    Factories(f64),
    ItemsPerSecond(f64)
}

impl Default for CalcTargetRate {
    fn default() -> Self {
        Self::Factories(1.0)
    }
}

impl CalcTargetRate {
    fn as_factories(&self, factory_ips: f64) -> f64 {
        match self {
            Self::Factories(f) => *f,
            Self::ItemsPerSecond(i) => i / factory_ips
        }
    }

    fn as_ips(&self, factory_ips: f64) -> f64 {
        match self {
            Self::Factories(f) => f * factory_ips,
            Self::ItemsPerSecond(i) => *i
        }
    }
}

#[derive(Debug)]
struct InputList;

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

#[derive(Debug, PartialEq, Properties)]
struct InputListProperties {
    #[prop_or_default]
    children: ChildrenWithProps<InputItem>,
}

#[derive(Debug, Clone)]
struct InputItem;

#[derive(Debug, Clone, PartialEq, Properties)]
struct InputItemProps {
    #[prop_or("electronic-circuit".to_string())]
    item: String,
    #[prop_or(1.0)]
    factories: f64,
    #[prop_or(1.0)]
    items_per_second: f64,
    onchanged: Callback<<Calculator as Component>::Message>,
    index: usize
}

#[derive(Debug)]
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
                callback.emit(CalculatorMessage::ChangeItem(props.item.clone(), s));
            },
            InputItemMessage::Factories(a) => {
                callback.emit(CalculatorMessage::ChangeRate(props.item.clone(), CalcTargetRate::Factories(a)));
            },
            InputItemMessage::ItemsPerSecond(a) => {
                callback.emit(CalculatorMessage::ChangeRate(props.item.clone(), CalcTargetRate::ItemsPerSecond(a)));
            },
            InputItemMessage::Remove => {
                callback.emit(CalculatorMessage::RemoveItem(props.item.clone()))
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
                <button class="target_button"> { props.item.clone() } </button>
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

fn main() {
    yew::start_app::<Calculator>();
}
