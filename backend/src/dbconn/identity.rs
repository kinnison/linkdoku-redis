use serde::{Deserialize, Serialize};

/// Identities are stored in the `identity` prefix in Redis
///
/// An identity has cached information such as a display name
/// and a cached email hash.  We never store the full email address
/// of an identity and only store the email hash in order to permit
/// gravatars etc. to be supplied
#[derive(Serialize, Deserialize, Debug)]
pub struct Identity {
    pub(crate) uuid: String,
    pub(crate) display_name: String,
    pub(crate) gravatar_hash: Option<String>,
}

impl Identity {
    /// Create a new identity based on the given display name and
    /// email address.  This will create a new UUID for the identity
    /// as well, based on the subject identifier
    pub fn new(subj: &str, display_name: &str, email: Option<&str>) -> Identity {
        let uuid = format!("{:x}", md5::compute(subj));
        let display_name = display_name.to_string();
        let gravatar_hash = email.map(|s| format!("{:x}", md5::compute(s)));
        Identity {
            uuid,
            display_name,
            gravatar_hash,
        }
    }

    /// This identity's UUID
    pub fn uuid(&self) -> &str {
        &self.uuid
    }

    /// This identity's display_name
    pub fn display_name(&self) -> &str {
        &self.display_name
    }

    /// This identity's gravatar hash
    pub fn gravatar_hash(&self) -> Option<&str> {
        self.gravatar_hash.as_deref()
    }

    /// Internal conversion from redis key/value list
    pub(crate) fn from_list(uuid: &str, mut kvs: impl Iterator<Item = String>) -> Identity {
        let mut ret = Identity {
            uuid: uuid.to_string(),
            display_name: String::new(),
            gravatar_hash: None,
        };
        while let Some(key) = kvs.next() {
            if let Some(value) = kvs.next() {
                match key.as_str() {
                    "display_name" => ret.display_name = value,
                    "gravatar_hash" => ret.gravatar_hash = Some(value),
                    _ => tracing::warn!("Unknown kv pair decoding Identity: {}={}", key, value),
                }
            }
        }
        ret
    }

    /// Retrieve the UUID of the default role for this identity
    pub fn get_default_role(&self) -> String {
        format!(
            "{:x}",
            md5::compute(format!("identity:{}:defaultrole", self.uuid))
        )
    }
}
