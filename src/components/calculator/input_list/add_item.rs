use yew::prelude::*;

use crate::components::{CalcTarget, Calculator, CalculatorMessage};

#[derive(Debug, Clone)]
pub struct AddItem;

#[derive(Debug, Clone, PartialEq, Properties)]
pub struct AddItemProperties {
    pub onclick: Callback<<Calculator as Component>::Message>,
}

impl Component for AddItem {
    type Message = ();
    type Properties = AddItemProperties;

    fn create(_ctx: &Context<Self>) -> Self {
        Self
    }

    fn update(&mut self, ctx: &Context<Self>, _msg: Self::Message) -> bool {
        ctx.props()
            .onclick
            .emit(CalculatorMessage::AddItem(CalcTarget::default()));
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
