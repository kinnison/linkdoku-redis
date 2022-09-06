//! Redis database stuff for Linkdoku
//!
//!

use std::{error::Error, fmt::Display};

use axum::Extension;
use redis::{aio::ConnectionManager, Client, Cmd, RedisError, Script};
use serde::{Deserialize, Serialize};

use crate::config::Configuration;

/// The Redis database connection
///
/// All interaction with Redis is done via this type, so that in the future
/// if there's a need to switch to SQL or otherwise, we just swap out the
/// implementation here and we're good.
#[derive(Clone)]
pub struct Database {
    conn: ConnectionManager,
}

/// On the off chance that something goes wrong, this error type will be returned.
///
/// You can't usefully interrogate it because that's considered deep voodoo.
#[derive(Debug)]
pub struct DatabaseError(RedisError);

impl Display for DatabaseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Database error: {}", self.0)
    }
}

impl Error for DatabaseError {}

pub type DatabaseResult<T> = Result<T, DatabaseError>;

impl From<RedisError> for DatabaseError {
    fn from(e: RedisError) -> Self {
        DatabaseError(e)
    }
}

pub async fn redis_layer(config: &Configuration) -> DatabaseResult<Extension<Database>> {
    let client = Client::open(config.redis_url.clone())?;
    let conn_mgr = ConnectionManager::new(client).await?;
    Ok(Extension(Database { conn: conn_mgr }))
}

/// Identities are stored in the `identity` prefix in Redis
///
/// An identity has cached information such as a display name
/// and a cached email hash.  We never store the full email address
/// of an identity and only store the email hash in order to permit
/// gravatars etc. to be supplied
#[derive(Serialize, Deserialize, Debug)]
pub struct Identity {
    uuid: String,
    display_name: String,
    gravatar_hash: Option<String>,
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
    fn from_list(uuid: &str, mut kvs: impl Iterator<Item = String>) -> Identity {
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
}

/// Database functions related to [Identity][]
///
/// Identities are stored in Redis in the following ways:
///
/// In all the below, `xxx` is an ID formed by hashing the subject identifier
///
/// * `identity:xxx` is a hash of display_name etc.
/// * `identity:xxx:roles` is the set of roles the identity has control of
impl Database {
    /// Acquire an identity from the database if it is available, by its computed id
    ///
    /// If the identity does not exist, this will return `Ok(None)`
    pub async fn identity_by_uuid(&mut self, uuid: &str) -> DatabaseResult<Option<Identity>> {
        let kvs: Vec<String> = Cmd::hgetall(format!("identity:{}", uuid))
            .query_async(&mut self.conn)
            .await?;
        if kvs.is_empty() {
            Ok(None)
        } else {
            Ok(Some(Identity::from_list(uuid, kvs.into_iter())))
        }
    }

    /// Acquire an identity from the database if it is available, by its subject identifier
    ///
    /// If the identity does not exist, this will return `Ok(None)`
    pub async fn identity_by_subject(&mut self, subj: &str) -> DatabaseResult<Option<Identity>> {
        let uuid = format!("{:x}", md5::compute(subj));
        self.identity_by_uuid(&uuid).await
    }

    /// Create an identity if it does not exist in the database, and if it exists, return the
    /// role list for it.
    pub async fn identity_upsert_and_roles(
        &mut self,
        identity: &Identity,
    ) -> DatabaseResult<Vec<String>> {
        const UPSERT_SCRIPT: &str = include_str!("scripts/identity_upsert.lua");
        let script = Script::new(UPSERT_SCRIPT);
        let mut invocation = script.prepare_invoke();
        invocation
            .key(format!("identity:{}", identity.uuid()))
            .key(format!("identity:{}:roles", identity.uuid()))
            .arg(identity.display_name())
            .arg(identity.gravatar_hash().unwrap_or(""));
        Ok(invocation.invoke_async(&mut self.conn).await?)
    }
}
