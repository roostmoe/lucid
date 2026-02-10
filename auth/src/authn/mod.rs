use std::fmt::Debug;

use lucid_db_fixed_data::user_builtin::USER_EXTERNAL_AUTHN;
use lucid_uuid_kinds::{BuiltInUserUuid, GenericUuid, OrganisationIdUuid, UserIdUuid};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub mod external;

/// Describes how the actor performing the current operation is authenticated
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Context {
    /// Describes whether the user is authenticated and provides more
    /// information depending on which case it is.
    kind: Kind,

    /// List of authentication schemes tried.
    schemes_tried: Vec<SchemeName>,
}

impl Context {
    pub fn actor(&self) -> Option<&Actor> {
        self.actor_required().ok()
    }

    pub fn actor_required(
        &self
    ) -> Result<&Actor, lucid_common::api::error::Error> {
        match &self.kind {
            Kind::Authenticated(Details { actor, .. }) => Ok(actor),
            Kind::Unauthenticated => Err(
                lucid_common::api::error::Error::Unauthenticated {
                    internal_message: "Actor required".to_string(),
                }
            )
        }
    }

    /// Returns the ID of the credential used to authenticate, if any.
    ///
    /// For session auth, this is the session ID. For access token auth, this is
    /// the token ID. For SCIM auth, this is the SCIM token ID.
    /// Not set for spoof auth, built-in users, or unauthenticated requests.
    pub fn credential_id(&self) -> Option<Uuid> {
        match &self.kind {
            Kind::Authenticated(Details { credential_id, .. }, ..) => {
                *credential_id
            }
            Kind::Unauthenticated => None,
        }
    }

    /// Returns the list of schemes tried, in order
    ///
    /// This should generally *not* be exposed to clients.
    pub fn schemes_tried(&self) -> &[SchemeName] {
        &self.schemes_tried
    }

    /// If the user is authenticated, return the last scheme in the list of
    /// schemes tried, which is the one that worked.
    pub fn scheme_used(&self) -> Option<&SchemeName> {
        match &self.kind {
            Kind::Authenticated(..) => self.schemes_tried().last(),
            Kind::Unauthenticated => None,
        }
    }

    pub fn external_authn() -> Context {
        Context::context_for_builtin_user(USER_EXTERNAL_AUTHN.id)
    }

    fn context_for_builtin_user(user_builtin_id: BuiltInUserUuid) -> Context {
        Context {
            kind: Kind::Authenticated(
                Details {
                    actor: Actor::UserBuiltin { user_builtin_id },
                    credential_id: None,
                },
            ),
            schemes_tried: Vec::new(),
        }
    }

    /// Returns an unauthenticated context for use internally
    pub fn internal_unauthenticated() -> Context {
        Context { kind: Kind::Unauthenticated, schemes_tried: vec![] }
    }
}

// Describes whether an actor is authenticated or not and provides more
// information depending on which case it is.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum Kind {
    /// Client did not attempt to authenticate
    Unauthenticated,
    /// Client successfully authenticated
    Authenticated(Details),
}

/// Describes the actor that was authenticated
///
/// This could eventually be extended to include more information about the
/// actor, such as the time of authentication, a remote IP, etc.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Details {
    /// the actor performing the request
    pub actor: Actor,
    /// ID of the credential used to authenticate the actor. (session ID, access
    /// token ID, certificate ID, etc.).
    pub credential_id: Option<Uuid>
}

/// Who is performing an operation?
#[derive(Clone, Copy, Deserialize, Eq, PartialEq, Serialize)]
pub enum Actor {
    OrganisationUser { user_id: UserIdUuid, organisation_id: OrganisationIdUuid },
    UserBuiltin { user_builtin_id: BuiltInUserUuid },
}

impl Actor {
    pub fn user_id(&self) -> Option<UserIdUuid> {
        match self {
            Actor::OrganisationUser { user_id, .. } => Some(*user_id),
            Actor::UserBuiltin { .. } => None,
        }
    }

    pub fn organisation_id(&self) -> Option<OrganisationIdUuid> {
        match self {
            Actor::OrganisationUser { organisation_id, .. } => Some(*organisation_id),
            Actor::UserBuiltin { .. } => None,
        }
    }

    pub fn id_and_type_for_role_binding(
        &self,
    ) -> Option<(Uuid, lucid_db_models::IdentityPrincipalType)> {
        match &self {
            Actor::OrganisationUser { user_id, .. } => Some((
                user_id.into_untyped_uuid(),
                lucid_db_models::IdentityPrincipalType::User,
            )),
            Actor::UserBuiltin { user_builtin_id } => Some((
                user_builtin_id.into_untyped_uuid(),
                lucid_db_models::IdentityPrincipalType::BuiltinUser,
            )),
        }
    }
}

impl Debug for Actor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Actor::OrganisationUser { user_id, organisation_id } => f
                .debug_struct("OrganisationUser")
                .field("user_id", user_id)
                .field("organisation_id", organisation_id)
                .finish(),
            Actor::UserBuiltin { user_builtin_id } => f
                .debug_struct("UserBuiltin")
                .field("user_builtin_id", user_builtin_id)
                .finish()
        }
    }
}

#[derive(Clone, Debug)]
pub struct ConsoleSession {
    pub console_session: lucid_db_models::ConsoleSession,
    pub organisation_id: OrganisationIdUuid,
}

impl external::session_cookie::Session for ConsoleSession {
    fn id(&self) -> lucid_uuid_kinds::ConsoleSessionIdUuid {
        lucid_types::identity::Resource::id(&self.console_session)
    }

    fn user_id(&self) -> UserIdUuid {
        self.console_session.user_id.into()
    }

    fn organisation_id(&self) -> OrganisationIdUuid {
        self.organisation_id
    }

    fn last_seen_at(&self) -> chrono::DateTime<chrono::Utc> {
        self.console_session.last_seen_at
    }

    fn created_at(&self) -> chrono::DateTime<chrono::Utc> {
        lucid_types::identity::Resource::created_at(&self.console_session)
    }
}

pub use lucid_types::authn::SchemeName;

#[derive(Debug, thiserror::Error)]
#[error("authentication failed (tried schemes: {schemes_tried:?})")]
pub struct Error {
    /// list of authentication schemes that were tried
    schemes_tried: Vec<SchemeName>,

    /// why authentication failed
    #[source]
    reason: Reason,
}

#[derive(Debug, thiserror::Error)]
pub enum Reason {
    /// The authn credentials were syntactically invalid
    #[error("bad authentication credentials: {source:#}")]
    BadFormat {
        #[source]
        source: anyhow::Error,
    },

    /// We did not find the actor that was attempting to authenticate
    #[error("unknown actor {actor:?}")]
    UnknownActor { actor: String },

    /// The credentials were syntactically valid, but semantically invalid
    /// (e.g., a cryptographic signature did not match)
    #[error("bad credentials for actor {actor:?}: {source:#}")]
    BadCredentials {
        actor: Actor,
        #[source]
        source: anyhow::Error,
    },

    /// Operational error while trying to authenticate
    #[error("unexpected error during authentication: {source:#}")]
    UnknownError {
        #[source]
        source: lucid_common::api::error::Error,
    }
}

impl From<Error> for dropshot::HttpError {
    fn from(authn_error: Error) -> Self {
        match authn_error.reason {
            e @ Reason::BadFormat { .. } => {
                dropshot::HttpError::for_bad_request(None, format!("{:#}", e))
            }

            e @ Reason::UnknownActor { .. }
            | e @ Reason::BadCredentials { .. } => dropshot::HttpError::from(
                lucid_common::api::error::Error::Unauthenticated {
                    internal_message: format!("{:#}", e)
                },
            ),
            
            Reason::UnknownError { source } => source.into(),
        }
    }
}
