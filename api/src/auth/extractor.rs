use std::future::Future;
use std::sync::Arc;

use axum::{extract::FromRequestParts, http::request::Parts};
use lucid_common::caller::{Caller, CallerError};

use crate::{context::ApiContext, error::ApiError};

/// Extractor that REQUIRES authentication.
///
/// Returns 401 Unauthorized if authentication fails.
/// Use this in handlers that need a valid authenticated caller.
///
/// # Examples
///
/// ```rust,ignore
/// use lucid_api::auth::extractor::Auth;
/// use lucid_common::caller::Permission;
///
/// pub async fn delete_host(
///     Auth(caller): Auth,  // ← extracts authenticated caller
///     Path(id): Path<String>,
/// ) -> Result<(), ApiError> {
///     caller.require(Permission::HostsDelete)?;
///     // ... delete host
///     Ok(())
/// }
/// ```
///
/// For optional authentication, use `Option<Auth>`:
///
/// ```rust,ignore
/// pub async fn list_hosts(
///     auth: Option<Auth>,  // ← works with or without auth
/// ) -> Result<Json<Vec<Host>>, ApiError> {
///     if let Some(Auth(caller)) = auth {
///         // return full host data
///     } else {
///         // return public host data only
///     }
///     Ok(Json(hosts))
/// }
/// ```
pub struct Auth(pub Caller);

impl FromRequestParts<ApiContext> for Auth {
    type Rejection = ApiError;

    fn from_request_parts(
        parts: &mut Parts,
        state: &ApiContext,
    ) -> impl Future<Output = Result<Self, Self::Rejection>> + Send {
        let auth_manager = Arc::clone(&state.auth_manager);
        async move {
            let caller = auth_manager.authenticate(parts).await.map_err(|e| {
                ApiError::CallerError(CallerError::unauthorized(Some(e.to_string())))
            })?;
            Ok(Auth(caller))
        }
    }
}

/// Alias for Auth - both require authentication now
pub type RequireAuth = Auth;
