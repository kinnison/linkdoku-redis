//! Everything to do with logging a user in, providing the logged in user as a value, etc.

use std::rc::Rc;

use linkdoku_common::BackendLoginStatus;
use linkdoku_common::LoginFlowStart;
use yew::prelude::*;
use yew::Reducible;
use yew_router::prelude::*;

use crate::Route;

use super::core::make_api_call;
use super::core::use_api_url;
use super::core::ReqwestClient;
use super::core::NO_BODY;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoginStatus {
    Unknown,
    LoggedOut,
    LoggedIn {
        name: String,
        gravatar_hash: Option<String>,
        roles: Vec<String>,
        role: String,
    },
}

impl LoginStatus {
    fn choose_role(&self, role: String) -> Self {
        match self {
            Self::Unknown => Self::Unknown,
            Self::LoggedOut => Self::LoggedOut,
            Self::LoggedIn {
                name,
                gravatar_hash,
                roles,
                ..
            } => Self::LoggedIn {
                name: name.clone(),
                gravatar_hash: gravatar_hash.clone(),
                roles: roles.clone(),
                role,
            },
        }
    }

    pub fn is_unknown(&self) -> bool {
        matches! {self, Self::Unknown}
    }
    pub fn is_logged_in(&self) -> bool {
        matches! {self, Self::LoggedIn{..}}
    }

    pub fn roles(&self) -> &[String] {
        match self {
            Self::LoggedIn { roles, .. } => roles,
            _ => &[],
        }
    }

    pub fn current_role(&self) -> Option<&str> {
        match self {
            Self::LoggedIn { role, .. } => Some(role.as_str()),
            _ => None,
        }
    }
}

pub enum LoginStatusAction {
    LoggedOut,
    LoggedIn(String, Option<String>, Vec<String>, String),
    ChosenRole(String),
}

impl Reducible for LoginStatus {
    type Action = LoginStatusAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        match action {
            LoginStatusAction::LoggedOut => LoginStatus::LoggedOut,
            LoginStatusAction::LoggedIn(name, gravatar_hash, roles, role) => {
                LoginStatus::LoggedIn {
                    name,
                    gravatar_hash,
                    roles,
                    role,
                }
            }
            LoginStatusAction::ChosenRole(role) => self.choose_role(role),
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
    let client = use_context::<ReqwestClient>().expect("No API client");

    // First time out of the gate, acquire the status
    if *state == LoginStatus::Unknown {
        use_effect({
            let dispatcher = state.dispatcher();
            || {
                wasm_bindgen_futures::spawn_local(async move {
                    let status: BackendLoginStatus =
                        make_api_call(client, status_url.as_str(), None, NO_BODY)
                            .await
                            .expect("Unable to make API call");
                    match status {
                        BackendLoginStatus::LoggedOut => {
                            dispatcher.dispatch(LoginStatusAction::LoggedOut);
                        }
                        BackendLoginStatus::LoggedIn {
                            name,
                            gravatar_hash,
                            roles,
                            role,
                        } => {
                            dispatcher.dispatch(LoginStatusAction::LoggedIn(
                                name,
                                gravatar_hash,
                                roles,
                                role,
                            ));
                        }
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
    let client = use_context::<ReqwestClient>().expect("No API client");
    let start_google = use_api_url("/login/start/google");
    let login_click = Callback::from(move |_| {
        // User clicked login, so we need to redirect the user to the login flow
        // startup
        let history = history.clone();
        let start_google = start_google.clone();
        let client = client.clone();
        wasm_bindgen_futures::spawn_local(async move {
            let res: LoginFlowStart = make_api_call(client, start_google.as_str(), None, NO_BODY)
                .await
                .expect("Unable to start login");
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
