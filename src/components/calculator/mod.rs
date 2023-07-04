mod calc_step;
mod calc_target;
mod calc_target_rate;
mod calculation;
mod factory;
mod input_list;

pub use calc_step::*;
pub use calc_target::*;
pub use calc_target_rate::*;
pub use calculation::*;
pub use factory::*;
pub use input_list::*;

use crate::{
    components::{FactoryStep, FactorySteps},
    Route,
};
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Debug)]
pub struct Calculator {
    pub targets: Vec<CalcTarget>,
    pub calculation: Result<Calculation, CalculationError>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CalculatorMessage {
    RemoveItem(usize),
    AddItem(CalcTarget),
    ChangeItem(usize, String),
    ChangeRate(usize, CalcTargetRate),
}

impl Component for Calculator {
    type Message = CalculatorMessage;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        let targets = vec![CalcTarget::default()];
        let calculation = Calculation::default().solve(&targets);
        Self {
            targets,
            calculation,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            CalculatorMessage::AddItem(target) => {
                self.targets.push(target);
            }
            CalculatorMessage::RemoveItem(idx) => {
                self.targets.remove(idx);
            }
            CalculatorMessage::ChangeItem(idx, name) => {
                self.targets[idx].name = name;
            }
            CalculatorMessage::ChangeRate(idx, rate) => {
                self.targets[idx].rate = rate;
            }
        }
        self.calculation = Calculation::default().solve(&self.targets);
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let targets = &self.targets;
        let steps: Vec<CalcStep> = self
            .calculation
            .as_ref()
            .map(|calc| {
                calc.steps
                    .iter()
                    .map(|(factory, amount)| CalcStep {
                        factory: factory.clone(),
                        amount: *amount,
                    })
                    .collect()
            })
            .unwrap_or_default();
        let link = ctx.link();
        log::info!("number of steps: {}", steps.len());
        html! {
            <div id="calc">
                <p> { "This is a calculator" } </p>
                <p> { format!("Version: {}", env!("CARGO_PKG_VERSION")) } </p>
                <p> { "Source code is available at " } <a href={"https://github.com/JohnTheCoolingFan/factorio-web-calculator"}>{"GitHub repo"}</a> </p>
                <p> { "Please report any issues you encounter" } </p>
                <p> <Link<Route> to={Route::Settings}>{"Settings"}</Link<Route>> </p>
                <p> { "Current targets:" } </p>
                <InputList>
                { for targets.iter().enumerate().map(|(i, t)| {
                    html_nested! { <InputItem
                        item={t.name.clone()}
                        rate={t.rate.clone()}
                        onchanged={link.callback(|m| m)}
                        index = {i} /> }
                }) }
                <AddItem onclick={link.callback(|m| m)}/>
                </InputList>
                <p>{ if let Err(why) = &self.calculation { format!("An error occured: {}", why) } else { "no errors".into() } }</p>
                <FactorySteps>
                {
                    for steps.iter().map(|step| {
                        html_nested! { <FactoryStep step={step.clone()} /> }
                    })
                }
                </FactorySteps>
                <h6>{ "Warning: might contain slight side-effects including but not limited to 3200 oil refineries" }</h6>
            </div>
        }
    }
}
