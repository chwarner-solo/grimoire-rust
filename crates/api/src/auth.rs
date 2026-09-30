use std::sync::Arc;
use std::time::{Duration, Instant};

use axum::async_trait;
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use jsonwebtoken::jwk::{AlgorithmParameters, JwkSet};
use jsonwebtoken::{decode, decode_header, DecodingKey, Validation};
use serde::Deserialize;
use tokio::sync::RwLock;
use uuid::Uuid;

use domain::shared::UserId;

// --- Configuration ---

#[derive(Debug, Clone)]
pub struct AuthConfig {
    pub jwks_uri: String,
    pub issuer: String,
    /// Audience claim expected in incoming tokens.
    pub audience: String,
}

// --- JWKS cache ---

struct CachedKeys {
    key_set: JwkSet,
    fetched_at: Instant,
}

impl CachedKeys {
    const TTL: Duration = Duration::from_secs(3600);

    fn is_stale(&self) -> bool {
        self.fetched_at.elapsed() > Self::TTL
    }
}

#[derive(Clone)]
pub struct AuthState {
    config: AuthConfig,
    cache: Arc<RwLock<Option<CachedKeys>>>,
    http: reqwest::Client,
}

impl AuthState {
    pub fn new(config: AuthConfig) -> Self {
        Self {
            config,
            cache: Arc::new(RwLock::new(None)),
            http: reqwest::Client::new(),
        }
    }

    async fn key_set(&self) -> Result<JwkSet, AuthError> {
        // Fast path — cache hit and not stale
        {
            let guard = self.cache.read().await;
            if let Some(c) = guard.as_ref() {
                if !c.is_stale() {
                    return Ok(c.key_set.clone());
                }
            }
        }
        // Slow path — fetch and cache
        let key_set: JwkSet = self
            .http
            .get(&self.config.jwks_uri)
            .send()
            .await
            .map_err(|e| AuthError::JwksFetch(e.to_string()))?
            .json()
            .await
            .map_err(|e| AuthError::JwksFetch(e.to_string()))?;

        *self.cache.write().await = Some(CachedKeys {
            key_set: key_set.clone(),
            fetched_at: Instant::now(),
        });

        Ok(key_set)
    }

    pub async fn validate(&self, token: &str) -> Result<ValidatedClaims, AuthError> {
        let header = decode_header(token).map_err(|_| AuthError::InvalidToken)?;
        let kid = header.kid.ok_or(AuthError::MissingKid)?;

        let key_set = self.key_set().await?;
        let jwk = key_set.find(&kid).ok_or(AuthError::UnknownKid)?;

        let decoding_key = match &jwk.algorithm {
            AlgorithmParameters::RSA(params) => {
                DecodingKey::from_rsa_components(&params.n, &params.e)
                    .map_err(|_| AuthError::InvalidKey)?
            }
            _ => return Err(AuthError::UnsupportedAlgorithm),
        };

        let mut validation = Validation::new(header.alg);
        validation.set_issuer(&[&self.config.issuer]);
        validation.set_audience(&[&self.config.audience]);

        let data = decode::<RawClaims>(token, &decoding_key, &validation)
            .map_err(|_| AuthError::InvalidToken)?;

        Ok(ValidatedClaims {
            user_id: user_id_from_claims(&data.claims.iss, &data.claims.sub),
            sub: data.claims.sub,
        })
    }
}

// --- JWT claims ---

#[derive(Debug, Deserialize)]
struct RawClaims {
    sub: String,
    iss: String,
}

#[derive(Debug, Clone)]
pub struct ValidatedClaims {
    pub user_id: UserId,
    pub sub: String,
}

/// Deterministic UUID from issuer + subject so any OIDC provider maps
/// to a stable UserId without a user database lookup.
fn user_id_from_claims(iss: &str, sub: &str) -> UserId {
    let input = format!("{iss}:{sub}");
    UserId::from_uuid(Uuid::new_v5(&Uuid::NAMESPACE_URL, input.as_bytes()))
}

// --- Extractor ---

/// Axum extractor — pulls the authenticated UserId from request extensions.
/// Handlers that require auth declare `AuthenticatedUser(user_id): AuthenticatedUser`.
#[derive(Debug, Clone)]
pub struct AuthenticatedUser(pub UserId);

#[async_trait]
impl<S> FromRequestParts<S> for AuthenticatedUser
where
    S: Send + Sync,
{
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<ValidatedClaims>()
            .map(|c| AuthenticatedUser(c.user_id))
            .ok_or(AuthError::MissingClaims)
    }
}

// --- Middleware ---

/// Tower middleware layer that validates the Bearer token and injects
/// `ValidatedClaims` into request extensions. Routes that don't require
/// auth are unaffected — they simply won't have the extension set.
pub async fn auth_middleware(
    axum::extract::State(auth): axum::extract::State<Arc<AuthState>>,
    mut req: axum::http::Request<axum::body::Body>,
    next: axum::middleware::Next,
) -> Response {
    let token = req
        .headers()
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "));

    if let Some(token) = token {
        match auth.validate(token).await {
            Ok(claims) => {
                req.extensions_mut().insert(claims);
            }
            Err(e) => {
                tracing::warn!(error = %e, "auth rejected");
                return StatusCode::UNAUTHORIZED.into_response();
            }
        }
    }

    next.run(req).await
}

// --- Errors ---

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("missing or malformed token")]
    InvalidToken,
    #[error("token missing kid header")]
    MissingKid,
    #[error("unknown key id")]
    UnknownKid,
    #[error("invalid signing key")]
    InvalidKey,
    #[error("unsupported key algorithm")]
    UnsupportedAlgorithm,
    #[error("failed to fetch JWKS: {0}")]
    JwksFetch(String),
    #[error("request not authenticated")]
    MissingClaims,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        StatusCode::UNAUTHORIZED.into_response()
    }
}
