use linkdoku_common::{LoginFlowStart, LoginStatus};
use reqwest::{StatusCode, Url};
use serde::Deserialize;
use yew::prelude::*;
use yew::{function_component, html};
use yew_router::prelude::*;

#[derive(Debug, Clone, PartialEq)]
struct AppGlobals {
    first_load: bool,
    login_status: LoginStatus,
    update_login_status: Callback<()>,
}

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
    let ctx = use_state(|| LoginStatus {
        display_name: None,
        email_address: None,
    });
    let first_load = use_state(|| true);
    let globals = AppGlobals {
        first_load: *first_load,
        login_status: (*ctx).clone(),
        update_login_status: {
            let ctx = ctx.clone();
            Callback::from(move |_| {
                let ctx = ctx.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    let status: LoginStatus =
                        reqwest::get("http://localhost:3000/api/login/status")
                            .await
                            .unwrap()
                            .json()
                            .await
                            .unwrap();
                    ctx.setter().set(status);
                });
            })
        },
    };

    if globals.first_load {
        let emitter = globals.update_login_status.clone();
        use_effect(move || {
            first_load.setter().set(false);
            emitter.emit(());
            || ()
        });
    }

    html! {
       <ContextProvider<AppGlobals> context={globals}>
          <BrowserRouter>
            <Switch<Route> render={Switch::render(switch)} />
          </BrowserRouter>
       </ContextProvider<AppGlobals>>
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

    let globals = use_context::<AppGlobals>().expect("No global context found");

    if globals.first_load {
        return html! {
            "Please wait, loading site"
        };
    }

    if query.code.is_none() || query.error.is_some() {
        // We had an error, so we should clear things and return to root
        use_effect(move || {
            wasm_bindgen_futures::spawn_local(async move {
                reqwest::get("http://localhost:3000/api/login/clear")
                    .await
                    .expect("Should be fine!");
                history.push(Route::Root);
            });
            || ()
        });
        return html! {
            "We had a problem, please hold..."
        };
    }

    let continuation_url = Url::parse_with_params(
        "http://localhost:3000/api/login/continue",
        &[
            ("state", query.state.clone()),
            ("code", query.code.as_ref().unwrap().to_string()),
        ],
    )
    .expect("Unable to construct continuation URL");

    gloo::console::log!("Setting up a callback for continuation");
    use_effect_with_deps(
        {
            move |continuation_url: &Url| {
                let continuation_url = continuation_url.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    gloo::console::log!("Running continuation callback");
                    reqwest::get(continuation_url)
                        .await
                        .expect("Unable to fetch API call");
                    gloo::console::log!("Navigating to root");
                    history.push(Route::Root);
                    globals.update_login_status.emit(());
                });
                || ()
            }
        },
        continuation_url,
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

#[function_component(LoginButton)]
fn login_button() -> Html {
    let history = use_history().unwrap();
    let login_click = Callback::from(move |_| {
        // User clicked login, so we need to redirect the user to the login flow
        // startup
        let history = history.clone();
        wasm_bindgen_futures::spawn_local(async move {
            let response = reqwest::get("http://localhost:3000/api/login/start/google")
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
        <button onclick={login_click}>
            {"Start login"}
        </button>
    }
}

#[function_component(LogoutButton)]
fn logout_button() -> Html {
    let globals = use_context::<AppGlobals>().expect("No global context found");
    let history = use_history().unwrap();
    let logout_click = Callback::from(move |_| {
        let history = history.clone();
        let emitter = globals.update_login_status.clone();
        wasm_bindgen_futures::spawn_local(async move {
            reqwest::get("http://localhost:3000/api/login/clear")
                .await
                .unwrap();
            history.push(Route::Root);
            emitter.emit(());
        });
    });
    html! {
        <button onclick={logout_click}>
            {"Log out"}
        </button>
    }
}

#[function_component(LoginStateShow)]
fn show_login_state() -> Html {
    let globals = use_context::<AppGlobals>().expect("No global context found");

    if let Some(name) = globals.login_status.display_name.as_ref() {
        html! {
            <div>
                {format!("Your name is: {}", name)}
                <br />
                {if let Some(addr) = globals.login_status.email_address.as_ref() {
                    html!{
                        <>
                        {format!("Your email is: {}", addr)}
                        <br />
                        </>
                    }
                } else {
                    html! {}
                }}
                <LogoutButton />
            </div>
        }
    } else {
        html! {
            <div> {"You are not logged in!"}
            <br />
            <LoginButton />
            </div>
        }
    }
}

fn main() {
    yew::start_app::<Root>();
}
