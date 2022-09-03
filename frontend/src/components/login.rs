//! Everything to do with logging a user in, providing the logged in user as a value, etc.

use std::rc::Rc;

use linkdoku_common::LoginFlowStart;
use reqwest::StatusCode;
use yew::prelude::*;
use yew::Reducible;
use yew_router::prelude::*;

use crate::Route;

use super::core::use_api_url;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoginStatus {
    Unknown,
    LoggedOut,
    LoggedIn { name: String, email: Option<String> },
}

pub enum LoginStatusAction {
    LoggedOut,
    LoggedIn(String, Option<String>),
}

impl Reducible for LoginStatus {
    type Action = LoginStatusAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        match action {
            LoginStatusAction::LoggedOut => LoginStatus::LoggedOut,
            LoginStatusAction::LoggedIn(name, email) => LoginStatus::LoggedIn { name, email },
        }
        .into()
    }
}

#[derive(Properties, Clone, PartialEq)]
pub struct UserProviderProps {
    pub children: Children,
}

pub type LoginStatusDispatcher = UseReducerDispatcher<LoginStatus>;

#[function_component(UserProvider)]
pub fn login_user_provider(props: &UserProviderProps) -> Html {
    let state = use_reducer_eq(|| LoginStatus::Unknown);
    let status_url = use_api_url("/login/status");

    // First time out of the gate, acquire the status
    if *state == LoginStatus::Unknown {
        use_effect({
            let dispatcher = state.dispatcher();
            || {
                wasm_bindgen_futures::spawn_local(async move {
                    let status: linkdoku_common::LoginStatus = reqwest::get(status_url)
                        .await
                        .unwrap()
                        .json()
                        .await
                        .unwrap();
                    if let (Some(display_name), email) = (status.display_name, status.email_address)
                    {
                        dispatcher.dispatch(LoginStatusAction::LoggedIn(display_name, email))
                    } else {
                        dispatcher.dispatch(LoginStatusAction::LoggedOut)
                    }
                });
                // No destructor
                || ()
            }
        });
    }

    html! {
        <ContextProvider<LoginStatus> context={(*state).clone()}>
            <ContextProvider<LoginStatusDispatcher> context={state.dispatcher()}>
                {props.children.clone()}
            </ContextProvider<LoginStatusDispatcher>>
        </ContextProvider<LoginStatus>>
    }
}

#[function_component(LoginButton)]
pub fn login_button() -> Html {
    let history = use_history().unwrap();
    let start_google = use_api_url("/login/start/google");
    let login_click = Callback::from(move |_| {
        // User clicked login, so we need to redirect the user to the login flow
        // startup
        let history = history.clone();
        let start_google = start_google.clone();
        wasm_bindgen_futures::spawn_local(async move {
            let response = reqwest::get(start_google)
                .await
                .expect("Unable to make call");
            if response.status() != StatusCode::OK {
                // some kind of error
            } else {
                let res: LoginFlowStart = response.json().await.unwrap();
                match res {
                    LoginFlowStart::Idle => history.push(Route::Root),
                    LoginFlowStart::Redirect(url) => {
                        gloo::utils::window().location().set_href(&url).unwrap();
                    }
                    LoginFlowStart::Error(err) => {
                        gloo::console::log!("Failure doing login? {}", err);
                        history.push(Route::Root);
                    }
                }
            }
        });
    });

    html! {
        <button class={"button is-primary"} onclick={login_click}>
            {"Login with Google"}
        </button>
    }
}

#[function_component(LogoutButton)]
pub fn logout_button() -> Html {
    let login_status_dispatch =
        use_context::<LoginStatusDispatcher>().expect("Cannot get login status dispatcher");
    let history = use_history().unwrap();
    let clear_login = use_api_url("/login/clear");
    let logout_click = Callback::from(move |_| {
        let history = history.clone();
        let login_status_dispatch = login_status_dispatch.clone();
        let clear_login = clear_login.clone();
        wasm_bindgen_futures::spawn_local(async move {
            reqwest::get(clear_login).await.unwrap();
            history.push(Route::Root);
            login_status_dispatch.dispatch(LoginStatusAction::LoggedOut);
        });
    });
    html! {
        <button class={"button is-danger"} onclick={logout_click}>
            {"Log out"}
        </button>
    }
}
