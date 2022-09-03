//! Everything to do with logging a user in, providing the logged in user as a value, etc.

use std::rc::Rc;

use yew::prelude::*;
use yew::Reducible;

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

#[function_component(LogInOutButton)]
pub fn login_inout_button() -> Html {
    todo!()
}
