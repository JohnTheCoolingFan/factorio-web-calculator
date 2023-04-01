use wasm_bindgen::JsCast;
use web_sys::{EventTarget, HtmlInputElement};
use yew::prelude::*;

use crate::constants::{DEFAULT_ITEM, GAME_DATA};

use super::ItemIcon;

#[derive(Debug)]
pub struct ItemSelectDropdown {
    is_open: bool,
    selected_item: String,
}

#[derive(Debug, Clone, PartialEq, Properties)]
pub struct ItemSelectDropdownProperties {
    pub callback: Callback<String>,
    pub index: usize,
    #[prop_or_else(default_item)]
    pub selected_item: String,
}

fn default_item() -> String {
    DEFAULT_ITEM.into()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ItemSelectDropdownMessage {
    OpenDropdown,
    CloseDropdown,
    ToggleDropdown,
    ItemSelected(String),
}

impl Component for ItemSelectDropdown {
    type Properties = ItemSelectDropdownProperties;
    type Message = ItemSelectDropdownMessage;

    fn create(ctx: &Context<Self>) -> Self {
        let props = ctx.props();
        Self {
            is_open: false,
            selected_item: props.selected_item.clone(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let props = ctx.props();
        match msg {
            ItemSelectDropdownMessage::OpenDropdown => {
                if self.is_open {
                    return false;
                }
                self.is_open = true
            }
            ItemSelectDropdownMessage::CloseDropdown => self.is_open = false,
            ItemSelectDropdownMessage::ToggleDropdown => self.is_open = !self.is_open,
            ItemSelectDropdownMessage::ItemSelected(item) => {
                self.selected_item = item.clone();
                self.is_open = false;
                props.callback.emit(item)
            }
        };
        true // Maybe don't re-render when selected?
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let link = ctx.link();
        let props = ctx.props();

        let on_item_selected = link.batch_callback(|e: Event| {
            log::info!("item selected");
            let target: Option<EventTarget> = e.target();
            let input = target.and_then(|t| t.dyn_into::<HtmlInputElement>().ok());
            input.map(|i| ItemSelectDropdownMessage::ItemSelected(i.value()))
        });

        let mut wrapper_classes = classes!("dropdown-wrapper");
        if self.is_open {
            wrapper_classes.push("open")
        }

        html! {
            <div class={wrapper_classes}>
                <div class="clicker" onclick={ link.callback(|_| ItemSelectDropdownMessage::CloseDropdown) }></div>
                // FIXME: sends 2 messages to open dropdown instead of selecting a label
                <div class="item-select-dropdown" onclick={link.callback(|_| ItemSelectDropdownMessage::OpenDropdown)}>
                    {
                        for GAME_DATA.items_in_groups().iter().enumerate().map(|(i_1, (_group_name, group))| {
                            html_nested! {
                                <>
                                {for group.iter().enumerate().map(|(i_2, (_subgroup_name, subgroup))| {
                                    html_nested!{
                                        <>
                                        {for subgroup.iter().enumerate().map(|(i_3, item)| {
                                            let input_id = format!("input-{}-{}-{}-{}", props.index, i_1, i_2, i_3);
                                            html_nested! {
                                                <span>
                                                <input
                                                    type="radio"
                                                    value={item.name.clone()}
                                                    onchange={on_item_selected.clone()}
                                                    name={format!("item-select-{}", props.index)}
                                                    checked={ item.name == self.selected_item }
                                                    id={input_id.clone()}
                                                />
                                                <label for={input_id}> <ItemIcon item={item.name.clone()} /> </label>
                                                </span>
                                            }
                                        })}
                                        <br/>
                                        </>
                                    }
                                })}
                                <hr/>
                                </>
                            }
                        })
                    }
                </div>
                <div class="spacer"></div>
            </div>
        }
    }
}
