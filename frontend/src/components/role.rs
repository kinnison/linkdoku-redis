use yew::prelude::*;

use crate::utils::cache::{CacheEntry, ObjectCache};

#[derive(Properties, PartialEq)]
pub struct RoleProps {
    pub uuid: String,
    pub onclick: Option<Callback<()>>,
}

#[function_component(Role)]
pub fn render_role(props: &RoleProps) -> Html {
    let cache = use_context::<ObjectCache>().expect("No cache?");
    let role_data = cache.cached_role(&props.uuid);

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

    let role_body = html! {
        <span class={"icon-text"}>
            <span class={"icon"}><i class={"fa-regular fa-circle-user"}/></span>
            {role_body}
        </span>
    };

    if let Some(cb) = &props.onclick {
        // We have an onclick, so use it
        let cb = cb.clone();
        let onclick = Callback::from(move |_| cb.emit(()));
        html! {
            <div class={"block"} onclick={onclick}>
                {role_body}
            </div>
        }
    } else {
        // No onclick, so just show
        html! {
            <div class={"block"}>
                {role_body}
            </div>
        }
    }
}
