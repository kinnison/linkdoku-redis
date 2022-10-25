//! A tabbed control using Bulma classes for Yew
//!

use std::rc::Rc;

use stylist::yew::styled_component;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct TabContentProps {
    pub title: String,
    pub children: Children,
}

#[styled_component(TabContent)]
pub fn tab_content(_props: &TabContentProps) -> Html {
    html! {}
}

#[derive(Properties, PartialEq)]
pub struct TabbedProps {
    pub default: String,
    pub tabchanged: Option<Callback<String>>,
    pub children: ChildrenWithProps<TabContent>,
}

#[styled_component(Tabbed)]
pub fn tabbed(props: &TabbedProps) -> Html {
    let tabs: Vec<_> = props
        .children
        .iter()
        .map(|tab| Rc::clone(&tab.props))
        .collect();
    let current_tab = use_state_eq(|| {
        tabs.iter()
            .enumerate()
            .find(|&(_, tab)| tab.title == props.default)
            .map(|(idx, _)| idx)
            .unwrap_or(0)
    });

    use_effect_with_deps(
        {
            let tabs = tabs.clone();
            let tabchanged = props.tabchanged.clone();
            move |tabidx: &usize| {
                if let Some(cb) = tabchanged.as_ref() {
                    cb.emit(tabs[*tabidx].title.clone());
                }
                || ()
            }
        },
        *current_tab,
    );

    let outer_style = classes!("mx-2", "block");

    let current_idx = *current_tab;

    let tab_bodies: Vec<Html> = props
        .children
        .iter()
        .enumerate()
        .map(|(idx, tab)| {
            html! {
                <TabInner visible={idx==current_idx}>
                    {tab.props.children.clone()}
                </TabInner>
            }
        })
        .collect();

    let tablist = tabs
        .into_iter()
        .enumerate()
        .map(|(idx, tprops)| {
            let current_tab = current_tab.setter();
            let set_tab = Callback::from(move |_| {
                current_tab.set(idx);
            });
            if idx == current_idx {
                html! {
                    <li class={"is-active"}>
                        <a>{tprops.title.clone()}</a>
                    </li>
                }
            } else {
                html! {
                    <li onclick={set_tab}>
                        <a>{tprops.title.clone()}</a>
                    </li>
                }
            }
        })
        .collect::<Html>();

    html! {
        <div class={outer_style}>
            <div class={"tabs is-boxed"}>
                <ul>
                    {tablist}
                </ul>
            </div>
            {tab_bodies}
        </div>
    }
}

#[derive(Properties, PartialEq)]
struct TabInnerProps {
    pub visible: bool,
    children: Children,
}

#[function_component(TabInner)]
fn tab_inner(props: &TabInnerProps) -> Html {
    html! {
        <div class={if props.visible { "is-block" } else { "is-hidden" }}>
            {props.children.clone()}
        </div>
    }
}
