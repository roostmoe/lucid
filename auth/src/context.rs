use std::{
    collections::BTreeMap,
    sync::Arc,
    time::{Instant, SystemTime},
};

use lucid_common::api::error::Error;
use lucid_uuid_kinds::UserIdUuid;

use crate::authn;

// ---------------------------------------------------------------------------
// OpContext
// ---------------------------------------------------------------------------

/// Operational context threaded through every datastore call.
///
/// Carries the authenticated actor and timing metadata for the operation.
/// No more authz or organisation stuff — just simple user identity.
pub struct OpContext {
    pub authn: Arc<authn::Context>,

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
    pub fn new(
        authn: Arc<authn::Context>,
        metadata_loader: impl FnOnce(&mut BTreeMap<String, String>),
        kind: OpKind,
    ) -> Self {
        let created_instant = Instant::now();
        let created_walltime = SystemTime::now();
        let mut metadata = OpContext::metadata_for_authn(&authn);
        metadata_loader(&mut metadata);

        OpContext {
            authn,
            created_instant,
            created_walltime,
            metadata,
            kind,
        }
    }

    /// Create an [`OpContext`] for tests.
    pub fn for_test(authn: Arc<authn::Context>) -> Self {
        Self {
            authn,
            created_instant: Instant::now(),
            created_walltime: SystemTime::now(),
            metadata: BTreeMap::new(),
            kind: OpKind::Test,
        }
    }

    fn metadata_for_authn(authn: &authn::Context) -> BTreeMap<String, String> {
        let mut metadata = BTreeMap::new();

        metadata.insert(
            "authenticated".to_string(),
            authn.actor().is_authenticated().to_string(),
        );
        metadata.insert("actor".to_string(), format!("{:?}", authn.actor()));

        metadata
    }

    pub fn load_request_metadata<C: Send + Sync + 'static>(
        rqctx: &dropshot::RequestContext<C>,
        metadata: &mut BTreeMap<String, String>,
    ) {
        let request = &rqctx.request;
        metadata.insert(String::from("request_id"), rqctx.request_id.clone());
        metadata.insert(String::from("http_method"), request.method().to_string());
        metadata.insert(String::from("http_uri"), request.uri().to_string());
    }

    // ------------------------------------------------------------------
    // Accessors
    // ------------------------------------------------------------------

    pub fn actor(&self) -> &authn::Actor {
        self.authn.actor()
    }

    /// Get the user ID, or error if not authenticated
    pub fn user_id(&self) -> Result<UserIdUuid, Error> {
        self.actor()
            .user_id()
            .ok_or_else(|| Error::Unauthenticated {
                internal_message: "user ID required".to_string(),
            })
    }

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
}

impl Clone for OpContext {
    fn clone(&self) -> Self {
        Self {
            authn: Arc::clone(&self.authn),
            created_instant: self.created_instant,
            created_walltime: self.created_walltime,
            metadata: self.metadata.clone(),
            kind: self.kind,
        }
    }
}
