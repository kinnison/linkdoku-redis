use serde::{Deserialize, Serialize};

/// Roles are part of how an identity interacts with the rest of the objects
///
/// An identity can be associated with multiple roles; and roles may have
/// multiple identities associated with them.
///
/// This is to permit (a) personal vs. official roles, and (b) to permit
/// helpers to administer official roles.
#[derive(Debug, Serialize, Deserialize)]
pub struct Role {
    uuid: String,
    owner: String,
    name: String,
    bio: String,
}

impl Role {
    /// Load a role from the database
    pub(crate) fn from_list(uuid: &str, mut kvs: impl Iterator<Item = String>) -> Role {
        let mut ret = Role {
            uuid: uuid.to_string(),
            owner: String::new(),
            name: String::new(),
            bio: String::new(),
        };
        while let Some(key) = kvs.next() {
            if let Some(value) = kvs.next() {
                match key.as_str() {
                    "owner" => ret.owner = value,
                    "name" => ret.name = value,
                    "bio" => ret.bio = value,
                    _ => tracing::warn!("Unknown kv pair decoding Role: {}={}", key, value),
                }
            }
        }
        ret
    }
}
