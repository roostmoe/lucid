use std::{collections::BTreeMap, sync::Arc, time::{Instant, SystemTime}};

use lucid_common::api::error::Error;
use lucid_uuid_kinds::OrganisationIdUuid;

use crate::{authn, storage::Storage};
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
    pub authz: authz::Context,

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
    // ------------------------------------------------------------------
    // Constructors
    // ------------------------------------------------------------------

    /// Create an [`OpContext`] for an external API request scoped to a specific
    /// organisation.
    pub async fn for_external_api(
        authn: Arc<authn::Context>,
        organisation_id: OrganisationIdUuid,
        storage: Arc<dyn authz::AuthzStorage>,
        metadata_loader: impl FnOnce(&mut BTreeMap<String, String>),
    ) -> Result<Self, Error> {
        let authz = authz::Context::load_for_actor(
            authn.actor(),
            Some(organisation_id),
            storage,
        )
        .await?;

        let mut metadata = Self::metadata_for_authn(&authn);
        metadata_loader(&mut metadata);

        Ok(Self {
            authn,
            authz,
            created_instant: Instant::now(),
            created_walltime: SystemTime::now(),
            metadata,
            kind: OpKind::ExternalApiRequest,
        })
    }

    /// Create an [`OpContext`] for a fleet-level operation with no org scope
    /// (e.g. listing all organisations).
    ///
    /// Only `system_admin` users will pass permission checks for fleet-scoped
    /// permissions.
    pub async fn for_fleet_operation(
        authn: Arc<authn::Context>,
        storage: Arc<dyn authz::AuthzStorage>,
        metadata_loader: impl FnOnce(&mut BTreeMap<String, String>),
    ) -> Result<Self, Error> {
        let authz = authz::Context::load_for_actor(
            authn.actor(),
            None,
            storage,
        )
        .await?;

        let mut metadata = Self::metadata_for_authn(&authn);
        metadata_loader(&mut metadata);

        Ok(Self {
            authn,
            authz,
            created_instant: Instant::now(),
            created_walltime: SystemTime::now(),
            metadata,
            kind: OpKind::InternalApiRequest,
        })
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

    // ------------------------------------------------------------------
    // Authorisation helpers
    // ------------------------------------------------------------------

    /// Check that the actor has the given permission. Returns `Err(Forbidden)`
    /// if they don't.
    pub fn authorize(
        &self,
        permission: &authz::Permission,
    ) -> Result<(), Error> {
        self.authz.require_permission(permission)
    }

    // ------------------------------------------------------------------
    // Metadata
    // ------------------------------------------------------------------

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
        self.authz.storage()
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
