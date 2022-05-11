mod data;

use data::*;
use gloo_storage::Storage;
use thiserror::Error;
use std::{collections::HashMap, sync::RwLock, cmp::Ordering};
use wasm_bindgen::JsCast;
use web_sys::{EventTarget, HtmlInputElement};
use yew::{virtual_dom::VChild, events::Event, html::ChildrenRenderer, prelude::*};
use yew_router::prelude::*;
use once_cell::sync::Lazy;
use serde::{Serialize, Deserialize};

const DEFAULT_ITEM: &str = "electronic-circuit";
const UNKNOWN_ITEM: &str = "item-unknown";
const ORIGINAL_SPRITESHEET_SIZE: usize = 960;
const ORIGINAL_ICON_SIZE: usize = 64;
const DOWNSCALE: usize = 2;
const SPRITESHEET_SIZE: usize = ORIGINAL_SPRITESHEET_SIZE / DOWNSCALE;
const ICON_SIZE: usize = ORIGINAL_ICON_SIZE / DOWNSCALE;
const RECURSION_LIMIT: usize = 5000;
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
pub struct UserSettings {
    recipe_category_prefs: HashMap<String, &'static AssemblingMachine>,
    resource_category_prefs: HashMap<String, &'static MiningDrill>
}

impl UserSettings {
    fn assembling_machine(&self, category: &str) -> Option<&'static AssemblingMachine> {
        Some(*self.recipe_category_prefs.get(category)?)
    }
    
    fn mining_drill(&self, category: &str) -> Option<&'static MiningDrill> {
        Some(*self.resource_category_prefs.get(category)?)
    }

    fn change_recipe_category(&mut self, category: &str, machine: &'static AssemblingMachine) {
        log::info!("Changed assembler for categpry {} to {}", category, machine.name);
        self.recipe_category_prefs.insert(category.to_string(), machine);
        self.write()
    }

    fn change_resource_category(&mut self, category: &str, machine: &'static MiningDrill) {
        log::info!("Changed mining drill for category {} to {}", category, machine.name);
        self.resource_category_prefs.insert(category.to_string(), machine);
        self.write()
    }

    fn write(&self) {
        gloo_storage::LocalStorage::set("user_settings", RawUserSettings::from(self)).unwrap();
    }

    fn init() -> Self {
        log::info!("User settings init");
        let mut recipe_category_prefs = HashMap::new();
        for (recipe_category, mut assemblers) in GAME_DATA.recipe_categories_with_multiple_assemblers() {
            assemblers.sort_by(|x, y| x.crafting_speed.partial_cmp(&y.crafting_speed).unwrap_or(Ordering::Equal));
            log::info!("assembler for category {}: {}", recipe_category, assemblers[0].name);
            recipe_category_prefs.insert(recipe_category, assemblers[0]);
        }
        let mut resource_category_prefs = HashMap::new();
        for (resource_category, mut mining_drills) in GAME_DATA.resource_categories_with_multiple_mining_drills() {
            mining_drills.sort_by(|x, y| x.mining_speed.partial_cmp(&y.mining_speed).unwrap_or(Ordering::Equal));
            log::info!("mining drill for category {}: {}", resource_category, mining_drills[0].name);
            resource_category_prefs.insert(resource_category, mining_drills[0]);
        }
        let result = Self{recipe_category_prefs, resource_category_prefs};
        result.write();
        result
    }

    fn from_raw(raw_us: RawUserSettings) -> Self {
        log::info!("Loading user settings");
        Self {
            recipe_category_prefs: raw_us.recipe_category_prefs.into_iter().map(|(cat, am)| (cat, GAME_DATA.assembling_machines.get(&am).unwrap())).collect(),
            resource_category_prefs: raw_us.resource_category_prefs.into_iter().map(|(cat, md)| (cat, GAME_DATA.mining_drills.get(&md).unwrap())).collect()
        }
    }

    fn create() -> Self {
        if let Ok(us) = gloo_storage::LocalStorage::get("user_settings") {
            Self::from_raw(us)
        } else {
            Self::init()
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RawUserSettings {
    recipe_category_prefs: HashMap<String, String>,
    resource_category_prefs: HashMap<String, String>
}

impl From<&UserSettings> for RawUserSettings {
    fn from(us: &UserSettings) -> Self {
        Self {
            recipe_category_prefs: us.recipe_category_prefs.iter().map(|(cat, am)| (cat.clone(), am.name.clone())).collect(),
            resource_category_prefs: us.resource_category_prefs.iter().map(|(cat, md)| (cat.clone(), md.name.clone())).collect()
        }
    }
}

static USER_SETTINGS: Lazy<RwLock<UserSettings>> = Lazy::new(|| {
    let result = UserSettings::create();
    RwLock::new(result)
});

#[derive(Debug, Clone, Routable, PartialEq)]
enum Route {
    #[at("/")]
    Home,
    #[at("/settings")]
    Settings
}

fn switch(route: &Route) -> Html {
    match route {
        Route::Home => html! { <Calculator /> },
        Route::Settings => html!{ <UserSettingsPage /> }
    }
}

#[function_component(MainApp)]
fn app() -> Html {
    html! {
        <BrowserRouter>
            <Switch<Route> render={Switch::render(switch)} />
        </BrowserRouter>
    }
}

#[derive(Debug)]
pub struct UserSettingsPage;

#[derive(Debug, Clone, PartialEq)]
pub enum UserSettingsPageMessage {
    ChangeAssembler(String, &'static AssemblingMachine),
    ChangeMiningDrill(String, &'static MiningDrill)
}

impl Component for UserSettingsPage {
    type Message = UserSettingsPageMessage;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <div id="usersettings">
                <p><a href="/">{"Go back"}</a></p>
                <div id="usersettings_assemblingmachine">
                    <ul>
                    {
                        for GAME_DATA.recipe_categories_with_multiple_assemblers().iter().map(|v| {
                            html_nested! {
                                <UserSettingRecipeCategory category={v.0.clone()} choices={v.1.clone()} />
                            }
                        })
                    }
                    </ul>
                </div>
                <div id="usersettings_miningdrill">
                    <p> {"mining drills TODO"} </p>
                </div>
            </div>
        }
    }
}

#[derive(Debug)]
pub struct UserSettingRecipeCategory;

#[derive(Debug, Clone, PartialEq, Properties)]
pub struct UserSettingRecipeCategoryProperties {
    category: String,
    choices: Vec<&'static AssemblingMachine>
}

impl Component for UserSettingRecipeCategory {
    type Properties = UserSettingRecipeCategoryProperties;
    type Message = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();
        html! {
            <li>
            <p> {props.category.clone()} </p>
            {
                for props.choices.iter().map(|am| {
                    html_nested! {
                        <label>
                            <input type="radio" name={format!("recipe-category-pref-{}", props.category)} checked={
                                USER_SETTINGS
                                    .read().ok().and_then(|us| {
                                        us.recipe_category_prefs
                                            .get(&props.category)
                                            .map(|amp| amp.name == am.name)
                                    })
                                .unwrap_or(false)} />
                            <SpriteSheetIcon name={am.name.clone()} prefix="assembling-machine"/>
                        </label>
                    }
                })
            }
            </li>
        }
    }
}

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
                <p> { "Source code is available at " } <a href={"https://github.com/JohnTheCoolingFan/factorio-web-calculator"}>{"GitHub repo"}</a> </p>
                <p> { "Please report any issues you encounter" } </p>
                <p> <a href="/settings">{"Settings"}</a> </p>
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
    #[error("Recipe or Resource for item {0} not found")]
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
            let mut amount = (item.1 * factory.energy_required()) / factory.crafting_speed();
            amount /= factory.item_produced_per_recipe(&item.0);
            log::info!("Amount will be divided by {}", factory.item_produced_per_recipe(&item.0));
            let step = CalcStep { factory, amount };
            self.apply_step(step);
        }

        if recursion_limit == 0 {
            return Err(CalculationError::RecursionLimit)
        }

        Ok(self)
    }

    pub fn apply_step(&mut self, step: CalcStep) {
        log::info!("Applying step in amount {:.3}", step.amount);
        let produced = step.produced_per_sec();
        let consumed = step.consumed_per_sec();

        for (name, amount) in &produced {
            log::info!("ingredient {} produced in amount of {:.3}", name, amount);
            let val = self.vector.entry(name.clone()).or_insert(0.0);
            *val += amount;
        }

        for (name, amount) in &consumed {
            log::info!("ingredient {} consumed in amount of {:.3}", name, amount);
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
            log::info!("Trying {}, {:.3}", name, value);
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

    fn item_produced_per_recipe(&self, item: &str) -> f64 {
        match self {
            Factory::AssemblingMachine(_, re) => {
                for product in &re.produces() {
                    if product.0 == item {
                        return product.1
                    }
                }
                0.0
            },
            Factory::MiningDrill(_, re) => {
                let products: Vec<(String, f64)> = (&re.results).into();
                for product in &products {
                    if product.0 == item {
                        return product.1
                    }
                }
                0.0
            },
            Factory::OffshorePump(_) => 1.0
        }
    }

    fn consumed_per_sec(&self) -> Vec<(String, f64)> {
        match self {
            Factory::AssemblingMachine(a, r) => r.consumes().into_iter().map(|(name, amount)| (name, (a.crafting_speed / r.energy_required()) * amount)).collect(),
            Factory::MiningDrill(md, re) => {
                if let Some(fluid_requirement) = &re.fluid_requirement {
                    vec![(fluid_requirement.required_fluid.clone(), fluid_requirement.fluid_amount * (md.mining_speed / re.mining_time))]
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
            if let Some(mining_drill) = USER_SETTINGS.read().ok().and_then(|us| us.mining_drill(&resource.category))
                .or_else(|| Self::find_mining_drill_for_resource(&resource.category)) {
                Ok(Self::MiningDrill(mining_drill, resource))
            } else {
                Err(CalculationError::MiningDrillNotFound(resource.category.clone()))
            }
        } else if let Some(recipe) = Self::find_recipe_for_item(item) {
            if let Some(assembling_machine) = USER_SETTINGS.read().ok().and_then(|us| us.assembling_machine(&recipe.category))
                .or_else(|| Self::find_assembling_machine_for_recipe(&recipe.category)) {
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
            if recipe.produces().iter().any(|(x, _)| x == item) && recipe.allow_decomposition() {
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

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::start_app::<MainApp>();
}
