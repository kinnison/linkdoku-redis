//! Puzzle related stuff
//!

use linkdoku_common::{PuzzleData, PuzzleState, Rating, UrlEntry};
use serde_json::{json, Value};
use stylist::{style, yew::*};
use yew::prelude::*;
use yew_bulma_tabs::{TabContent, Tabbed};
use yew_hooks::prelude::*;
use yew_markdown::{editor::MarkdownEditor, render::MarkdownRender};
use yew_router::prelude::*;
use yew_toastrack::*;

use web_sys::{HtmlButtonElement, HtmlInputElement, HtmlSelectElement};

use serde::{Deserialize, Serialize};

use crate::{
    components::{
        core::{make_api_call, use_api_url, use_page_url, ReqwestClient},
        login::LoginStatus,
        utility::{CopyButton, Tooltip, TooltipAlignment},
    },
    utils::cache::{CacheEntry, ObjectCache},
    Route,
};

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq, Eq)]
pub struct CreatePuzzleState {
    pub owner: String,
    pub short_name: String,
    pub display_name: String,
    pub puzzle_state: PuzzleState,
}

#[function_component(NoPuzzleRedirect)]
pub fn no_puzzle_redirect() -> Html {
    use_history().expect("no history?").replace(Route::Root);
    html! {}
}

#[derive(Properties, PartialEq)]
pub struct PuzzleRatingProps {
    pub value: Rating,
    pub onclick: Option<Callback<Rating>>,
}

#[styled_component(PuzzleRating)]
pub fn puzzle_rating(props: &PuzzleRatingProps) -> Html {
    let tutorial_icon = html! {
        <i class={"fas fa-solid fa-graduation-cap"} />
    };
    let star_icon = html! {
        <i class={"fas fa-solid fa-star"} />
    };

    use Rating::*;

    let render_value = use_state(|| props.value);

    let mouseleave = Callback::from({
        let setter = render_value.setter();
        let reset = props.value;
        move |_| {
            setter.set(reset);
        }
    });

    let draw_icon = |on: bool, icon: Html, rating: Rating| {
        let onmouseover = if props.onclick.is_some() {
            let setter = render_value.setter();
            Some(Callback::from(move |_| {
                setter.set(rating);
            }))
        } else {
            None
        };
        let onclick = if let Some(onclick) = &props.onclick {
            let onclick = onclick.clone();
            Some(Callback::from(move |_| onclick.emit(rating)))
        } else {
            None
        };
        html! {
            <span class={if on { "icon has-text-success"} else { "icon" }} onmouseover={onmouseover} onclick={onclick}>
                {icon}
            </span>
        }
    };

    let outer_class = classes!(
        "button",
        style!("display: inline-block; height: auto;").unwrap()
    );

    let render_value = *render_value;
    html! {
        <div class={outer_class}>
            <span onmouseleave={mouseleave}>
                {draw_icon(matches!(render_value, Tutorial), tutorial_icon, Tutorial)}
                {" | "}
                {draw_icon(matches!(render_value, Beginner | Easy | Regular | Hard | VeryHard), star_icon.clone(), Beginner)}
                {draw_icon(matches!(render_value,            Easy | Regular | Hard | VeryHard), star_icon.clone(), Easy)}
                {draw_icon(matches!(render_value,                   Regular | Hard | VeryHard), star_icon.clone(), Regular)}
                {draw_icon(matches!(render_value,                             Hard | VeryHard), star_icon.clone(), Hard)}
                {draw_icon(matches!(render_value,                                    VeryHard), star_icon.clone(), VeryHard)}
            </span>
            <br />
            <span>
                {render_value.title()}
            </span>
        </div>
    }
}

#[derive(Properties, PartialEq, Eq, Debug)]
pub struct PuzzlePageProps {
    pub puzzle: String,
}

#[styled_component(PuzzlePage)]
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

    let state_index = use_state(|| puzzle_data.states.len().checked_sub(1));

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

    let puzzle_uuid_url = use_page_url(Route::PuzzlePage {
        puzzle: puzzle_data.uuid.clone(),
    });

    let puzzle_short_name_url = use_page_url(Route::PuzzlePage {
        puzzle: puzzle_data.short_name.clone(),
    });

    let valign_top = use_style("vertical-align: top;");
    let valign_middle = use_style("vertical-align: middle;");

    let current_state = state_index.map(|n| &puzzle_data.states[n]);

    let rating = if let Some(state) = current_state {
        html! {
            <span class={valign_middle}>
                <PuzzleRating value={state.setter_rating} />
            </span>
        }
    } else {
        html! {}
    };

    let description = current_state.map(|s| s.description.as_str()).unwrap_or("");

    html! {
        <>
            <div class={"block"}>
                <span class={"title is-1"}>
                    {puzzle_data.display_name.clone()}
                </span>
                <span class={valign_top}>
                    <Tooltip content={"Copy link to puzzle"} alignment={TooltipAlignment::Right}>
                        <CopyButton content={puzzle_short_name_url.as_str().to_string()} />
                    </Tooltip>
                    <Tooltip content={"Copy permalink to puzzle"} alignment={TooltipAlignment::Right}>
                        <CopyButton content={puzzle_uuid_url.as_str().to_string()} icon={"hashtag"}/>
                    </Tooltip>
                </span>
                {rating}
                <MarkdownRender markdown={description.to_string()} />
            </div>
        </>
    }
}

#[derive(Properties, PartialEq)]
struct PuzzleStateEditorProps {
    initial: PuzzleState,
    changed: Callback<PuzzleState>,
}

#[derive(Copy, Clone, PartialEq, Eq)]
enum EditorKind {
    Nothing,
    FPuzzles,
    URLs,
    Pack,
}

const KIND_TITLE_NOTHING: &str = "No data";
const KIND_TITLE_FPUZZLES: &str = "F-Puzzles data";
const KIND_TITLE_URLS: &str = "List of URLs";
const KIND_TITLE_PACK: &str = "List of puzzles";

impl EditorKind {
    fn title(self) -> &'static str {
        use EditorKind::*;
        match self {
            Nothing => KIND_TITLE_NOTHING,
            FPuzzles => KIND_TITLE_FPUZZLES,
            URLs => KIND_TITLE_URLS,
            Pack => KIND_TITLE_PACK,
        }
    }

    fn from_title(title: &str) -> Self {
        use EditorKind::*;
        match title {
            KIND_TITLE_NOTHING => Nothing,
            KIND_TITLE_FPUZZLES => FPuzzles,
            KIND_TITLE_URLS => URLs,
            KIND_TITLE_PACK => Pack,
            _ => unreachable!(),
        }
    }
}

#[function_component(PuzzleStateEditor)]
fn puzzle_state_editor(props: &PuzzleStateEditorProps) -> Html {
    let fpuzzles_data = use_state(|| match &props.initial.data {
        PuzzleData::FPuzzles(fpuzz) => Some(fpuzz.clone()),
        _ => None,
    });
    let urls_data = use_state(|| match &props.initial.data {
        PuzzleData::URLs(urls) => urls.clone(),
        _ => Vec::new(),
    });
    let pack_data = use_state(|| match &props.initial.data {
        PuzzleData::Pack(pack) => pack.clone(),
        _ => Vec::new(),
    });
    let editor_kind = use_state(|| match props.initial.data {
        PuzzleData::Nothing => EditorKind::Nothing,
        PuzzleData::FPuzzles(_) => EditorKind::FPuzzles,
        PuzzleData::URLs(_) => EditorKind::URLs,
        PuzzleData::Pack(_) => EditorKind::Pack,
    });

    let description = use_state(|| props.initial.description.clone());
    let setter_rating = use_state(|| props.initial.setter_rating);

    use_effect_with_deps(
        {
            let visibility = props.initial.visibility;
            let visibility_changed = props.initial.visibility_changed.clone();
            let changed = props.changed.clone();

            let empty_grid: Value = json!({
                "size": 3,
                "grid": [
                    [{}, {}, {}],
                    [{}, {}, {}],
                    [{}, {}, {}],
                ]
            });

            move |(
                editor_kind,
                fpuzzles_data,
                urls_data,
                pack_data,
                description,
                setter_rating,
            ): &(
                EditorKind,
                Option<Value>,
                Vec<UrlEntry>,
                Vec<String>,
                String,
                Rating,
            )| {
                let new_state = PuzzleState {
                    description: description.clone(),
                    setter_rating: *setter_rating,
                    visibility,
                    visibility_changed,
                    data: match editor_kind {
                        EditorKind::Nothing => PuzzleData::Nothing,
                        EditorKind::FPuzzles => PuzzleData::FPuzzles(
                            fpuzzles_data.clone().unwrap_or_else(|| empty_grid.clone()),
                        ),
                        EditorKind::URLs => PuzzleData::URLs(urls_data.clone()),
                        EditorKind::Pack => PuzzleData::Pack(pack_data.clone()),
                    },
                };
                changed.emit(new_state);
                || ()
            }
        },
        (
            (*editor_kind).clone(),
            (*fpuzzles_data).clone(),
            (*urls_data).clone(),
            (*pack_data).clone(),
            (*description).clone(),
            (*setter_rating).clone(),
        ),
    );

    let rating_control = {
        let rating_click = Callback::from({
            let setter_rating = setter_rating.clone();
            move |rating| setter_rating.set(rating)
        });

        html! {
            <div class={"field"}>
                <label class={"label"}>
                    {"Estimated (setter) puzzle rating"}
                </label>
                <div class={"control"}>
                    <PuzzleRating value={*setter_rating} onclick={rating_click}/>
                </div>
            </div>
        }
    };

    let description_control = {
        let description_changed = Callback::from({
            let description = description.clone();
            move |new| {
                description.set(new);
            }
        });
        html! {
            <div class={"field"}>
                <label class={"label"}>
                    {"Puzzle description"}
                </label>
                <div class={"control"}>
                    <MarkdownEditor initial={(*description).clone()} onchange={description_changed} />
                </div>
            </div>
        }
    };

    let puzzle_data_control = {
        let tabchanged = Callback::from({
            let editor_kind = editor_kind.clone();
            move |title: String| {
                editor_kind.set(EditorKind::from_title(&title));
            }
        });

        html! {
            <div class={"field"}>
                <label class={"label"}>
                    {"Puzzle data"}
                </label>
                <div class={"control"}>
                    <Tabbed default={editor_kind.title()} tabchanged={tabchanged}>
                        <TabContent title={EditorKind::Nothing.title()}>
                            <span class={"title"}>{"No extra data"}</span>
                        </TabContent>
                    </Tabbed>
                </div>
            </div>
        }
    };

    html! {
        <>
            {rating_control}
            {description_control}
            {puzzle_data_control}
        </>
    }
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

    let puzzle_data_control = {
        let state_changed = Callback::from({
            let state = state.clone();
            move |new_puzzle_state| {
                let mut new_state = (*state).clone();
                new_state.puzzle_state = new_puzzle_state;
                state.set(new_state);
            }
        });
        html! {
            <PuzzleStateEditor initial={state.puzzle_state.clone()} changed={state_changed}/>
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
                use linkdoku_common::{CreatePuzzleResponse, Puzzle, Visibility};
                let button: HtmlButtonElement = button_ref.cast().unwrap();
                button.set_class_name(pending_classes);
                let puzzle = Puzzle {
                    uuid: String::new(),
                    owner: state.owner.clone(),
                    short_name: state.short_name.clone(),
                    display_name: state.display_name.clone(),
                    visibility: Visibility::Restricted,
                    visibility_changed: None,
                    states: vec![state.puzzle_state.clone()],
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
            {puzzle_data_control}
            {create_puzzle_button}
        </>
    }
}
