use axum::{extract::Path, routing::get, Extension, Json, Router};
use linkdoku_common::RoleData;

use crate::dbconn::{Database, DatabaseError};

async fn role_by_uuid(
    Path(role): Path<String>,
    Extension(mut dbconn): Extension<Database>,
) -> Json<RoleData> {
    match dbconn.role_by_uuid(&role).await {
        Ok(role) => Json::from(RoleData {
            uuid: role.uuid().to_string(),
            owner: role.owner().to_string(),
            short_name: role.short_name().to_string(),
            display_name: role.display_name().to_string(),
            bio: role.bio().to_string(),
        }),
        Err(DatabaseError::NotFound(_)) => Json::from(RoleData::default()),
        Err(e) => {
            tracing::error!("Failure retrieving role {}: {:?}", role, e);
            Json::from(RoleData::default())
        }
    }
}

pub fn router() -> Router {
    Router::new().route("/get/:role", get(role_by_uuid))
}
