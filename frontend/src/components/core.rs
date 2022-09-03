//! Core components for linkdoku
//!
//!

use crate::Route;

use crate::components::user::UserMenuNavbarItem;

use reqwest::Url;
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
    html! {
        <nav class={"navbar is-dark"} role={"navigation"} aria-label={"main navigation"}>
            <div class={"navbar-brand"}>
                <Link<Route> to={Route::Root} classes={"navbar-item"}>
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

    let children = props.children.clone();

    html! {
        <ContextProvider<BaseURI> context={(*uri).clone()}>
            {children}
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
