mod step;

pub use step::*;

use yew::prelude::*;

#[derive(Debug)]
pub struct FactorySteps;

#[derive(Debug, Clone, PartialEq, Properties)]
pub struct FactoryStepsProperties {
    #[prop_or_default]
    pub children: ChildrenWithProps<FactoryStep>,
}

impl Component for FactorySteps {
    type Message = ();
    type Properties = FactoryStepsProperties;

    fn create(_ctx: &Context<Self>) -> Self {
        Self
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <ul class="factory-steps">
                { for ctx.props().children.iter() }
            </ul>
        }
    }
}
