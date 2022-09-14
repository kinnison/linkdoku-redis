use stylist::yew::{styled_component, use_style};
use yew::prelude::*;

use crate::utils::cache::{CacheEntry, ObjectCache};

#[derive(Properties, PartialEq)]
pub struct RoleProps {
    pub uuid: String,
    pub active: Option<bool>,
    pub onclick: Option<Callback<()>>,
}

#[styled_component(Role)]
pub fn render_role(props: &RoleProps) -> Html {
    let role_style = use_style!(
        r#"
        display: inline-block;
        width: 250px;
        overflow-x: hidden;
        "#
    );
    let cache = use_context::<ObjectCache>().expect("No cache?");
    let role_data = cache.cached_role(&props.uuid);

    let active = props.active.unwrap_or(false);

    let role_body = match &*role_data {
        CacheEntry::Pending => {
            // Do the "loading animation"
            html! {}
        }
        CacheEntry::Missing => {
            // The role is missing, so the role content will be the role's UUID
            html! {
                <span>{props.uuid.clone()}</span>
            }
        }
        CacheEntry::Value(role) => {
            // The role is available, so use it
            html! {
                <span>{role.display_name.clone()}</span>
            }
        }
    };

    let icon_class = if active {
        classes! {"fa-solid", "fa-circle-user"}
    } else {
        classes! {"fa-regular", "fa-circle-user"}
    };

    let role_body = html! {
        <span class={"icon-text"}>
            <span class={"icon"}><i class={icon_class}/></span>
            {role_body}
        </span>
    };

    let role_classes = {
        let mut ret = if active {
            classes!(
                "box",
                "has-background-primary-light",
                "is-shadowless",
                "p-1"
            )
        } else {
            classes!("block")
        };
        ret.push(role_style);
        if props.onclick.is_some() {
            ret.push(classes! {"is-clickable"});
        }
        ret
    };

    if let Some(cb) = &props.onclick {
        // We have an onclick, so use it
        let cb = cb.clone();
        let onclick = Callback::from(move |_| cb.emit(()));
        html! {
            <div class={role_classes} onclick={onclick}>
                {role_body}
            </div>
        }
    } else {
        // No onclick, so just show
        html! {
            <div class={role_classes}>
                {role_body}
            </div>
        }
    }
}
