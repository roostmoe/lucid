mod builtin_roles;
pub mod permissions;

pub use builtin_roles::{BuiltinRole, BUILTIN_ROLES, is_builtin_role, get_builtin_role};

use std::collections::HashSet;
use std::sync::Arc;

use lucid_common::api::error::Error;
use lucid_uuid_kinds::{OrganisationIdUuid, UserIdUuid};

use crate::authn;

// ---------------------------------------------------------------------------
// Permission
// ---------------------------------------------------------------------------

/// A permission string in the format `"resource.action"`.
///
/// Supports exact matching, resource-level wildcards (`"users.*"`), and a
/// system wildcard (`"*"`) that grants every permission on every resource.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Permission(String);

impl Permission {
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Returns the resource prefix, e.g. `"users"` from `"users.create"`.
    pub fn resource_prefix(&self) -> Option<&str> {
        let dot = self.0.find('.')?;
        Some(&self.0[..dot])
    }

    /// Fleet-scoped permissions require the `system_admin` flag on the user
    /// rather than org-level role bindings.
    pub fn is_fleet_scoped(&self) -> bool {
        self.0.starts_with("organisations.") || self.0.starts_with("system.")
    }
}

impl std::fmt::Display for Permission {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

// ---------------------------------------------------------------------------
// Action
// ---------------------------------------------------------------------------

/// The kind of operation being performed against a resource.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Action {
    Read,
    Create,
    Modify,
    Delete,
    List,
}

// ---------------------------------------------------------------------------
// Storage trait
// ---------------------------------------------------------------------------

/// Trait that the datastore must implement to supply permission data during
/// [`Context`] construction.
///
/// These queries run *without* authorisation checks — they **are** the
/// authorisation setup.
#[async_trait::async_trait]
pub trait AuthzStorage: Send + Sync {
    /// Load every permission the user has in the given organisation (by
    /// flattening all their role bindings → role → permissions).
    async fn permissions_for_user_in_org(
        &self,
        user_id: UserIdUuid,
        organisation_id: OrganisationIdUuid,
    ) -> Result<HashSet<String>, Error>;

    /// Return `true` if the user is a member of the organisation.
    async fn user_is_member_of_org(
        &self,
        user_id: UserIdUuid,
        organisation_id: OrganisationIdUuid,
    ) -> Result<bool, Error>;

    /// Return `true` if the user has the `system_admin` flag set.
    async fn user_is_system_admin(
        &self,
        user_id: UserIdUuid,
    ) -> Result<bool, Error>;
}

// ---------------------------------------------------------------------------
// Context
// ---------------------------------------------------------------------------

/// Authorisation context, constructed once per [`OpContext`] and cached for the
/// lifetime of the operation.
///
/// Holds the actor, their loaded permissions, and a reference to the storage
/// backend so that sub-contexts can be created if needed.
#[derive(Clone)]
pub struct Context {
    actor: Option<authn::Actor>,
    organisation_id: Option<OrganisationIdUuid>,
    permissions: HashSet<Permission>,
    is_system_admin: bool,
    storage: Arc<dyn AuthzStorage>,
}

impl Context {
    // ------------------------------------------------------------------
    // Construction
    // ------------------------------------------------------------------

    /// Build an authz context for a given actor.
    ///
    /// * `organisation_id` — if `Some`, permissions are loaded for the actor in
    ///   that org.  If `None`, this is a fleet-level operation and only the
    ///   `system_admin` flag matters.
    pub async fn load_for_actor(
        actor: Option<&authn::Actor>,
        organisation_id: Option<OrganisationIdUuid>,
        storage: Arc<dyn AuthzStorage>,
    ) -> Result<Self, Error> {
        let Some(actor) = actor else {
            return Ok(Self {
                actor: None,
                organisation_id: None,
                permissions: HashSet::new(),
                is_system_admin: false,
                storage,
            });
        };

        let user_id = actor.user_id().ok_or_else(|| Error::Internal {
            internal_message: "actor has no user_id".into(),
        })?;

        // Check system_admin flag.
        let is_system_admin = storage.user_is_system_admin(user_id).await?;

        // If an org context was requested, verify membership and load
        // permissions.
        let permissions = if let Some(org_id) = organisation_id {
            let is_member = storage
                .user_is_member_of_org(user_id, org_id)
                .await?;

            if !is_member {
                return Err(Error::Forbidden {
                    internal_message: format!(
                        "user {} is not a member of organisation {}",
                        user_id, org_id,
                    ),
                    required_permission: None,
                });
            }

            storage
                .permissions_for_user_in_org(user_id, org_id)
                .await?
                .into_iter()
                .map(Permission::new)
                .collect()
        } else {
            HashSet::new()
        };

        Ok(Self {
            actor: Some(*actor),
            organisation_id,
            permissions,
            is_system_admin,
            storage,
        })
    }

    // ------------------------------------------------------------------
    // Permission checking
    // ------------------------------------------------------------------

    /// Returns `true` if the loaded permission set satisfies `required`.
    pub fn has_permission(&self, required: &Permission) -> bool {
        // System admins implicitly have every fleet-scoped permission.
        if self.is_system_admin && required.is_fleet_scoped() {
            return true;
        }

        // Exact match.
        if self.permissions.contains(required) {
            return true;
        }

        self.check_wildcard_match(required)
    }

    /// Check resource-level wildcards (`users.*`) and the system wildcard
    /// (`*`).
    fn check_wildcard_match(&self, required: &Permission) -> bool {
        // System wildcard — grants everything.
        if self.permissions.contains(&Permission::new("*")) {
            return true;
        }

        // Resource wildcard — e.g. `users.*` matches `users.create`.
        if let Some(resource) = required.resource_prefix() {
            let wildcard = Permission::new(format!("{}.*", resource));
            if self.permissions.contains(&wildcard) {
                return true;
            }
        }

        false
    }

    /// Return `Ok(())` if the permission is satisfied, or `Err(Forbidden)`.
    pub fn require_permission(
        &self,
        permission: &Permission,
    ) -> Result<(), Error> {
        if self.has_permission(permission) {
            Ok(())
        } else {
            Err(Error::Forbidden {
                internal_message: format!(
                    "actor {:?} lacks permission: {}",
                    self.actor,
                    permission,
                ),
                required_permission: Some(permission.as_str().to_string()),
            })
        }
    }

    // ------------------------------------------------------------------
    // Accessors
    // ------------------------------------------------------------------

    pub fn organisation_id(&self) -> Option<OrganisationIdUuid> {
        self.organisation_id
    }

    pub fn is_system_admin(&self) -> bool {
        self.is_system_admin
    }

    pub fn actor(&self) -> Option<&authn::Actor> {
        self.actor.as_ref()
    }

    pub fn storage(&self) -> &Arc<dyn AuthzStorage> {
        &self.storage
    }
}

impl std::fmt::Debug for Context {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("authz::Context")
            .field("actor", &self.actor)
            .field("organisation_id", &self.organisation_id)
            .field("permissions", &self.permissions.len())
            .field("is_system_admin", &self.is_system_admin)
            .finish()
    }
}
