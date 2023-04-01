use yew::prelude::*;

use crate::components::{CalcStep, SpriteSheetIcon};

#[derive(Debug)]
pub struct FactoryStep;

#[derive(Debug, Clone, PartialEq, Properties)]
pub struct FactoryStepProperties {
    pub step: CalcStep,
}

impl Component for FactoryStep {
    type Message = ();
    type Properties = FactoryStepProperties;

    fn create(_ctx: &Context<Self>) -> Self {
        Self
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();
        let famount = props.step.amount;
        html! {
            <li><p>
                {format!("{}x ", format!("{:.3}", famount).trim_end_matches('0').trim_end_matches('.'))}
                <SpriteSheetIcon prefix={props.step.factory.icon_prefix().to_string()} name={props.step.machine_name()} />
                {" producing "}
                {
                    for props.step.produced_per_sec().iter().map(|(name, amount)| {
                        html_nested! {
                            <>
                            <SpriteSheetIcon prefix={"item".to_string()} name={name.clone()}/>
                            {format!("{}; ", format!("x{:.3}", amount).trim_end_matches('0').trim_end_matches('.'))}
                            </>
                        }
                    })
                }
            </p></li>
        }
    }
}
