use axum::{extract::Path, routing::get, Extension, Json, Router};
use linkdoku_common::RoleData;

use crate::dbconn::{Database, DatabaseError};

async fn role_by_uuid_or_short_name(
    Path(role): Path<String>,
    Extension(mut dbconn): Extension<Database>,
) -> Json<Option<RoleData>> {
    tracing::info!("Looking for role: {}", role);
    match dbconn.role_by_uuid_or_short_name(&role).await {
        Ok(role) => {
            tracing::info!("Found role: {:?}", role);
            Json::from(Some(RoleData {
                uuid: role.uuid().to_string(),
                owner: role.owner().to_string(),
                short_name: role.short_name().to_string(),
                display_name: role.display_name().to_string(),
                bio: role.bio().to_string(),
            }))
        }
        Err(DatabaseError::NotFound(_)) => {
            tracing::warn!("Role {} not found", role);
            Json::from(None)
        }
        Err(e) => {
            tracing::error!("Failure retrieving role {}: {:?}", role, e);
            Json::from(None)
        }
    }
}

pub fn router() -> Router {
    Router::new().route("/get/:role", get(role_by_uuid_or_short_name))
}
