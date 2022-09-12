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

    pub async fn create_default_role(&mut self, identity: &Identity) -> DatabaseResult<()> {
        let role_key = format!("role:{}", identity.get_default_role());
        let name = identity.display_name().to_string();
        let owner = identity.uuid().to_string();
        let bio = format!("# {}\n\nTODO", identity.display_name());
        Cmd::hset_multiple(role_key, &[("name", name), ("owner", owner), ("bio", bio)])
            .query_async(&mut self.conn)
            .await?;
        Ok(())
    }
}
