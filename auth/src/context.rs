use std::{collections::BTreeMap, fmt::Debug, sync::Arc, time::{Instant, SystemTime}};

use lucid_common::api::error::Error;

use crate::{authn, authz::AuthorizedResource, storage::Storage};
use crate::authz;

// ---------------------------------------------------------------------------
// OpContext
// ---------------------------------------------------------------------------

/// Operational context threaded through every datastore call.
///
/// Carries the authenticated actor, their loaded permissions, timing metadata,
/// and the kind of operation being performed. Authorisation checks happen here
/// — not at the HTTP layer.
pub struct OpContext {
    pub authn: Arc<authn::Context>,

    authz: authz::Context,
    created_instant: Instant,
    created_walltime: SystemTime,
    metadata: BTreeMap<String, String>,
    kind: OpKind,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OpKind {
    ExternalApiRequest,
    InternalApiRequest,
    Background,
    Test,
}

impl OpContext {
    pub fn new<E>(
        auth_fn: impl FnOnce() -> Result<(Arc<authn::Context>, authz::Context), E>,
        metadata_loader: impl FnOnce(&mut BTreeMap<String, String>),
        kind: OpKind,
    ) -> Result<Self, E> {
        let created_instant = Instant::now();
        let created_walltime = SystemTime::now();
        let (authn, authz) = auth_fn()?;
        let mut metadata = OpContext::metadata_for_authn(&authn);
        metadata_loader(&mut metadata);

        Ok(OpContext {
            authz,
            authn,
            created_instant,
            created_walltime,
            metadata,
            kind,
        })
    }

    pub async fn new_async<E>(
        auth_fut: impl std::future::Future<
            Output = Result<(Arc<authn::Context>, authz::Context), E>,
        >,
        metadata_loader: impl FnOnce(&mut BTreeMap<String, String>),
        kind: OpKind,
    ) -> Result<Self, E> {
        let created_instant = Instant::now();
        let created_walltime = SystemTime::now();
        let (authn, authz) = auth_fut.await?;
        let mut metadata = OpContext::metadata_for_authn(&authn);
        metadata_loader(&mut metadata);

        Ok(OpContext {
            authz,
            authn,
            created_instant,
            created_walltime,
            metadata,
            kind,
        })
    }

    pub async fn authorize<Resource>(
        &self,
        action: authz::Action,
        resource: &Resource,
    ) -> Result<(), Error>
    where
        Resource: AuthorizedResource + Debug + Clone,
    {
        self.authz.authorize(self, action, resource.clone()).await
    }

    /// Create an [`OpContext`] for a background job that inherits the
    /// permissions of the original triggering request.
    pub fn for_background(parent: &OpContext) -> Self {
        Self {
            authn: Arc::clone(&parent.authn),
            authz: parent.authz.clone(),
            created_instant: Instant::now(),
            created_walltime: SystemTime::now(),
            metadata: parent.metadata.clone(),
            kind: OpKind::Background,
        }
    }

    /// Create an [`OpContext`] for tests.
    ///
    /// The caller supplies a pre-built authz context so tests can control
    /// exactly what permissions are available.
    pub fn for_test(
        authn: Arc<authn::Context>,
        authz: authz::Context,
    ) -> Self {
        Self {
            authn,
            authz,
            created_instant: Instant::now(),
            created_walltime: SystemTime::now(),
            metadata: BTreeMap::new(),
            kind: OpKind::Test,
        }
    }

    fn metadata_for_authn(
        authn: &authn::Context,
    ) -> BTreeMap<String, String> {
        let mut metadata = BTreeMap::new();

        if let Some(actor) = &authn.actor() {
            metadata.insert("authenticated".to_string(), "true".to_string());
            metadata.insert("actor".to_string(), format!("{:?}", actor));
        } else {
            metadata.insert("authenticated".to_string(), "false".to_string());
        }

        metadata
    }

    pub fn load_request_metadata<C: Send + Sync + 'static>(
        rqctx: &dropshot::RequestContext<C>,
        metadata: &mut BTreeMap<String, String>,
    ) {
        let request = &rqctx.request;
        metadata.insert(String::from("request_id"), rqctx.request_id.clone());
        metadata
            .insert(String::from("http_method"), request.method().to_string());
        metadata.insert(String::from("http_uri"), request.uri().to_string());
    }

    // ------------------------------------------------------------------
    // Accessors
    // ------------------------------------------------------------------

    pub fn kind(&self) -> OpKind {
        self.kind
    }

    pub fn created_instant(&self) -> Instant {
        self.created_instant
    }

    pub fn created_walltime(&self) -> SystemTime {
        self.created_walltime
    }

    pub fn metadata(&self) -> &BTreeMap<String, String> {
        &self.metadata
    }

    pub fn datastore(&self) -> &Arc<dyn Storage> {
        self.authz.datastore()
    }
}

impl Clone for OpContext {
    fn clone(&self) -> Self {
        Self {
            authn: Arc::clone(&self.authn),
            authz: self.authz.clone(),
            created_instant: self.created_instant,
            created_walltime: self.created_walltime,
            metadata: self.metadata.clone(),
            kind: self.kind,
        }
    }
}
