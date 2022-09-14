use serde::{Deserialize, Serialize};

/// Roles are part of how an identity interacts with the rest of the objects
///
/// An identity can be associated with multiple roles; and roles may have
/// multiple identities associated with them.
///
/// This is to permit (a) personal vs. official roles, and (b) to permit
/// helpers to administer official roles.
///
/// Redis keys:
///
/// * `role:{uuid}` hash containing owner, short_name, display_name, bio, etc.
/// * `role:byname` hash containing short_name -> UUID mappings
#[derive(Debug, Serialize, Deserialize)]
pub struct Role {
    uuid: String,
    owner: String,
    short_name: String,
    display_name: String,
    bio: String,
}

impl Role {
    /// UUID of role
    pub fn uuid(&self) -> &str {
        &self.uuid
    }

    /// Owner of role
    pub fn owner(&self) -> &str {
        &self.owner
    }

    /// Short name of role
    pub fn short_name(&self) -> &str {
        &self.short_name
    }

    /// Display name of role
    pub fn display_name(&self) -> &str {
        &self.display_name
    }

    /// Bio text of role
    pub fn bio(&self) -> &str {
        &self.bio
    }

    /// Load a role from the database
    pub(crate) fn from_list(uuid: &str, mut kvs: impl Iterator<Item = String>) -> Role {
        let mut ret = Role {
            uuid: uuid.to_string(),
            owner: String::new(),
            short_name: String::new(),
            display_name: String::new(),
            bio: String::new(),
        };
        while let Some(key) = kvs.next() {
            if let Some(value) = kvs.next() {
                match key.as_str() {
                    "owner" => ret.owner = value,
                    "short_name" => ret.short_name = value,
                    "display_name" => ret.display_name = value,
                    "bio" => ret.bio = value,
                    _ => tracing::warn!("Unknown kv pair decoding Role: {}={}", key, value),
                }
            }
        }
        ret
    }
}
