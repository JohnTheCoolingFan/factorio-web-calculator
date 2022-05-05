mod data;

use data::*;
use thiserror::Error;
use std::collections::HashMap;
use wasm_bindgen::JsCast;
use yew::{events::Event, html::ChildrenRenderer};
use web_sys::{EventTarget, HtmlInputElement};
use yew::{virtual_dom::VChild, prelude::*};
use once_cell::sync::Lazy;

const DEFAULT_ITEM: &str = "electronic-circuit";
const UNKNOWN_ITEM: &str = "item-unknown";
const ORIGINAL_SPRITESHEET_SIZE: usize = 960;
const ORIGINAL_ICON_SIZE: usize = 64;
const DOWNSCALE: usize = 2;
const SPRITESHEET_SIZE: usize = ORIGINAL_SPRITESHEET_SIZE / DOWNSCALE;
const ICON_SIZE: usize = ORIGINAL_ICON_SIZE / DOWNSCALE;
const RECURSION_LIMIT: usize = 500;
const VERY_SMALL: f64 = 1e-10;

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
        let targets = vec![CalcTarget::default()];
        let calculation = Calculation::default().solve(&targets);
        Self {
            targets,
            calculation
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            CalculatorMessage::AddItem(target) => {
                self.targets.push(target);
            },
            CalculatorMessage::RemoveItem(idx) => {
                self.targets.remove(idx);
            },
            CalculatorMessage::ChangeItem(idx, name) => {
                let item = self.targets.get_mut(idx).unwrap();
                item.name = name;
            }
            CalculatorMessage::ChangeRate(idx, rate) => {
                let item = self.targets.get_mut(idx).unwrap();
                item.rate = rate;
            }
        }
        self.calculation = Calculation::default().solve(&self.targets);
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let targets = &self.targets;
        let steps = if let Ok(calculation) = &self.calculation {
            calculation.steps.clone()
        } else {
            vec![]
        };
        let link = ctx.link();
        log::info!("number of steps: {}", steps.len());
        html! {
            <div id="calc">
                <p> { "This is a calculator" } </p>
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

#[derive(Debug, Error)]
pub enum CalculationError {
    #[error("Recipe or Resource for item {0} npt found")]
    RecipeOrResourceNotFound(String),
    #[error("Assembling machine for recipe {0} not found")]
    AssemblingMachineNotFound(String),
    #[error("Minming Drill for resource {0} not found")]
    MiningDrillNotFound(String),
    #[error("Recursion limit")]
    RecursionLimit,
    #[error("No item to pick")]
    NoItemToPick
}

#[derive(Debug, Clone, Default)]
pub struct Calculation {
    vector: HashMap<String, f64>,
    pub steps: Vec<CalcStep>
}

impl Calculation {
    pub fn solve(mut self, input: &[CalcTarget]) -> Result<Self, CalculationError> {
        for target in input {
            let name = &target.name.clone();
            let factory = Factory::for_item(name)?;
            let items_per_second = target.rate.as_ips(factory.item_produced_per_sec(name));
            self.vector.insert(name.clone(), -items_per_second);
        }

        let mut recursion_limit = RECURSION_LIMIT;
        while !self.is_solved() && recursion_limit > 0 {
            recursion_limit -= 1;
            let item = self.pick_item().ok_or(CalculationError::NoItemToPick)?;
            let factory = Factory::for_item(&item.0)?;
            let amount = (item.1 * factory.energy_required()) / factory.crafting_speed();
            let step = CalcStep { factory, amount };
            self.apply_step(step);
        }

        if recursion_limit == 0 {
            return Err(CalculationError::RecursionLimit)
        }

        Ok(self)
    }

    pub fn apply_step(&mut self, step: CalcStep) {
        log::info!("Applying step");
        let produced = step.produced_per_sec();
        let consumed = step.consumed_per_sec();

        for (name, amount) in &produced {
            let val = self.vector.entry(name.clone()).or_insert(0.0);
            *val += amount;
        }

        for (name, amount) in &consumed {
            let val = self.vector.entry(name.clone()).or_insert(0.0);
            *val -= amount;
        }
        self.steps.push(step)
    }

    fn is_solved(&self) -> bool {
        //self.vector.values().sum::<f64>() == 0.0
        self.vector.values().all(|i| (*i >= 0.0) || (i.abs() < VERY_SMALL))
    }

    fn pick_item(&self) -> Option<(String, f64)> {
        log::info!("Picking an item");
        for (name, value) in &self.vector {
            log::info!("Trying {}, {}", name, value);
            if (value < &0.0) && (value.abs() > VERY_SMALL) {
                log::info!("Picked!");
                return Some((name.clone(), -value))
            }
        }
        None
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CalcStep {
    factory: Factory<'static>,
    amount: f64
}

impl CalcStep {
    fn produced_per_sec(&self) -> Vec<(String, f64)> {
        self.factory.produced_per_sec().into_iter().map(|(name, amount)| (name, amount * self.amount)).collect()
    }

    fn consumed_per_sec(&self) -> Vec<(String, f64)> {
        self.factory.consumed_per_sec().into_iter().map(|(name, amount)| (name, amount * self.amount)).collect()
    }
    
    fn machine_name(&self) -> String {
        self.factory.name()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Factory<'a> {
    AssemblingMachine(&'a AssemblingMachine, &'a Recipe),
    MiningDrill(&'a MiningDrill, &'a Resource),
    OffshorePump(&'a OffshorePump)
}

impl<'a> Factory<'a> {
    fn produced_per_sec(&self) -> Vec<(String, f64)> {
        match self {
            Factory::AssemblingMachine(am, re) => re
                .produces()
                .into_iter()
                .map(|(name, amount)| (name, (am.crafting_speed / re.energy_required()) * amount))
                .collect(),
            Factory::MiningDrill(md, re) => {
                let temp: Vec<(String, f64)> = (&re.results).into();
                temp.into_iter().map(|(name, amount)| (name, (md.mining_speed / re.mining_time) * amount)).collect()
            },
            Factory::OffshorePump(op) => vec![(op.fluid.clone(), op.pumping_speed)]
        }
    }

    fn item_produced_per_sec(&self, item: &str) -> f64 {
        for product in self.produced_per_sec() {
            if product.0 == item {
                return product.1
            }
        }
        0.0
    }

    fn consumed_per_sec(&self) -> Vec<(String, f64)> {
        match self {
            Factory::AssemblingMachine(a, r) => r.consumes().into_iter().map(|(name, amount)| (name, r.energy_required() / a.crafting_speed * amount)).collect(),
            Factory::MiningDrill(md, re) => {
                if let Some(fluid_requirement) = &re.fluid_requirement {
                    vec![(fluid_requirement.required_fluid.clone(), fluid_requirement.fluid_amount * (re.mining_time / md.mining_speed))]
                } else {
                    vec![]
                }
            },
            _ => vec![]
        }
    }

    fn icon_prefix(&self) -> &str {
        match self {
            Factory::AssemblingMachine(_, _) => "assembling-machine",
            Factory::MiningDrill(_, _) => "mining-drill",
            Factory::OffshorePump(_) => "offshore-pump"
        }
    }

    fn name(&self) -> String {
        match self {
            Factory::AssemblingMachine(am, _) => am.name.clone(),
            Factory::MiningDrill(md, _) => md.name.clone(),
            Factory::OffshorePump(op) => op.name.clone()
        }
    }

    fn ips_for_item(item: &str) -> f64 {
        if let Ok(factory) = Self::for_item(item) {
            for (name, amount) in factory.produced_per_sec() {
                if name == item {
                    return amount
                }
            }
        }
        1.0
    }

    fn crafting_speed(&self) -> f64 {
        match self {
            Factory::AssemblingMachine(am, _) => am.crafting_speed,
            Factory::MiningDrill(md, _) => md.mining_speed,
            Factory::OffshorePump(op) => op.pumping_speed
        }
    }

    fn energy_required(&self) -> f64 {
        match self {
            Factory::AssemblingMachine(_, recipe) => recipe.energy_required(),
            Factory::MiningDrill(_, resource) => resource.mining_time,
            Factory::OffshorePump(_) => 1.0
        }
    }

    fn for_item(item: &str) -> Result<Self, CalculationError> {
        if let Some(offshore_pump) = Self::find_offshore_pump_for_item(item) {
            Ok(Self::OffshorePump(offshore_pump))
        } else if let Some(resource) = Self::find_resource_for_item(item) {
            if let Some(mining_drill) = Self::find_mining_drill_for_resource(&resource.category) {
                Ok(Self::MiningDrill(mining_drill, resource))
            } else {
                Err(CalculationError::MiningDrillNotFound(resource.category.clone()))
            }
        } else if let Some(recipe) = Self::find_recipe_for_item(item) {
            if let Some(assembling_machine) = Self::find_assembling_machine_for_recipe(&recipe.category) {
                Ok(Self::AssemblingMachine(assembling_machine, recipe))
            } else {
                Err(CalculationError::AssemblingMachineNotFound(recipe.category.clone()))
            }
        } else {
            Err(CalculationError::RecipeOrResourceNotFound(item.into()))
        }
    }

    fn find_recipe_for_item(item: &str) -> Option<&'static Recipe> {
        for recipe in GAME_DATA.recipes.values() {
            if recipe.produces().iter().any(|(x, _)| x == item) {
                return Some(recipe)
            }
        }
        None
    }

    fn find_assembling_machine_for_recipe(recipe_category: &str) -> Option<&'static AssemblingMachine> {
        for assembling_machine in GAME_DATA.assembling_machines.values() {
            if assembling_machine.crafting_categories.iter().any(|c| c == recipe_category) {
                return Some(assembling_machine)
            }
        }
        None
    }

    fn find_resource_for_item(item: &str) -> Option<&'static Resource> {
        for resource in GAME_DATA.resources.values() {
            let results: Vec<(String, f64)> = (&resource.results).into();
            if results.iter().any(|(x, _)| x == item) {
                return Some(resource)
            }
        }
        None
    }

    fn find_mining_drill_for_resource(resource_category: &str) -> Option<&'static MiningDrill> {
        for mining_drill in GAME_DATA.mining_drills.values() {
            if mining_drill.resource_categories.iter().any(|c| c == resource_category) {
                return Some(mining_drill)
            }
        }
        None
    }

    fn find_offshore_pump_for_item(item: &str) -> Option<&'static OffshorePump> {
        for offshore_pump in GAME_DATA.offshore_pumps.values() {
            if offshore_pump.fluid == item {
                return Some(offshore_pump)
            }
        }
        None
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
    rate: CalcTargetRate,
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

        let ips = Factory::ips_for_item(&props.item);

        html! {
            <li class="target">
                // Remove this item from the list
                <button class="remove-item" onclick={link.callback(|_| InputItemMessage::Remove)}> {"x"} </button>
                // Change this item's target
                <button class="target-item" onclick={link.callback(|_| InputItemMessage::OpenItem)}> <ItemIcon item={props.item.clone()}/> </button>
                // Input factories
                {"Factories: "}
                <input type="text" onchange={on_factories_change} value={props.rate.as_factories(ips).to_string()} />
                // Input Items Per Second
                {"items/s: "}
                <input type="text" onchange={on_ips_change} value={props.rate.as_ips(ips).to_string()}/>
                // Input item manually
                {"item: "}
                <input type="text" onchange={on_item_selected} value={props.item.clone()}/>
            </li>
        }
    }
}

#[derive(Debug)]
pub struct SpriteSheetIcon;

#[derive(Debug, Clone, PartialEq, Properties)]
pub struct SpriteSheetIconProperties {
    prefix: String,
    name: String
}

impl Component for SpriteSheetIcon {
    type Message = ();
    type Properties = SpriteSheetIconProperties;

    fn create(_ctx: &Context<Self>) -> Self {
        Self
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();
        let pos = ICON_MAP.get(&format!("{}-{}", props.prefix, props.name))
            .or_else(|| ICON_MAP.get(&format!("item-{}", UNKNOWN_ITEM)))
            .unwrap_or(&(ORIGINAL_SPRITESHEET_SIZE - ORIGINAL_ICON_SIZE,
                    ORIGINAL_SPRITESHEET_SIZE - ORIGINAL_ICON_SIZE));
        html! {
            <img src="assets/empty.gif" title={props.name.clone()} alt={props.name.clone()} style={ format!("background-image: url(\"assets/generated/spritesheet.png\"); background-position-x: -{0}px; background-position-y: -{1}px; width: {2}px; height: {2}px; background-size: {3}px", pos.0 / DOWNSCALE, pos.1 / DOWNSCALE, ICON_SIZE, SPRITESHEET_SIZE) }/>
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

    fn create(_ctx: &Context<Self>) -> Self {
        Self
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();
        html! {
            <SpriteSheetIcon prefix={"item"} name={props.item.clone()} />
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

#[derive(Debug)]
pub struct FactorySteps;

#[derive(Debug, Clone, PartialEq, Properties)]
pub struct FactoryStepsProperties {
    #[prop_or_default]
    children: ChildrenWithProps<FactoryStep>
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

#[derive(Debug)]
pub struct FactoryStep;

#[derive(Debug, Clone, PartialEq, Properties)]
pub struct FactoryStepProperties {
    step: CalcStep
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
        html!{
            <li><p>
                {format!("{:.3}x ", famount)}
                <SpriteSheetIcon prefix={props.step.factory.icon_prefix().to_string()} name={props.step.machine_name()} />
                {" producing "}
                {
                    for props.step.produced_per_sec().iter().map(|(name, amount)| {
                        html_nested! {
                            <>
                            <SpriteSheetIcon prefix={"item".to_string()} name={name.clone()}/>
                            {format!("x{:.3}; ", amount)}
                            </>
                        }
                    })
                }
            </p></li>
        }
    }
}

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::start_app::<Calculator>();
}
