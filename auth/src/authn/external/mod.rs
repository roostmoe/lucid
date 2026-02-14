use std::borrow::Borrow;

use async_trait::async_trait;
use tracing::trace;

use crate::authn::{self, Details, Reason};

pub mod session_cookie;

pub struct Authenticator<T> {
    allowed_schemes: Vec<Box<dyn HttpAuthnScheme<T>>>,
}

impl<T> Authenticator<T>
where
    T: Send + Sync + 'static,
{
    pub fn new(allowed_schemes: Vec<Box<dyn HttpAuthnScheme<T>>>) -> Authenticator<T> {
        Authenticator { allowed_schemes }
    }

    pub async fn authn_request<Q>(
        &self,
        rqctx: &dropshot::RequestContext<Q>,
    ) -> Result<authn::Context, authn::Error>
    where
        Q: Borrow<T> + Send + Sync + 'static,
    {
        let ctx = rqctx.context().borrow();
        let request_id = rqctx.request_id.as_str();
        let result = self
            .authn_request_generic(ctx, request_id, &rqctx.request)
            .await;
        trace!("authn result: {:?}", result);
        result
    }

    /// Authenticate an incoming HTTP request (dropshot-agnostic)
    pub async fn authn_request_generic(
        &self,
        ctx: &T,
        request_id: &str,
        request: &dropshot::RequestInfo,
    ) -> Result<authn::Context, authn::Error> {
        let mut schemes_tried = Vec::with_capacity(self.allowed_schemes.len());
        for scheme_impl in &self.allowed_schemes {
            let scheme_name = scheme_impl.name();
            trace!(?request_id, "trying scheme {:?}", scheme_name);
            schemes_tried.push(scheme_name);

            let result = scheme_impl.authn(ctx, request).await;
            match result {
                SchemeResult::Failed(reason) => {
                    return Err(authn::Error {
                        reason,
                        schemes_tried,
                    });
                }

                SchemeResult::Authenticated(details) => {
                    return Ok(authn::Context {
                        kind: authn::Kind::Authenticated(details),
                        schemes_tried,
                    });
                }

                SchemeResult::NotRequested => (),
            }
        }

        Ok(authn::Context {
            kind: authn::Kind::Unauthenticated,
            schemes_tried,
        })
    }
}

#[async_trait]
pub trait HttpAuthnScheme<T>: std::fmt::Debug + Send + Sync + 'static
where
    T: Send + Sync + 'static,
{
    /// Returns the (unique) name for this scheme (for observability)
    fn name(&self) -> authn::SchemeName;

    /// Locate credentials in the HTTP request and attempt to verify them
    async fn authn(&self, ctx: &T, request: &dropshot::RequestInfo) -> SchemeResult;
}

/// Result returned by each authentication scheme when trying to authenticate a
/// request
#[derive(Debug)]
pub enum SchemeResult {
    /// The client is not trying to use this authn scheme
    NotRequested,
    /// The client successfully authenticated
    Authenticated(Details),
    /// The client tried and failed to authenticate
    Failed(Reason),
}
