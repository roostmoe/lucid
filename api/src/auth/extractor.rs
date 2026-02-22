use std::{future::Future, sync::Arc};

use axum::{extract::FromRequestParts, http::request::Parts};
use lucid_common::caller::{ApiCaller, Caller, CallerError};

use crate::{context::ApiContext, error::ApiError};

/// Extractor that provides the current Caller (authenticated or not)
/// Never fails - returns Unauthenticated if no valid auth found
pub struct Auth(pub Caller);

impl FromRequestParts<ApiContext> for Auth {
    type Rejection = std::convert::Infallible;

    fn from_request_parts(
        parts: &mut Parts,
        state: &ApiContext,
    ) -> impl Future<Output = Result<Self, Self::Rejection>> + Send {
        let auth_manager = Arc::clone(&state.auth_manager);
        async move {
            let caller = auth_manager.authenticate(parts).await;
            Ok(Auth(caller))
        }
    }
}

/// Extractor that REQUIRES authentication
/// Returns 401 if not authenticated
pub struct RequireAuth(pub Arc<dyn ApiCaller>);

impl FromRequestParts<ApiContext> for RequireAuth {
    type Rejection = ApiError;

    fn from_request_parts(
        parts: &mut Parts,
        state: &ApiContext,
    ) -> impl Future<Output = Result<Self, Self::Rejection>> + Send {
        let auth_manager = Arc::clone(&state.auth_manager);
        async move {
            let caller = auth_manager.authenticate(parts).await;
            let api_caller = caller
                .api_caller()
                .map_err(|_| ApiError::CallerError(CallerError::unauthorized(None)))?;
            Ok(RequireAuth(api_caller))
        }
    }
}
