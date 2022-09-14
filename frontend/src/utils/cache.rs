//! Cache provision for Linkdoku
//!

use std::{collections::HashMap, future::Future, time::Duration};

use js_sys::Date;
use linkdoku_common::RoleData;
use serde::{de::DeserializeOwned, Serialize};
use serde_json::Value;
use yew::prelude::*;
use yew_hooks::{use_async_with_options, use_map, UseAsyncHandle, UseAsyncOptions, UseMapHandle};

use crate::components::core::{make_api_call, use_api_url, APIError, ReqwestClient, NO_BODY};

const ROLE_CACHE_LIFETIME: Duration = Duration::from_secs(60 * 5); // Five minute cache for roles

#[derive(Properties, PartialEq)]
pub struct ObjectCacheProviderProps {
    pub children: Children,
}

#[derive(Clone, PartialEq)]
pub struct CacheEntryExpiry {
    expires: f64,
    entry: Value,
}

#[derive(Clone, PartialEq)]
pub struct ObjectCache {
    cache: UseMapHandle<String, CacheEntryExpiry>,
}

#[derive(Clone, PartialEq)]
pub enum CacheEntry<V: Clone + PartialEq> {
    Pending,
    Missing,
    Value(V),
}

#[function_component(ObjectCacheProvider)]
pub fn object_cache_provider(props: &ObjectCacheProviderProps) -> Html {
    let cache = ObjectCache {
        cache: use_map(HashMap::new()),
    };

    html! {
        <ContextProvider<ObjectCache> context={cache}>
            {props.children.clone()}
        </ContextProvider<ObjectCache>>
    }
}

impl ObjectCache {
    pub fn add_object<V: Serialize>(&self, uuid: &str, lifetime: Duration, value: V) {
        let val = serde_json::to_value(value).expect("Unable to serialize value!");
        gloo::console::log!(format!("Adding cache entry for {}", uuid));
        self.cache.insert(
            uuid.to_string(),
            CacheEntryExpiry {
                expires: Date::now() + (lifetime.as_secs_f64() * 1000.0),
                entry: val,
            },
        );
    }

    pub fn get<V: DeserializeOwned>(&self, uuid: &str) -> Option<V> {
        if let Some(value) = self.cache.current().get(uuid) {
            if Date::now() <= value.expires {
                gloo::console::log!(format!("Retrieved {} from cache", uuid));
                serde_json::from_value(value.entry.clone()).ok()
            } else {
                gloo::console::log!(format!("Entry for {} expired", uuid));
                None
            }
        } else {
            None
        }
    }

    pub fn use_cached_value<F, T, E>(
        &self,
        uuid: &str,
        lifetime: Duration,
        fetcher: F,
    ) -> UseAsyncHandle<Option<T>, E>
    where
        T: Clone + Serialize + DeserializeOwned + 'static,
        E: Clone + 'static,
        F: Future<Output = Result<Option<T>, E>> + 'static,
    {
        let cache = self.clone();
        let uuid = uuid.to_string();
        use_async_with_options(
            async move {
                let value = cache.get(&uuid);
                if value.is_some() {
                    return Ok(value);
                };
                let value = fetcher.await?;
                match &value {
                    Some(value) => cache.add_object(&uuid, lifetime, value),
                    None => {
                        cache.cache.remove(&uuid);
                    }
                }
                Ok(value)
            },
            UseAsyncOptions::enable_auto(),
        )
    }

    pub fn cached_role(&self, uuid: &str) -> UseStateHandle<CacheEntry<RoleData>> {
        let lifetime = ROLE_CACHE_LIFETIME;
        let cache = use_context::<ObjectCache>().expect("Cache not extant!");
        let state = use_state_eq(|| CacheEntry::Pending);
        let client = use_context::<ReqwestClient>().expect("No API client");
        let async_handle: UseAsyncHandle<Option<RoleData>, crate::components::core::APIError> = {
            let api_url = use_api_url(&format!("/role/get/{}", uuid));
            cache.use_cached_value(uuid, lifetime, async move {
                let out: Option<RoleData> =
                    make_api_call(client, api_url.as_str(), None, NO_BODY).await?;
                Ok(out)
            })
        };

        use_effect_with_deps(
            {
                let state = state.clone();
                move |handle: &UseAsyncHandle<Option<RoleData>, APIError>| {
                    if !handle.loading {
                        match &handle.data {
                            Some(None) | None => state.set(CacheEntry::Missing),
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
