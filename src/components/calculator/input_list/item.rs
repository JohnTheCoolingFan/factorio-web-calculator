use wasm_bindgen::JsCast;
use web_sys::{EventTarget, HtmlInputElement};
use yew::prelude::*;

use crate::components::{
    CalcTargetRate, Calculator, CalculatorMessage, Factory, ItemSelectDropdown,
};

#[derive(Debug, Clone)]
pub struct InputItem;

#[derive(Debug, Clone, PartialEq, Properties)]
pub struct InputItemProps {
    pub item: String,
    pub rate: CalcTargetRate,
    pub onchanged: Callback<<Calculator as Component>::Message>,
    pub index: usize,
}

#[derive(Debug, Clone)]
pub enum InputItemMessage {
    Remove,
    ItemSelected(String),
    Factories(f64),
    ItemsPerSecond(f64),
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
            }
            InputItemMessage::Factories(a) => {
                callback.emit(CalculatorMessage::ChangeRate(
                    props.index,
                    CalcTargetRate::Factories(a),
                ));
            }
            InputItemMessage::ItemsPerSecond(a) => {
                callback.emit(CalculatorMessage::ChangeRate(
                    props.index,
                    CalcTargetRate::ItemsPerSecond(a),
                ));
            }
            InputItemMessage::Remove => callback.emit(CalculatorMessage::RemoveItem(props.index)),
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();
        let link = ctx.link();

        let on_factories_change = link.batch_callback(|e: Event| {
            e.target()
                .and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
                .and_then(|i| Some(InputItemMessage::Factories(i.value().parse().ok()?)))
        });

        let on_ips_change = link.batch_callback(|e: Event| {
            e.target()
                .and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
                .and_then(|i| Some(InputItemMessage::ItemsPerSecond(i.value().parse().ok()?)))
        });

        let ips = Factory::ips_for_item(&props.item);

        html! {
            <li class="target">
                // Remove this item from the list
                <button class="remove-item" onclick={link.callback(|_| InputItemMessage::Remove)}> {"x"} </button>
                // Change this item's target
                //<button class="target-item" onclick={link.callback(|_| InputItemMessage::OpenItem)}> <ItemIcon item={props.item.clone()}/> </button>
                <ItemSelectDropdown index={props.index} selected_item={props.item.clone()} callback={link.callback(InputItemMessage::ItemSelected)} />
                // Input factories
                {"Factories: "}
                <input type="text" onchange={on_factories_change} value={props.rate.as_factories(ips).to_string()} />
                // Input Items Per Second
                {"items/s: "}
                <input type="text" onchange={on_ips_change} value={props.rate.as_ips(ips).to_string()}/>
                // Input item manually
                //{"item: "}
                //<input type="text" onchange={on_item_selected} value={props.item.clone()}/>
            </li>
        }
    }
}
