//! Utility components
//!

use yew::prelude::*;
use yew_hooks::prelude::*;
use yew_toastrack::*;

#[derive(Properties, PartialEq, Eq)]
pub struct CopyButtonProps {
    pub content: String,
    pub icon: Option<String>,
    pub toast: Option<Toast>,
}

#[function_component(CopyButton)]
pub fn copy_button(props: &CopyButtonProps) -> Html {
    let clipboard = use_clipboard();

    let icon_class = classes!(
        "fas",
        format!("fa-{}", props.icon.as_deref().unwrap_or("copy"))
    );

    let onclick = Callback::from({
        let toast = props.toast.as_ref().map(Toast::clone).unwrap_or_else(|| {
            Toast::new("Copied")
                .with_lifetime(Some(1000))
                .with_level(ToastLevel::Success)
        });
        let content = props.content.clone();
        move |_| {
            Toaster::toast(toast.clone());
            clipboard.write_text(content.clone());
        }
    });

    html! {
        <span onclick={onclick} class={"icon"}>
            <i class={icon_class} />
        </span>
    }
}

#[derive(Properties, PartialEq)]
pub struct TooltipProps {
    pub content: String,
    #[prop_or(TooltipAlignment::Top)]
    pub alignment: TooltipAlignment,
    #[prop_or(true)]
    pub arrow: bool,
    #[prop_or(TooltipLevel::Normal)]
    pub level: TooltipLevel,
    #[prop_or(false)]
    pub multiline: bool,
    #[prop_or(false)]
    pub active: bool,
    #[prop_or(TooltipTextAlignment::Default)]
    pub textalign: TooltipTextAlignment,
    #[prop_or(false)]
    pub block: bool,
    pub children: Children,
}

#[allow(dead_code)]
#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub enum TooltipTextAlignment {
    Default,
    Left,
    Centered,
    Right,
}

#[allow(dead_code)]
#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub enum TooltipAlignment {
    Default,
    Top,
    Bottom,
    Left,
    Right,
}

#[allow(dead_code)]
#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub enum TooltipLevel {
    Normal,
    Info,
    Warning,
    Primary,
    Success,
    Danger,
}

#[function_component(Tooltip)]
pub fn tooltip_component(props: &TooltipProps) -> Html {
    let classes = classes! {
        match props.alignment {
            TooltipAlignment::Default => None,
            TooltipAlignment::Top => Some("has-tooltip-top"),
            TooltipAlignment::Bottom => Some("has-tooltip-bottom"),
            TooltipAlignment::Left => Some("has-tooltip-left"),
            TooltipAlignment::Right => Some("has-tooltip-right"),
        },
        if props.arrow {
            Some("has-tooltip-arrow")
        } else {
            None
        },
        match props.level {
            TooltipLevel::Normal => None,
            TooltipLevel::Info => Some("has-tooltip-info"),
            TooltipLevel::Warning => Some("has-tooltip-warning"),
            TooltipLevel::Primary => Some("has-tooltip-primary"),
            TooltipLevel::Success => Some("has-tooltip-success"),
            TooltipLevel::Danger => Some("has-tooltip-danger"),
        },
        match props.textalign {
            TooltipTextAlignment::Default => None,
            TooltipTextAlignment::Left => Some("has-tooltip-text-left"),
            TooltipTextAlignment::Centered => Some("has-tooltip-text-centered"),
            TooltipTextAlignment::Right => Some("has-tooltip-text-right"),
        },
        if props.multiline {
            Some("has-tooltip-multiline")
        } else {
            None
        },
        if props.active {
            Some("has-tooltip-active")
        } else {
            None
        },
    };

    if props.block {
        html! {
            <div class={classes} data-tooltip={props.content.clone()}>
                {props.children.clone()}
            </div>
        }
    } else {
        html! {
            <span class={classes} data-tooltip={props.content.clone()}>
                {props.children.clone()}
            </span>
        }
    }
}
