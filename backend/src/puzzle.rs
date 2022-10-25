use axum::{
    extract::Path,
    routing::{get, post},
    Extension, Json, Router,
};
use linkdoku_common::{CreatePuzzleResponse, Puzzle as APIPuzzle, Visibility};
use tower_cookies::Cookies;

use crate::{
    dbconn::{self, Database},
    login::login_flow_status,
};

async fn create_puzzle(
    cookies: Cookies,
    Json(puzzle): Json<APIPuzzle>,
    Extension(mut dbconn): Extension<Database>,
) -> Json<CreatePuzzleResponse> {
    let flow = login_flow_status(&cookies).await;
    let user = match flow.user() {
        Some(x) => x,
        None => {
            // User isn't logged in, cannot possibly create puzzles
            return CreatePuzzleResponse::NotLoggedIn.into();
        }
    };

    // Must not have supplied a UUID
    if !puzzle.uuid.is_empty() {
        return CreatePuzzleResponse::FailedUUIDSupplied.into();
    }

    // Verify if the role supplied by the user matches one the user has access to
    if !user.has_role(&puzzle.owner) {
        return CreatePuzzleResponse::InvalidOwnerRole.into();
    }

    // Owner matches, next validation is that there is exactly one puzzle state
    if puzzle.states.len() != 1 {
        return CreatePuzzleResponse::InvalidStateVector.into();
    }

    // Verify that the puzzle visibility and the state visibility are restricted
    if puzzle.visibility != Visibility::Restricted
        || puzzle.visibility_changed.is_some()
        || puzzle.states[0].visibility != Visibility::Restricted
        || puzzle.states[0].visibility_changed.is_some()
    {
        return CreatePuzzleResponse::InvalidVisiblityData.into();
    }

    // Let's try and transform the puzzle into a database puzzle
    let puzzle = dbconn::Puzzle::from(puzzle);

    // At this point it's safe to create the puzzle...
    match dbconn.create_puzzle(&puzzle).await {
        Ok(uuid) => CreatePuzzleResponse::Success(uuid),
        Err(e) => CreatePuzzleResponse::DatabaseFailure(e.to_string()),
    }
    .into()
}

pub async fn retrieve_puzzle(
    cookies: Cookies,
    Path(puzzle): Path<String>,
    Extension(mut dbconn): Extension<Database>,
) -> Json<Option<APIPuzzle>> {
    let puzzle_data = match dbconn.puzzle_by_uuid_or_short_name(&puzzle).await {
        Ok(puzzle) => puzzle,
        Err(_) => return None.into(),
    };

    tracing::info!("Fetched puzzle {}", puzzle);

    let is_logged_in_owner = {
        let flow = login_flow_status(&cookies).await;
        match flow.user() {
            Some(x) => x.has_role(puzzle_data.owner()),
            None => false,
        }
    };

    tracing::info!(
        "Caller is logged in as owner of puzzle: {}",
        is_logged_in_owner
    );

    let can_see_puzzle = match puzzle_data.visibility() {
        Visibility::Restricted => is_logged_in_owner,
        Visibility::Public => true,
        Visibility::Published => true,
    };

    if !can_see_puzzle {
        tracing::info!("Calling user? cannot see puzzle");
        return None.into();
    }

    Some(puzzle_data.as_api_puzzle(is_logged_in_owner)).into()
}

pub fn router() -> Router {
    Router::new()
        .route("/create", post(create_puzzle))
        .route("/get/:puzzle", get(retrieve_puzzle))
}
