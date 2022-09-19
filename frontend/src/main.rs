use linkdoku_common::{BackendLoginStatus, LoginFlowResult};
use reqwest::Url;
use serde::Deserialize;
use serde_json::Value;
use yew::prelude::*;
use yew::{function_component, html};
use yew_router::prelude::*;
use yew_toastrack::{Toast, ToastContainer, ToastLevel, Toaster};

mod components;

mod utils;

use utils::cache::ObjectCacheProvider;

use components::core::{BaseURIProvider, Footer, Navbar};
use components::login::{LoginStatus, UserProvider};

use crate::components::core::use_api_url;
use crate::components::login::{LoginStatusAction, LoginStatusDispatcher};
use crate::components::role::Role;

use yew_markdown::editor::MarkdownEditor;

#[derive(Routable, PartialEq, Clone)]
enum Route {
    #[at("/-/")]
    Root,
    #[at("/-/complete-login")]
    CompleteLogin,
    #[at("/-/utils/lz")]
    LZPage,
    #[not_found]
    #[at("/-/404")]
    NotFound,
}

#[function_component(Root)]
fn app_root() -> Html {
    html! {
        <BaseURIProvider>
            <ObjectCacheProvider>
                <UserProvider>
                    <ToastContainer />
                    <BrowserRouter>
                        <Navbar />
                        <Switch<Route> render={Switch::render(switch)} />
                        <Footer />
                    </BrowserRouter>
                </UserProvider>
            </ObjectCacheProvider>
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
                        let status: BackendLoginStatus = reqwest::get(login_status_url)
                            .await
                            .expect("Unable to do API call")
                            .json()
                            .await
                            .expect("Unable to unpack json");
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
        Route::LZPage => html! { <LZPage /> },
    }
}

#[function_component(LoginStateShow)]
fn show_login_state() -> Html {
    let login_status = use_context::<LoginStatus>().expect("Cannot get login status");

    let counter = use_state(|| 0usize);

    let toasty = Callback::from({
        move |_| {
            Toaster::toast(
                Toast::new(&format!("Hello world ({})", *counter))
                    .with_lifetime(Some(5000))
                    .with_level(ToastLevel::Success),
            );
            counter.set(*counter + 1);
        }
    });

    let utility = Callback::from({
        let history = use_history().expect("What, no history?");
        move |_| {
            history.push(Route::LZPage);
        }
    });

    const MARKDOWN: &str = r#"
# Hello World

This is a paragraph of text as a markdown object.
It will be interesting to see what the everything looks like.

| What | Are | You | Up | To |
| :---- | :--- | ---: | :--: | :-- |
| I'm  | Not | Sure | Truly | Sorry |

Are you interested in [this link](https://example.com/)?
Perhaps you want to try a [reference link][rl] instead?


[rl]: https://www.digital-scurf.org/

    "#;

    match login_status {
        LoginStatus::Unknown => html! {},
        LoginStatus::LoggedOut => {
            html! {
                <div> {"You are not logged in!"}
                </div>
            }
        }
        LoginStatus::LoggedIn { name, role, .. } => {
            html! {
                <div>
                    {format!("Your name is: {}", name)}
                    <br />
                    <Role uuid={role.clone()} />
                    <br />
                    <button class={"button is-danger"} onclick={toasty}>{"Say hello"}</button>
                    <br />
                    <button class={"button is-primary"} onclick={utility}>{"LZ Utility"}</button>
                    <br />
                    <MarkdownEditor name={"markdown"} initial={MARKDOWN} />
                </div>
            }
        }
    }
}

#[function_component(LZPage)]
fn show_lz_page() -> Html {
    use web_sys::{HtmlInputElement, HtmlTextAreaElement};

    let lz_input = use_node_ref();
    let textarea = use_node_ref();

    const DICTIONARY: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/\\";

    let decompress_action = {
        let lz_input = lz_input.clone();
        let textarea = textarea.clone();
        Callback::from(move |_| {
            if let (Some(input), Some(textarea)) = (
                lz_input.cast::<HtmlInputElement>(),
                textarea.cast::<HtmlTextAreaElement>(),
            ) {
                let input_str = input.value();
                let nums: Vec<_> = input_str
                    .bytes()
                    .flat_map(|b| {
                        DICTIONARY
                            .iter()
                            .enumerate()
                            .find(|&v| *v.1 == b)
                            .map(|v| v.0 as u32)
                    })
                    .collect();
                if let Some(decomp) = lz_str::decompress(&nums, 6) {
                    match serde_json::from_str::<Value>(&decomp) {
                        Ok(v) => {
                            let s =
                                serde_json::to_string_pretty(&v).expect("Can't re-serialise JSON");
                            textarea.set_value(&s);
                        }
                        Err(e) => {
                            textarea.set_value(&format!(
                                "Unable to read compressed JSON: {:?}\n\n'{}'",
                                e, decomp
                            ));
                        }
                    }
                } else {
                    textarea.set_value(&format!("Unable to decompress: {}", input.value()));
                }
            }
        })
    };

    let compress_action = {
        let lz_input = lz_input.clone();
        let textarea = textarea.clone();
        Callback::from(move |_| {
            if let (Some(input), Some(textarea)) = (
                lz_input.cast::<HtmlInputElement>(),
                textarea.cast::<HtmlTextAreaElement>(),
            ) {
                match serde_json::from_str::<Value>(&textarea.value()) {
                    Ok(v) => {
                        let squished = serde_json::to_string(&v).unwrap();
                        let compressed = lz_str::compress(&squished, 6, |v| {
                            *DICTIONARY.get(v as usize).unwrap() as u32
                        });
                        let nums: String =
                            compressed.into_iter().map(|v| v as u8 as char).collect();
                        input.set_value(&nums);
                    }
                    Err(e) => {
                        input.set_value(&format!("Unable to parse JSON: {:?}", e));
                    }
                }
            }
        })
    };

    html! {
        <div class={"section"}>
            <div class={"field"}>
                <label class={"label"}>{"Compressed puzzle"}</label>
                <div class={"control"}>
                    <input class={"input"} type={"text"} placeholder={"f-puzzles style compressed input"} ref={lz_input}/>
                </div>
            </div>
            <div class={"field"}>
                <label class={"label"}>{"Uncompressed puzzle"}</label>
                <div class={"control"}>
                    <textarea class={"textarea"} placeholder={"uncompressed json output"} ref={textarea}></textarea>
                </div>
            </div>
            <div class={"buttons"}>
                <button class={"button"} onclick={decompress_action}>{"Decompress"}</button>
                <button class={"button"} onclick={compress_action}>{"Compress"}</button>
            </div>
        </div>
    }
}

fn main() {
    yew::start_app::<Root>();
}
