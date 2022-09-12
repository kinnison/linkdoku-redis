//! Login related work
//!

/*

OpenID Connect / OAuth login flow is fairly standardised.

To achieve it we have to create and store nonces in cookies
and similar things though.

For now our design assumes the google OIDC provider and
we don't support others.  We might change that later.

*/

use std::collections::HashMap;

use axum::{
    extract::{Path, Query},
    http::StatusCode,
    routing::get,
    Extension, Json, Router,
};
use cookie::SameSite;
use lazy_static::lazy_static;
use linkdoku_common::{BackendLoginStatus, LoginFlowResult, LoginFlowStart};
use openidconnect::{
    core::{CoreAuthenticationFlow, CoreClient, CoreProviderMetadata},
    reqwest::async_http_client,
    url::Url,
    AuthorizationCode, ClientId, ClientSecret, CsrfToken, IssuerUrl, Nonce, PkceCodeChallenge,
    PkceCodeVerifier, RedirectUrl, Scope, TokenResponse,
};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use tower_cookies::{Cookie, Cookies, Key};
use tracing::instrument;

use crate::{
    config::Configuration,
    dbconn::{Database, Identity},
};

struct ProviderSetup {
    client_id: String,
    client_secret: String,
    provider_metadata: CoreProviderMetadata,
    scopes: Vec<Scope>,
}

async fn load_providers(providers: &mut HashMap<String, ProviderSetup>, config: &Configuration) {
    for (name, oidp) in config.openid.iter() {
        tracing::info!("Loading OIDC metdata for {} from config", name);
        let provider_metadata = CoreProviderMetadata::discover_async(
            IssuerUrl::new(oidp.discovery_doc.clone()).expect("Unable to grok discovery_doc url"),
            async_http_client,
        )
        .await;
        if let Ok(provider_metadata) = provider_metadata {
            tracing::info!("Loaded openid connect provider {}", name);
            let client_id = oidp.client_id.clone();
            let client_secret = oidp.client_secret.clone();
            let scopes = oidp
                .scopes
                .iter()
                .map(String::clone)
                .map(Scope::new)
                .collect();
            providers.insert(
                name.to_lowercase(),
                ProviderSetup {
                    client_id,
                    client_secret,
                    provider_metadata,
                    scopes,
                },
            );
        }
    }
}

lazy_static! {
    static ref REDIRECT_URL: Mutex<String> = Mutex::new(String::new());
    static ref PROVIDERS: Mutex<HashMap<String, ProviderSetup>> = Mutex::new(HashMap::new());
    static ref LOGIN_KEY: Mutex<Key> = Mutex::new(Key::generate());
}

#[derive(Debug, Serialize, Deserialize)]
struct LoginFlowSetup {
    provider: String,
    pkce_verifier: PkceCodeVerifier,
    url: Url,
    csrf_token: CsrfToken,
    nonce: Nonce,
}

#[derive(Serialize, Deserialize)]
struct LoginFlowUserData {
    identity: Identity,
    cached_roles: Vec<String>,
    active_role: String,
}

#[derive(Serialize, Deserialize, Default)]
struct LoginFlowStatus {
    flow: Option<LoginFlowSetup>,
    user: Option<LoginFlowUserData>,
}

async fn login_flow_status(cookies: &Cookies) -> LoginFlowStatus {
    serde_json::from_str(
        &cookies
            .private(&*LOGIN_KEY.lock().await)
            .get("login")
            .map(|c| c.value().to_owned())
            .unwrap_or_default(),
    )
    .unwrap_or_default()
}

async fn set_login_flow_status(cookies: &Cookies, login: &LoginFlowStatus) {
    cookies.private(&*LOGIN_KEY.lock().await).add(
        Cookie::build(
            "login",
            serde_json::to_string(login).expect("Unable to serialise login"),
        )
        .path("/")
        .same_site(SameSite::Lax)
        .finish(),
    );
}

async fn start_auth(Path(provider): Path<String>, cookies: Cookies) -> Json<LoginFlowStart> {
    let mut flow = login_flow_status(&cookies).await;
    // First up, if we're already logged in, just redirect the user to the root of the app
    if flow.user.is_some() {
        return Json::from(LoginFlowStart::Idle);
    }
    if let Some(setup) = flow.flow.as_ref() {
        if setup.provider == provider {
            // We already have a login flow in progress, so redirect the user again
            return Json::from(LoginFlowStart::Redirect(setup.url.to_string()));
        }
    }
    if let Some(provider_data) = PROVIDERS.lock().await.get(&provider) {
        // Either no flow in progress, or user is trying a different flow for whatever reason
        let client = CoreClient::from_provider_metadata(
            provider_data.provider_metadata.clone(),
            ClientId::new(provider_data.client_id.clone()),
            Some(ClientSecret::new(provider_data.client_secret.clone())),
        )
        .set_redirect_uri(RedirectUrl::new(REDIRECT_URL.lock().await.clone()).unwrap());

        let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();
        let (url, csrf_token, nonce) = {
            let mut actor = client.authorize_url(
                CoreAuthenticationFlow::AuthorizationCode,
                CsrfToken::new_random,
                Nonce::new_random,
            );
            for scope in provider_data.scopes.iter() {
                actor = actor.add_scope(scope.clone());
            }
            actor.set_pkce_challenge(pkce_challenge).url()
        };

        flow.flow = Some(LoginFlowSetup {
            provider,
            pkce_verifier,
            url: url.clone(),
            csrf_token,
            nonce,
        });

        tracing::info!("Set up flow: {:?}", flow.flow);

        set_login_flow_status(&cookies, &flow).await;

        Json::from(LoginFlowStart::Redirect(url.to_string()))
    } else {
        // Selected provider was not available, let's go again
        Json::from(LoginFlowStart::Error(format!(
            "Provider: {} not known",
            provider
        )))
    }
}

#[derive(Deserialize)]
struct LoginContinueQuery {
    state: Option<String>,
    code: Option<String>,
    error: Option<String>,
}

async fn handle_login_continue(
    cookies: Cookies,
    Query(params): Query<LoginContinueQuery>,
    Extension(mut dbconn): Extension<Database>,
) -> Json<LoginFlowResult> {
    let mut flow = login_flow_status(&cookies).await;
    // First up, if we're already logged in, just redirect the user to the root of the app
    if flow.user.is_some() {
        return Json::from(LoginFlowResult { error: None });
    }
    if let Some(setup) = flow.flow.as_ref() {
        // Flow is in progress, so let's check the state first
        if params.state.as_ref() != Some(setup.csrf_token.secret()) {
            // State value is bad, so clean up and BAD_REQUEST
            flow.flow = None;
            set_login_flow_status(&cookies, &flow).await;
            return Json::from(LoginFlowResult {
                error: Some("bad-state".to_string()),
            });
        }
        if let Some(error) = params.error {
            tracing::error!("Error in flow: {}", error);
            flow.flow = None;
            set_login_flow_status(&cookies, &flow).await;
            return Json::from(LoginFlowResult { error: Some(error) });
        }
        let code = params.code.as_deref().unwrap();
        tracing::info!("Trying to transact code: {}", code);
        if let Some(provider_data) = PROVIDERS.lock().await.get(&setup.provider) {
            let client = CoreClient::from_provider_metadata(
                provider_data.provider_metadata.clone(),
                ClientId::new(provider_data.client_id.clone()),
                Some(ClientSecret::new(provider_data.client_secret.clone())),
            )
            .set_redirect_uri(RedirectUrl::new(REDIRECT_URL.lock().await.clone()).unwrap());
            match client
                .exchange_code(AuthorizationCode::new(code.to_string()))
                .set_pkce_verifier(PkceCodeVerifier::new(setup.pkce_verifier.secret().clone()))
                .request_async(async_http_client)
                .await
            {
                Ok(token_response) => {
                    let id_token = match token_response.id_token() {
                        Some(token) => token,
                        None => {
                            tracing::error!("Failed to get id_token");
                            flow.flow = None;
                            set_login_flow_status(&cookies, &flow).await;
                            return Json::from(LoginFlowResult {
                                error: Some("no-id-token".to_string()),
                            });
                        }
                    };
                    let claims = match id_token.claims(&client.id_token_verifier(), &setup.nonce) {
                        Ok(claims) => claims,
                        Err(e) => {
                            tracing::error!("Failed to verify id_token: {:?}", e);
                            flow.flow = None;
                            set_login_flow_status(&cookies, &flow).await;
                            return Json::from(LoginFlowResult {
                                error: Some("bad-id-token".to_string()),
                            });
                        }
                    };
                    // Okay, at this point we *are* logged in, so let's prepare our data
                    let subject = format!("{}:{}", setup.provider, claims.subject().as_str());
                    let name = claims
                        .name()
                        .and_then(|n| n.get(None).map(|n| n.to_string()));
                    let email = claims.email().map(|e| e.to_string());
                    flow.flow = None;
                    let identity = Identity::new(
                        &subject,
                        name.as_ref().unwrap_or(&subject),
                        email.as_deref(),
                    );
                    match dbconn.identity_upsert_and_roles(&identity).await {
                        Ok(mut roles) => {
                            let default_role = identity.get_default_role();
                            if !roles.iter().any(|v| v == &default_role) {
                                if let Err(e) = dbconn.create_default_role(&identity).await {
                                    tracing::error!(
                                        "Failed creating default role for identity {:?}: {:?}",
                                        identity,
                                        e
                                    );
                                    set_login_flow_status(&cookies, &flow).await;
                                    return Json::from(LoginFlowResult {
                                        error: Some("databse-error".to_string()),
                                    });
                                }
                                roles.push(default_role.clone());
                            }
                            flow.user = Some(LoginFlowUserData {
                                identity,
                                cached_roles: roles,
                                active_role: default_role,
                            });
                            set_login_flow_status(&cookies, &flow).await;
                            Json::from(LoginFlowResult { error: None })
                        }
                        Err(e) => {
                            tracing::error!("Failed upserting identity: {:?}", e);
                            set_login_flow_status(&cookies, &flow).await;
                            Json::from(LoginFlowResult {
                                error: Some("database-error".to_string()),
                            })
                        }
                    }
                }
                Err(e) => {
                    // Failed to exchange the token, return something
                    tracing::error!("Failed exchanging codes: {:?}", e);
                    flow.flow = None;
                    set_login_flow_status(&cookies, &flow).await;
                    Json::from(LoginFlowResult {
                        error: Some("code-exchange-failed".to_string()),
                    })
                }
            }
        } else {
            flow.flow = None;
            set_login_flow_status(&cookies, &flow).await;
            Json::from(LoginFlowResult {
                error: Some("bad-provider".to_string()),
            })
        }
    } else {
        // No login in progress, redirect user to root
        Json::from(LoginFlowResult { error: None })
    }
}

async fn handle_login_status(cookies: Cookies) -> Json<BackendLoginStatus> {
    let flow = login_flow_status(&cookies).await;
    if let Some(data) = flow.user {
        Json::from(BackendLoginStatus::LoggedIn {
            name: data.identity.display_name().to_string(),
            gravatar_hash: data.identity.gravatar_hash().map(String::from),
            roles: data.cached_roles.clone(),
            role: data.active_role.clone(),
        })
    } else {
        Json::from(BackendLoginStatus::LoggedOut)
    }
}

async fn handle_clear_login(cookies: Cookies) -> StatusCode {
    let mut flow = login_flow_status(&cookies).await;
    flow.flow = None;
    flow.user = None;
    set_login_flow_status(&cookies, &flow).await;
    StatusCode::NO_CONTENT
}

#[instrument(skip(config))]
pub async fn setup(config: &Configuration) {
    let mut providers = PROVIDERS.lock().await;
    tracing::info!("Loading OIDC providers...");
    load_providers(&mut providers, config).await;
    tracing::info!("Loaded {} providers", providers.len());
    *(REDIRECT_URL.lock().await) = config.redirect_url.clone();
    *(LOGIN_KEY.lock().await) = Key::derive_from(config.cookie_secret.as_bytes());
}

pub fn router() -> Router {
    Router::new()
        .route("/continue", get(handle_login_continue))
        .route("/start/:provider", get(start_auth))
        .route("/status", get(handle_login_status))
        .route("/clear", get(handle_clear_login))
}
