//! Core components for linkdoku
//!
//!

use std::sync::Arc;

use crate::Route;

use crate::components::user::UserMenuNavbarItem;

use reqwest::{Client, Url};
use serde::{de::DeserializeOwned, Serialize};
use thiserror::Error;
use wasm_bindgen::JsCast;
use yew::prelude::*;
use yew_router::prelude::*;

#[function_component(Footer)]
pub fn core_page_footer() -> Html {
    html! {
        <footer class={"footer"}>
            <div class="content has-text-centered">
                <p>
                    <strong>{"Linkdoku"}</strong> {" by "} <a href="https://github.com/kinnison">{"Daniel Silverstone"}</a>{". "}
                    <a href="https://github.com/kinnison/linkdoku">{"Linkdoku"}</a> {" is licensed "}
                    <a href="https://www.gnu.org/licenses/#AGPL">{" GNU Affero GPL Version 3"}</a>{"."}
                </p>
            </div>
        </footer>
    }
}

#[function_component(Navbar)]
pub fn core_page_navbar() -> Html {
    let shortcut_icon = use_state(|| {
        use web_sys::HtmlLinkElement;
        let mut node = gloo::utils::head().first_child();
        while let Some(maybe_link) = node {
            node = maybe_link.next_sibling();
            if let Ok(link) = maybe_link.dyn_into::<HtmlLinkElement>() {
                if &link.rel() == "icon" {
                    return Some(link.href());
                }
            }
        }
        None
    });
    html! {
        <nav class={"navbar is-dark"} role={"navigation"} aria-label={"main navigation"}>
            <div class={"navbar-brand"}>
                <Link<Route> to={Route::Root} classes={"navbar-item"}>
                    {
                        if let Some(icon) = shortcut_icon.as_ref() {
                           html! {<img src={icon.to_string()} width={"32"} height={"32"} />}
                        } else {
                            html!{}
                        }
                    }
                    {"Linkdoku"}
                </Link<Route>>

                <a role={"button"} class={"navbar-burger"} aria-label={"menu"} aria-expanded={"false"} data-target={"navbarMenu"}>
                    <span aria-hidden={"true"}></span>
                    <span aria-hidden={"true"}></span>
                    <span aria-hidden={"true"}></span>
                </a>
            </div>

            <div id={"navbarMenu"} class={"navbar-menu"}>
                <div class={"navbar-start"}>
                    <Link<Route> to={Route::Root} classes={"navbar-item"}>
                        {"Home"}
                    </Link<Route>>
                </div>

                <div class={"navbar-end"}>
                    <UserMenuNavbarItem />
                    <div class={"navbar-item"}>
                    </div>
                </div>
            </div>
        </nav>
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BaseURI {
    pub base: Url,
}

#[derive(Debug, Clone, PartialEq, Properties)]
pub struct BaseURIProviderProps {
    pub children: Children,
}

#[derive(Debug, Clone)]
pub struct ReqwestClient {
    client: Arc<Client>,
}

impl PartialEq for ReqwestClient {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.client, &other.client)
    }
}

#[function_component(BaseURIProvider)]
pub fn core_base_uri_provider(props: &BaseURIProviderProps) -> Html {
    let uri = use_state(|| {
        let uri = gloo::utils::document()
            .base_uri()
            .expect("Could not read document")
            .expect("Document lacked .baseURI");
        let mut uri = Url::parse(&uri).expect("Base URI was bad");
        uri.set_path("/");
        BaseURI { base: uri }
    });

    let client = use_state(|| ReqwestClient {
        client: Arc::new(
            Client::builder()
                .build()
                .expect("Unable to construct client"),
        ),
    });

    let children = props.children.clone();

    html! {
        <ContextProvider<BaseURI> context={(*uri).clone()}>
            <ContextProvider<ReqwestClient> context={(*client).clone()}>
                {children}
            </ContextProvider<ReqwestClient>>
        </ContextProvider<BaseURI>>
    }
}

/// Retrieve an API url
///
/// For example: `use_api_url("/login/status")`.
///
/// The given api must start with `/` or things will fail.
pub fn use_api_url(api: &str) -> Url {
    let api_path = format!("/api{}", api);
    let base = use_context::<BaseURI>().expect("Cannot use_api_url() outside of <BaseURIProvider>");
    let mut ret = base.base;
    ret.set_path(&api_path);
    ret.set_fragment(None);
    ret.set_query(None);
    ret
}

pub const NO_BODY: Option<()> = None;

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum APIError {
    #[error("URL Parse Error: {0}")]
    URLParseError(#[from] url::ParseError),
    #[error("Error with reqwest: {0}")]
    ReqwestError(String),
}

impl From<reqwest::Error> for APIError {
    fn from(v: reqwest::Error) -> Self {
        APIError::ReqwestError(format!("{}", v))
    }
}

/// Make an async API call.
///
/// You *must* have acquired the API Url already
/// but you can specify additional query string
/// or body here.
pub async fn make_api_call<IN, OUT>(
    client: ReqwestClient,
    api: &str,
    query_params: impl IntoIterator<Item = (&str, &str)>,
    body: Option<IN>,
) -> Result<OUT, APIError>
where
    IN: Serialize,
    OUT: DeserializeOwned,
{
    let url = Url::parse_with_params(api, query_params)?;
    let request = if let Some(body) = body {
        client.client.post(url).json(&body).build()?
    } else {
        client.client.get(url).build()?
    };
    let response = client.client.execute(request).await?;
    Ok(response.json().await?)
}
