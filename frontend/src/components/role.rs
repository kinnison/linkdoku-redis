use stylist::yew::{styled_component, use_style};
use yew::prelude::*;
use yew_hooks::use_title;
use yew_markdown::render::MarkdownRender;
use yew_router::prelude::{use_history, History, Location};
use yew_toastrack::{Toast, ToastLevel, Toaster};

use crate::{
    components::puzzle::CreatePuzzleState,
    utils::cache::{CacheEntry, ObjectCache},
    Route,
};

use super::login::LoginStatus;

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
        CacheEntry::Missing | CacheEntry::Error(_) => {
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

#[function_component(DefaultRoleRedirect)]
pub fn default_role_redirect() -> Html {
    let login_stats = use_context::<LoginStatus>().expect("No login status?");
    let history = use_history().expect("No browser history?");
    gloo::console::log!(format!("Role redirect, login_stats={:?}", login_stats));

    use_effect_with_deps(
        move |stats| {
            gloo::console::log!(format!("in use_effect, login_stats={:?}", stats));
            match stats {
                LoginStatus::Unknown => (),
                LoginStatus::LoggedOut => {
                    history.push(Route::Root);
                }
                LoginStatus::LoggedIn { role, .. } => {
                    history.push(Route::RolePage { role: role.clone() });
                }
            };
            || ()
        },
        login_stats,
    );

    html! {}
}

#[derive(Properties, PartialEq, Eq, Debug)]
pub struct RolePageProps {
    pub role: String,
}

#[function_component(RolePage)]
pub fn role_page(props: &RolePageProps) -> Html {
    let cache = use_context::<ObjectCache>().expect("No cache?");
    let role_data = cache.cached_role(&props.role);
    let history = use_history().expect("No history?");

    gloo::console::log!(format!("Role Page: role={:?}", &*role_data));

    if role_data.is_pending() {
        // Eventually render a page spinner
        return html! {};
    }

    if role_data.is_missing() {
        Toaster::toast(
            Toast::new(&format!("Role {} was not found", props.role))
                .with_lifetime(Some(5000))
                .with_level(ToastLevel::Danger),
        );
        history.push(Route::Root);
        return html! {};
    }

    if role_data.is_error() {
        Toaster::toast(
            Toast::new(&format!(
                "Error fetching role {}: {}",
                props.role,
                role_data.error_text()
            ))
            .with_lifetime(Some(5000))
            .with_level(ToastLevel::Danger),
        );
        history.push(Route::Root);
        return html! {};
    }

    // We have the role data, so let's render it
    let role_data = role_data.value().unwrap().clone();

    // if it turns out we were invoked by UUID, redirect to short-name because it's nicer for copy/pasta
    if props.role == role_data.uuid {
        // check if the current history value shows the current role by uuid too
        if let Some(Route::RolePage { role }) = history.location().route::<Route>() {
            gloo::console::log!(format!(
                "role == {}, uuid == {}, route_role == {}",
                props.role, role_data.uuid, role
            ));
            if role == props.role {
                // Still showing UUID, so replace in the URL
                history.replace(Route::RolePage {
                    role: role_data.short_name.clone(),
                });
            }
        }
    }

    use_title(format!("Linkdoku - Role - {}", role_data.display_name));

    let create_puzzle_click = Callback::from(move |_| {
        history
            .push_with_state(
                Route::CreatePuzzle,
                CreatePuzzleState {
                    owner: role_data.uuid.clone(),
                    ..Default::default()
                },
            )
            .expect("Unable to push create puzzle state")
    });

    html! {
        <>
            <h1 class={"title is-1"}>{role_data.display_name.clone()}</h1>
            <MarkdownRender markdown={role_data.bio} />
            <hr />
            <h2 class={"title is-2"}>{"No puzzle list renderer yet"}</h2>
            <hr />
            <div class={"level is-mobile"}>
                <div class={"level-left"} />
                <div class={"level-right"}>
                    <div class={"level-item"}>
                        <button class={"button is-primary"} onclick={create_puzzle_click}>{"Create puzzle"}</button>
                    </div>
                </div>
            </div>
        </>
    }
}
