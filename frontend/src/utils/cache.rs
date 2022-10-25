//! Cache provision for Linkdoku
//!

use std::{fmt::Debug, future::Future, time::Duration};

use gloo::storage::{LocalStorage, Storage};
use js_sys::Date;
use linkdoku_common::{Puzzle, RoleData};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use yew::prelude::*;
use yew_hooks::{use_async_with_options, UseAsyncHandle, UseAsyncOptions};

use crate::components::core::{make_api_call, use_api_url, APIError, ReqwestClient, NO_BODY};

const ROLE_CACHE_LIFETIME: Duration = Duration::from_secs(60 * 5); // Five minute cache for roles
const PUZZLE_CACHE_LIFETIME: Duration = Duration::from_secs(60); // One minute cache for puzzles

#[derive(Properties, PartialEq)]
pub struct ObjectCacheProviderProps {
    pub children: Children,
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct CacheEntryWithExpiry<T> {
    expires: f64,
    entry: T,
}

#[derive(Clone, PartialEq, Eq)]
pub struct ObjectCache;

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum CacheEntry<V>
where
    V: Clone + PartialEq + Debug,
{
    Pending,
    Missing,
    Error(String),
    Value(V),
}

impl<V> CacheEntry<V>
where
    V: Clone + PartialEq + Debug,
{
    pub fn is_pending(&self) -> bool {
        matches!(self, Self::Pending)
    }

    pub fn is_missing(&self) -> bool {
        matches!(self, Self::Missing)
    }

    pub fn is_error(&self) -> bool {
        matches!(self, Self::Error(_))
    }

    pub fn error_text(&self) -> &str {
        match self {
            Self::Error(e) => e,
            _ => "",
        }
    }
    pub fn value(&self) -> Option<&V> {
        match self {
            Self::Value(v) => Some(v),
            _ => None,
        }
    }
}

#[function_component(ObjectCacheProvider)]
pub fn object_cache_provider(props: &ObjectCacheProviderProps) -> Html {
    let cache = ObjectCache;

    html! {
        <ContextProvider<ObjectCache> context={cache}>
            {props.children.clone()}
        </ContextProvider<ObjectCache>>
    }
}

impl ObjectCache {
    pub fn add_object<V>(&self, key: &str, lifetime: Duration, value: V)
    where
        V: Serialize + DeserializeOwned + Clone + 'static,
    {
        gloo::console::log!(format!("Adding cache entry for {}", key));

        LocalStorage::set(
            key,
            CacheEntryWithExpiry {
                expires: Date::now() + (lifetime.as_secs_f64() * 1000.0),
                entry: value,
            },
        )
        .unwrap_or_else(|_| panic!("Unable to set {} into storage", key));
    }

    pub fn get<V>(&self, key: &str) -> Option<V>
    where
        V: Serialize + DeserializeOwned + Clone + 'static,
    {
        let value: Option<CacheEntryWithExpiry<V>> = LocalStorage::get(key).ok();
        if let Some(value) = value {
            if Date::now() <= value.expires {
                gloo::console::log!(format!("Retrieved {} from cache", key));
                Some(value.entry)
            } else {
                gloo::console::log!(format!("Entry for {} expired", key));
                LocalStorage::delete(key);
                None
            }
        } else {
            None
        }
    }

    pub fn use_cached_value<F, T, E>(
        &self,
        key: &str,
        lifetime: Duration,
        fetcher: F,
    ) -> UseAsyncHandle<Option<T>, E>
    where
        T: Clone + Serialize + DeserializeOwned + 'static,
        E: Clone + 'static,
        F: Future<Output = Result<Option<T>, E>> + 'static,
    {
        let cache = self.clone();
        let key = key.to_string();
        use_async_with_options(
            async move {
                let value = cache.get(&key);
                if value.is_some() {
                    return Ok(value);
                };
                let value = fetcher.await?;
                match &value {
                    Some(value) => cache.add_object(&key, lifetime, value.clone()),
                    None => {
                        LocalStorage::delete(&key);
                    }
                }
                Ok(value)
            },
            UseAsyncOptions::enable_auto(),
        )
    }

    pub fn cached_role(&self, uuid_or_short_name: &str) -> UseStateHandle<CacheEntry<RoleData>> {
        self.cached_object(uuid_or_short_name, "role", ROLE_CACHE_LIFETIME)
    }

    pub fn cached_puzzle(&self, uuid_or_short_name: &str) -> UseStateHandle<CacheEntry<Puzzle>> {
        self.cached_object(uuid_or_short_name, "puzzle", PUZZLE_CACHE_LIFETIME)
    }

    fn cached_object<T>(
        &self,
        uuid_or_short_name: &str,
        kind: &'static str,
        lifetime: Duration,
    ) -> UseStateHandle<CacheEntry<T>>
    where
        T: Clone + PartialEq + Debug + Serialize + DeserializeOwned + 'static,
    {
        let cache = use_context::<ObjectCache>().expect("Cache not extant!");
        let state = use_state_eq(|| CacheEntry::Pending);
        let client = use_context::<ReqwestClient>().expect("No API client");
        let key = format!("{}:{}", kind, uuid_or_short_name);
        let async_handle: UseAsyncHandle<Option<T>, crate::components::core::APIError> = {
            let api_url = use_api_url(&format!("/{}/get/{}", kind, uuid_or_short_name));
            cache.use_cached_value(&key, lifetime, async move {
                let out: Option<T> = make_api_call(client, api_url.as_str(), None, NO_BODY).await?;
                Ok(out)
            })
        };

        use_effect_with_deps(
            {
                let state = state.clone();
                move |handle: &UseAsyncHandle<Option<T>, APIError>| {
                    if !handle.loading && (handle.data.is_some() || handle.error.is_some()) {
                        gloo::console::log!(format!(
                            "Cached {} callback, handle data returned {:?}",
                            kind, handle.data
                        ));
                        match &handle.data {
                            None => {
                                gloo::console::log!(format!("handle.error: {:?}", handle.error));
                                state.set(CacheEntry::Error(format!(
                                    "{}",
                                    handle.error.as_ref().unwrap()
                                )))
                            }
                            Some(None) => state.set(CacheEntry::Missing),
                            Some(Some(value)) => state.set(CacheEntry::Value(value.clone())),
                        }
                    }

                    || ()
                }
            },
            async_handle,
        );

        state
    }
}
