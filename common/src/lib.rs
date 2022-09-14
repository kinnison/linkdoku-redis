use serde::{Deserialize, Serialize};

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
