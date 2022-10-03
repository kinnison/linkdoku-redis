use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BackendLoginStatus {
    LoggedOut,
    LoggedIn {
        name: String,
        gravatar_hash: Option<String>,
        roles: Vec<String>,
        role: String,
    },
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LoginFlowStart {
    Idle,
    Redirect(String),
    Error(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LoginFlowResult {
    pub error: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoleData {
    pub uuid: String,
    pub owner: String,
    pub short_name: String,
    pub display_name: String,
    pub bio: String,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Visibility {
    #[default]
    Restricted,
    Public,
    Published,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Puzzle {
    pub uuid: String,
    pub owner: String,
    pub display_name: String,
    pub short_name: String,
    pub visibility: Visibility,
    pub visibility_changed: Option<String>,
    pub states: Vec<PuzzleState>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Rating {
    Easier,
    Easy,
    #[default]
    Normal,
    Hard,
    Harder,
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PuzzleState {
    pub description: String,
    pub setter_rating: Rating,
    pub data: PuzzleData,
    pub visibility: Visibility,
    pub visibility_changed: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum PuzzleData {
    #[default]
    Nothing,
    URLs(Vec<UrlEntry>),
    Pack(Vec<String>),
    FPuzzles(Value),
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UrlEntry {
    pub title: String,
    pub url: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CreatePuzzleResponse {
    /// Successful create, contained string is puzzle UUID
    Success(String),
    /// Failure because user is not logged in
    NotLoggedIn,
    /// Failure because provided puzzle contained a UUID
    FailedUUIDSupplied,
    /// Invalid role supplied as owner
    InvalidOwnerRole,
    /// Invalid state vector, must be exactly one entry
    InvalidStateVector,
    /// Invalid visibility data provided
    InvalidVisiblityData,
    /// Something went wrong in the database layer
    DatabaseFailure(String),
}
