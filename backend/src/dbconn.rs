//! Redis database stuff for Linkdoku
//!
//!

use std::{error::Error, fmt::Display};

use axum::Extension;
use redis::{aio::ConnectionManager, Client, Cmd, RedisError, Script};

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
pub enum DatabaseError {
    NotFound(String),
    Redis(RedisError),
}

impl Display for DatabaseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Database error: ")?;
        match self {
            Self::NotFound(s) => write!(f, "{} not found", s),
            Self::Redis(rediserror) => write!(f, "{}", rediserror),
        }
    }
}

impl Error for DatabaseError {}

pub type DatabaseResult<T> = Result<T, DatabaseError>;

impl From<RedisError> for DatabaseError {
    fn from(e: RedisError) -> Self {
        DatabaseError::Redis(e)
    }
}

pub async fn redis_layer(config: &Configuration) -> DatabaseResult<Extension<Database>> {
    let client = Client::open(config.redis_url.clone())?;
    let conn_mgr = ConnectionManager::new(client).await?;
    Ok(Extension(Database { conn: conn_mgr }))
}

mod identity;
pub use identity::*;

mod role;
pub use role::*;

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
    #[allow(dead_code)]
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
    #[allow(dead_code)]
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

    pub async fn role_by_uuid(&mut self, uuid: &str) -> DatabaseResult<Role> {
        let kvs: Vec<String> = Cmd::hgetall(format!("role:{}", uuid))
            .query_async(&mut self.conn)
            .await?;
        if kvs.is_empty() {
            Err(DatabaseError::NotFound(format!("role:{}", uuid)))
        } else {
            Ok(Role::from_list(uuid, kvs.into_iter()))
        }
    }

    /// Normalise a role name, and ensure it is unique.
    /// Note: this is no guarantee of uniqueness by the time you get to the server later, but it's
    /// a good way to ensure nothing unusual happens.
    pub async fn normalise_and_unique_rolename(
        &mut self,
        role_name: &str,
    ) -> DatabaseResult<String> {
        // Step one is to take the lower-cased ascii version of role_name
        let mut role_name = role_name.to_ascii_lowercase();
        // Next we replace any spaces with underscores
        role_name = role_name.replace(' ', "_");
        // Next, we remove anything which isn't `-`, `_`, `.`, a letter, or a digit
        role_name.retain(|c| "abcdefghijklmnopqrstuvwxyz0123456789-_.".contains(c));
        // Now we take the role name and if it's a reserved word we add an underscore afterwards
        const RESERVED_ROLE_NAMES: &[&str] = &["api", "-", "linkdoku"];
        if RESERVED_ROLE_NAMES.iter().any(|&s| s == role_name) {
            role_name.push('_');
        }
        // Finally we set a counter at zero, and we try and find a unique role name...
        let mut full_role_name = role_name.clone();
        let mut counter = 0;
        loop {
            let found: bool = Cmd::hexists("role:byname", &full_role_name)
                .query_async(&mut self.conn)
                .await?;
            if !found {
                break Ok(full_role_name);
            }
            full_role_name = format!("{}_{}", role_name, counter);
            counter += 1;
        }
    }

    pub async fn create_default_role(&mut self, identity: &Identity) -> DatabaseResult<()> {
        let uuid = identity.get_default_role();
        let display_name = identity.display_name().to_string();
        let short_name = self
            .normalise_and_unique_rolename(identity.display_name())
            .await?;
        let owner = identity.uuid().to_string();
        let bio = format!("# {}\n\nTODO", identity.display_name());

        const CREATE_ROLE_SCRIPT: &str = include_str!("scripts/create_role.lua");
        let script = Script::new(CREATE_ROLE_SCRIPT);
        let mut invocation = script.prepare_invoke();
        invocation
            .key(format!("role:{}", uuid))
            .key("role:byname")
            .key(format!("identity:{}:roles", identity.uuid()))
            .arg(uuid)
            .arg(owner)
            .arg(short_name)
            .arg(display_name)
            .arg(bio);
        Ok(invocation.invoke_async(&mut self.conn).await?)
    }
}
