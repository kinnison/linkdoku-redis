use linkdoku_common::LoginFlowResult;
use reqwest::Url;
use serde::Deserialize;
use yew::prelude::*;
use yew::{function_component, html};
use yew_router::prelude::*;

mod components;

use components::core::{BaseURIProvider, Footer, Navbar};
use components::login::{LoginStatus, UserProvider};

use crate::components::core::use_api_url;
use crate::components::login::{LoginStatusAction, LoginStatusDispatcher};

#[derive(Routable, PartialEq, Clone)]
enum Route {
    #[at("/-/")]
    Root,
    #[at("/-/complete-login")]
    CompleteLogin,
    #[not_found]
    #[at("/-/404")]
    NotFound,
}

#[function_component(Root)]
fn app_root() -> Html {
    html! {
        <BaseURIProvider>
            <UserProvider>
                <BrowserRouter>
                    <Navbar />
                    <Switch<Route> render={Switch::render(switch)} />
                    <Footer />
                </BrowserRouter>
            </UserProvider>
        </BaseURIProvider>
    }
}

#[derive(Deserialize)]
struct FlowContinuation {
    state: String,
    error: Option<String>,
    code: Option<String>,
}

#[function_component(HandleLoginFlow)]
fn login_flow() -> Html {
    let history = use_history().expect("Not able to get history object");
    let location = use_location().expect("Not able to get router location");
    let query: FlowContinuation = location.query().expect("Not able to get query string");

    let login_status = use_context::<LoginStatus>().expect("No login status?");
    let login_status_dispatch =
        use_context::<LoginStatusDispatcher>().expect("No login status dispatcher?");

    if login_status == LoginStatus::Unknown {
        return html! {
            "Please wait, loading site"
        };
    }

    if query.code.is_none() || query.error.is_some() {
        // We had an error, so we should clear things and return to root
        let clear_url = use_api_url("/login/clear");
        use_effect(move || {
            wasm_bindgen_futures::spawn_local(async move {
                reqwest::get(clear_url).await.expect("Should be fine!");
                history.push(Route::Root);
            });
            || ()
        });
        return html! {
            "We had a problem, please hold..."
        };
    }

    let continuation_url = Url::parse_with_params(
        use_api_url("/login/continue").as_str(),
        &[
            ("state", query.state.clone()),
            ("code", query.code.as_ref().unwrap().to_string()),
        ],
    )
    .expect("Unable to construct continuation URL");

    gloo::console::log!("Setting up a callback for continuation");
    let login_status_url = use_api_url("/login/status");
    use_effect_with_deps(
        {
            move |(continuation_url, dispatcher): &(Url, LoginStatusDispatcher)| {
                let continuation_url = continuation_url.clone();
                let dispatcher = dispatcher.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    gloo::console::log!("Running continuation callback");
                    let result: LoginFlowResult = reqwest::get(continuation_url)
                        .await
                        .expect("Unable to fetch API call")
                        .json()
                        .await
                        .expect("Unable to unpack json");
                    if let Some(_error) = result.error {
                        // Error while trying to retrieve login results, so say we're logged out
                        dispatcher.dispatch(LoginStatusAction::LoggedOut);
                    } else {
                        // Success, so retrieve the login status info
                        let status: linkdoku_common::LoginStatus = reqwest::get(login_status_url)
                            .await
                            .expect("Unable to do API call")
                            .json()
                            .await
                            .expect("Unable to unpack json");
                        if let (Some(name), email) = (status.display_name, status.email_address) {
                            dispatcher.dispatch(LoginStatusAction::LoggedIn(name, email));
                        } else {
                            dispatcher.dispatch(LoginStatusAction::LoggedOut);
                        }
                    }
                    history.push(Route::Root);
                });
                || ()
            }
        },
        (continuation_url, login_status_dispatch),
    );

    html! {
        "Handling login, please hold..."
    }
}

#[function_component(ShowNotFound)]
fn show_not_found() -> Html {
    let location = use_location().unwrap();
    html! {
        <>
            {"I have no idea what you mean by:"}
            <tt>
                {location.pathname()}
            </tt>
        </>
    }
}

fn switch(route: &Route) -> Html {
    match route {
        Route::Root => html! { <LoginStateShow /> },
        Route::CompleteLogin => html! { <HandleLoginFlow /> },
        Route::NotFound => html! { <ShowNotFound /> },
    }
}

#[function_component(LoginStateShow)]
fn show_login_state() -> Html {
    let login_status = use_context::<LoginStatus>().expect("Cannot get login status");
    match login_status {
        LoginStatus::Unknown => html! {},
        LoginStatus::LoggedOut => {
            html! {
                <div> {"You are not logged in!"}
                </div>
            }
        }
        LoginStatus::LoggedIn { name, email } => {
            html! {
                <div>
                    {format!("Your name is: {}", name)}
                    <br />
                    {if let Some(addr) = email.as_ref() {
                        html!{
                            <>
                            {format!("Your email is: {}", addr)}
                            <br />
                            </>
                        }
                    } else {
                        html! {}
                    }}
                </div>
            }
        }
    }
}

fn main() {
    yew::start_app::<Root>();
}
