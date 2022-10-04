//! Puzzle related stuff
//!

use linkdoku_common::Rating;
use yew::prelude::*;
use yew_hooks::prelude::*;
use yew_markdown::editor::MarkdownEditor;
use yew_router::prelude::*;
use yew_toastrack::*;

use web_sys::{HtmlButtonElement, HtmlInputElement, HtmlSelectElement};

use serde::{Deserialize, Serialize};

use crate::{
    components::{
        core::{make_api_call, use_api_url, ReqwestClient},
        login::LoginStatus,
    },
    utils::cache::{CacheEntry, ObjectCache},
    Route,
};

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq, Eq)]
pub struct CreatePuzzleState {
    pub owner: String,
    pub short_name: String,
    pub display_name: String,
    pub description: String,
    pub rating: Rating,
}

#[function_component(NoPuzzleRedirect)]
pub fn no_puzzle_redirect() -> Html {
    use_history().expect("no history?").replace(Route::Root);
    html! {}
}

#[derive(Properties, PartialEq, Eq, Debug)]
pub struct PuzzlePageProps {
    pub puzzle: String,
}

#[function_component(PuzzlePage)]
pub fn puzzle_page(props: &PuzzlePageProps) -> Html {
    let cache = use_context::<ObjectCache>().expect("No cache?");
    let puzzle_data = cache.cached_puzzle(&props.puzzle);
    let history = use_history().expect("No history?");

    gloo::console::log!(format!("Puzzle Page: puzzle={:?}", &*puzzle_data));

    if puzzle_data.is_pending() {
        // Eventually render a page spinner
        return html! {};
    }

    if puzzle_data.is_missing() {
        Toaster::toast(
            Toast::new(&format!("Puzzle {} was not found", props.puzzle))
                .with_lifetime(Some(5000))
                .with_level(ToastLevel::Danger),
        );
        history.push(Route::Root);
        return html! {};
    }

    if puzzle_data.is_error() {
        Toaster::toast(
            Toast::new(&format!(
                "Error fetching puzzle {}: {}",
                props.puzzle,
                puzzle_data.error_text()
            ))
            .with_lifetime(Some(5000))
            .with_level(ToastLevel::Danger),
        );
        history.push(Route::Root);
        return html! {};
    }

    // We have the role data, so let's render it
    let puzzle_data = puzzle_data.value().unwrap().clone();

    // if it turns out we were invoked by UUID, redirect to short-name because it's nicer for copy/pasta
    if props.puzzle == puzzle_data.uuid {
        // check if the current history value shows the current puzzle by uuid too
        if let Some(Route::PuzzlePage { puzzle }) = history.location().route::<Route>() {
            gloo::console::log!(format!(
                "puzzle == {}, uuid == {}, route_puzzle == {}",
                props.puzzle, puzzle_data.uuid, puzzle
            ));
            if puzzle == props.puzzle {
                // Still showing UUID, so replace in the URL
                history.replace(Route::PuzzlePage {
                    puzzle: puzzle_data.short_name.clone(),
                });
            }
        }
    }

    use_title(format!("Linkdoku - Puzzle - {}", puzzle_data.display_name));

    gloo::console::log!(format!("Rendering puzzle page for {}", props.puzzle));
    html! {}
}

#[function_component(CreatePuzzle)]
pub fn create_puzzle() -> Html {
    let cache = use_context::<ObjectCache>().expect("No cache?");
    let history = use_history().expect("no history?");
    let login_status = use_context::<LoginStatus>().expect("No login status?");

    if login_status.is_unknown() {
        // Eventually a spinner
        return html! {};
    }

    if !login_status.is_logged_in() {
        Toaster::toast(
            Toast::new("Cannot create puzzle, you must log in first.")
                .with_lifetime(Some(5000))
                .with_level(ToastLevel::Danger),
        );
        history.push(Route::Root);
        return html! {};
    }

    let incoming_state: Option<CreatePuzzleState> = history.state().ok();
    let state = use_state_eq(CreatePuzzleState::default);
    if let Some(incoming_state) = incoming_state {
        gloo::console::log!(format!("Incoming state: {:?}", incoming_state));
        state.set(incoming_state);
        // Clear state
        history.replace(Route::CreatePuzzle);
        return html! {};
    }

    if state.owner.is_empty() {
        let mut new_state = (*state).clone();
        new_state.owner = login_status.current_role().unwrap().to_string();
        state.set(new_state);
    }

    gloo::console::log!(format!(
        "Rendering create_puzzle UI with state=={:?}",
        &*state
    ));

    let owner_control = {
        let role_options = login_status
        .roles()
        .iter()
        .flat_map(|uuid| {
            let role_data = cache.cached_role(uuid);
            match &*role_data {
                CacheEntry::Pending => Some(html! {
                    <option value={uuid.clone()} selected={*uuid == state.owner}>{uuid}</option>
                }),
                CacheEntry::Missing | CacheEntry::Error(_) => None,
                CacheEntry::Value(role_data) => Some(html! {
                    <option value={role_data.uuid.clone()} selected={*uuid == state.owner}>{role_data.display_name.clone()}</option>
                }),
            }
        })
        .collect::<Vec<_>>();

        let selector = use_node_ref();

        let owner_changed = Callback::from({
            let state = state.clone();
            let selector = selector.clone();
            move |_| {
                let selector: HtmlSelectElement = selector.cast().unwrap();
                let mut new_state = (*state).clone();
                new_state.owner = selector.value();
                state.set(new_state);
            }
        });

        html! {
            <div class={"field"}>
                <label class={"label"}>
                    {"Owning role"}
                </label>
                <div class={"control"}>
                    <div class={"select"} ref={selector}>
                        <select onchange={owner_changed}>
                            {role_options}
                        </select>
                    </div>
                </div>
            </div>
        }
    };

    let short_name_control = {
        let input_ref = use_node_ref();

        let short_name_changed = Callback::from({
            let input_ref = input_ref.clone();
            let state = state.clone();
            move |_| {
                let input: HtmlInputElement = input_ref.cast().unwrap();
                let mut new_state = (*state).clone();
                new_state.short_name = input.value();
                state.set(new_state);
            }
        });

        html! {
            <div class={"field"}>
                <label class={"label"}>
                    {"Puzzle short name"}
                </label>
                <div class={"control"}>
                    <input ref={input_ref} class={"input"} value={state.short_name.clone()} onchange={short_name_changed}/>
                </div>
            </div>
        }
    };

    let display_name_control = {
        let input_ref = use_node_ref();

        let display_name_changed = Callback::from({
            let input_ref = input_ref.clone();
            let state = state.clone();
            move |_| {
                let input: HtmlInputElement = input_ref.cast().unwrap();
                let mut new_state = (*state).clone();
                new_state.display_name = input.value();
                state.set(new_state);
            }
        });

        html! {
            <div class={"field"}>
                <label class={"label"}>
                    {"Puzzle display name"}
                </label>
                <div class={"control"}>
                    <input ref={input_ref} class={"input"} value={state.display_name.clone()} onchange={display_name_changed}/>
                </div>
            </div>
        }
    };

    let description_control = {
        let description_changed = Callback::from({
            let state = state.clone();
            move |content| {
                let mut new_state = (*state).clone();
                new_state.description = content;
                state.set(new_state);
            }
        });
        html! {
            <div class={"field"}>
                <label class={"label"}>
                    {"Puzzle description"}
                </label>
                <div class={"control"}>
                    <MarkdownEditor initial={state.description.clone()} onchange={description_changed} />
                </div>
            </div>
        }
    };

    let rating_control = {
        let selector = use_node_ref();

        let ratings = Rating::values()
            .iter().copied()
            .map(|rating| {
                html! {
                    <option value={rating.value()} selected={rating == state.rating}>{rating.title()}</option>
                }
            })
            .collect::<Vec<_>>();

        let rating_changed = Callback::from({
            let selector = selector.clone();
            let state = state.clone();
            move |_| {
                let selector: HtmlSelectElement = selector.cast().unwrap();
                let mut new_state = (*state).clone();
                new_state.rating = Rating::from_value(&selector.value());
                state.set(new_state);
            }
        });
        html! {
            <div class={"field"}>
                <label class={"label"}>
                    {"Puzzle rating"}
                </label>
                <div class={"control"}>
                    <div class={"select"}>
                        <select ref={selector} onchange={rating_changed}>
                            {ratings}
                        </select>
                    </div>
                </div>
            </div>
        }
    };

    let create_puzzle_button = {
        let button_ref = use_node_ref();

        let plain_classes = "button is-primary";
        let pending_classes = "button is-primary is-loading";

        let create_puzzle_url = use_api_url("/puzzle/create");
        let client = use_context::<ReqwestClient>().expect("No API client");

        let onclick = Callback::from({
            let button_ref = button_ref.clone();
            let state = state.clone();
            let history = history.clone();
            move |_| {
                use linkdoku_common::{
                    CreatePuzzleResponse, Puzzle, PuzzleData, PuzzleState, Visibility,
                };
                let button: HtmlButtonElement = button_ref.cast().unwrap();
                button.set_class_name(pending_classes);
                let puzzle = Puzzle {
                    uuid: String::new(),
                    owner: state.owner.clone(),
                    short_name: state.short_name.clone(),
                    display_name: state.display_name.clone(),
                    visibility: Visibility::Restricted,
                    visibility_changed: None,
                    states: vec![PuzzleState {
                        description: state.description.clone(),
                        setter_rating: state.rating,
                        data: PuzzleData::Nothing,
                        visibility: Visibility::Restricted,
                        visibility_changed: None,
                    }],
                };
                let client = client.clone();
                let create_puzzle_url = create_puzzle_url.clone();
                let history = history.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    let result: CreatePuzzleResponse =
                        match make_api_call(client, create_puzzle_url.as_str(), None, Some(puzzle))
                            .await
                        {
                            Ok(res) => res,
                            Err(e) => {
                                Toaster::toast(
                                    Toast::new(&format!("API Error: {}", e))
                                        .with_lifetime(Some(2000))
                                        .with_level(ToastLevel::Danger),
                                );
                                button.set_class_name(plain_classes);
                                return;
                            }
                        };
                    if let CreatePuzzleResponse::Success(uuid) = &result {
                        // Success, so redirect to this puzzle
                        Toaster::toast(
                            Toast::new("Created successfully")
                                .with_lifetime(Some(1000))
                                .with_level(ToastLevel::Success),
                        );
                        history.push(Route::PuzzlePage {
                            puzzle: uuid.clone(),
                        });
                    } else {
                        Toaster::toast(
                            Toast::new(&format!("Unable to create puzzle: {}", result))
                                .with_lifetime(Some(5000))
                                .with_level(ToastLevel::Danger),
                        );
                    }
                    button.set_class_name(plain_classes);
                });
            }
        });

        html! {
            <div class={"field is-grouped"}>
                <div class={"control"}>
                    <button ref={button_ref} class={plain_classes} onclick={onclick}>
                        {"Create puzzle…"}
                    </button>
                </div>
            </div>
        }
    };

    drop(state);
    drop(history);

    html! {
        <>
            <p class={"title is-1"}>
                {"Creating a puzzle..."}
            </p>
            {owner_control}
            {short_name_control}
            {display_name_control}
            {rating_control}
            {description_control}
            {create_puzzle_button}
        </>
    }
}
