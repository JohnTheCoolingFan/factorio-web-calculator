mod calc_step;
mod calc_target;
mod calc_target_rate;
mod calculation;
mod factory;
mod input_list;

use std::rc::Rc;

pub use calc_step::*;
pub use calc_target::*;
pub use calc_target_rate::*;
pub use calculation::*;
use factorio_web_calculator::data::GameData;
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
    pub calculation: Option<Result<Calculation, CalculationError>>,
    #[allow(dead_code)]
    game_data_context_listener: ContextHandle<Option<Rc<GameData>>>,
    pub game_data: Option<Rc<GameData>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CalculatorMessage {
    GameDataUpdated(Option<Rc<GameData>>),
    RemoveItem(usize),
    AddItem(CalcTarget),
    ChangeItem(usize, String),
    ChangeRate(usize, CalcTargetRate),
}

impl Component for Calculator {
    type Message = CalculatorMessage;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let scope = ctx.link();
        let (game_data, game_data_context_listener) = scope
            .context(scope.callback(CalculatorMessage::GameDataUpdated))
            .expect("Game Data context was not provided");

        Self {
            targets: vec![],
            calculation: None,
            game_data,
            game_data_context_listener,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match (&self.game_data, msg) {
            (None, CalculatorMessage::GameDataUpdated(game_data)) => {
                self.targets = vec![CalcTarget::default()];
                self.game_data = game_data
            }
            (Some(_), CalculatorMessage::GameDataUpdated(game_data)) => {
                log::warn!("Changed game data when it is already set");
                self.targets = vec![CalcTarget::default()];
                self.game_data = game_data
            }
            // If game data is not available then other messages don't change anything
            (None, _) => return false,
            (Some(_), CalculatorMessage::AddItem(target)) => {
                self.targets.push(target);
            }
            (Some(_), CalculatorMessage::RemoveItem(idx)) => {
                self.targets.remove(idx);
            }
            (Some(_), CalculatorMessage::ChangeItem(idx, name)) => {
                self.targets[idx].name = name;
            }
            (Some(_), CalculatorMessage::ChangeRate(idx, rate)) => {
                self.targets[idx].rate = rate;
            }
        }
        self.calculation = Some(Calculation::default().solve(&self.targets));
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let targets = &self.targets;
        // Silenced the warning because sorting the resulting steps is planned
        #[allow(unused_mut)]
        let mut steps: Vec<CalcStep> = self
            .calculation
            .as_ref()
            .and_then(|rescalc| rescalc.as_ref().ok())
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
        // Commented out because this doesn't work well with optimized sorting algos
        //steps.sort_by(|cs1, cs2| cs1.factory.sort_by(&cs2.factory));
        let link = ctx.link();
        log::info!("number of steps: {}", steps.len());
        let status_message = if let Some(rescalc) = &self.calculation {
            if let Err(why) = rescalc {
                format!("An error occured: {}", why)
            } else {
                String::from("No errors during calculation")
            }
        } else {
            String::from("Calculation is not ready yet")
        };
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
                <p>{ status_message }</p>
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
