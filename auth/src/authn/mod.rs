use std::fmt::Debug;

use lucid_uuid_kinds::UserIdUuid;
use serde::{Deserialize, Serialize};

// pub mod external;  // TODO: remove or rewrite for JWT
pub mod jwt;
pub mod oidc;

pub use jwt::{Claims, JwtConfig, JwtManager};
pub use oidc::{OidcClient, OidcConfig, OidcUserInfo};

/// Who is performing an operation?
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum Actor {
    /// Authenticated user
    User { user_id: UserIdUuid },
    /// Not authenticated
    Unauthenticated,
}

impl Actor {
    pub fn user_id(&self) -> Option<UserIdUuid> {
        match self {
            Actor::User { user_id } => Some(*user_id),
            Actor::Unauthenticated => None,
        }
    }

    pub fn is_authenticated(&self) -> bool {
        matches!(self, Actor::User { .. })
    }
}

/// Describes how the actor performing the current operation is authenticated
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Context {
    /// The actor performing this operation
    actor: Actor,

    /// List of authentication schemes tried.
    schemes_tried: Vec<SchemeName>,
}

impl Context {
    /// Create a context for an authenticated user
    pub fn user(user_id: UserIdUuid) -> Self {
        Self {
            actor: Actor::User { user_id },
            schemes_tried: vec![],
        }
    }

    /// Create a context for an unauthenticated request
    pub fn unauthenticated() -> Self {
        Self {
            actor: Actor::Unauthenticated,
            schemes_tried: vec![],
        }
    }

    /// Get the actor for this context
    pub fn actor(&self) -> &Actor {
        &self.actor
    }

    /// Get the actor, or error if not authenticated
    pub fn actor_required(&self) -> Result<&Actor, lucid_common::api::error::Error> {
        match &self.actor {
            Actor::User { .. } => Ok(&self.actor),
            Actor::Unauthenticated => Err(lucid_common::api::error::Error::Unauthenticated {
                internal_message: "Actor required".to_string(),
            }),
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
        if self.actor.is_authenticated() {
            self.schemes_tried().last()
        } else {
            None
        }
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
    },
}

impl From<Error> for dropshot::HttpError {
    fn from(authn_error: Error) -> Self {
        match authn_error.reason {
            e @ Reason::BadFormat { .. } => {
                dropshot::HttpError::for_bad_request(None, format!("{:#}", e))
            }

            e @ Reason::UnknownActor { .. } | e @ Reason::BadCredentials { .. } => {
                dropshot::HttpError::from(lucid_common::api::error::Error::Unauthenticated {
                    internal_message: format!("{:#}", e),
                })
            }

            Reason::UnknownError { source } => source.into(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use lucid_uuid_kinds::GenericUuid;

    fn test_user_id() -> UserIdUuid {
        use uuid::Uuid;
        UserIdUuid::from_untyped_uuid(
            Uuid::parse_str("01234567-89ab-cdef-0123-456789abcdef").unwrap(),
        )
    }

    #[test]
    fn test_actor_user_creation() {
        let user_id = test_user_id();
        let actor = Actor::User { user_id };

        assert_eq!(actor.user_id(), Some(user_id));
        assert!(actor.is_authenticated());
    }

    #[test]
    fn test_actor_unauthenticated() {
        let actor = Actor::Unauthenticated;

        assert_eq!(actor.user_id(), None);
        assert!(!actor.is_authenticated());
    }

    #[test]
    fn test_context_user() {
        let user_id = test_user_id();
        let ctx = Context::user(user_id);

        assert_eq!(ctx.actor().user_id(), Some(user_id));
        assert!(ctx.actor_required().is_ok());
        assert_eq!(ctx.actor_required().unwrap().user_id(), Some(user_id));
    }

    #[test]
    fn test_context_unauthenticated() {
        let ctx = Context::unauthenticated();

        assert_eq!(ctx.actor().user_id(), None);
        let result = ctx.actor_required();
        assert!(result.is_err());
    }

    #[test]
    fn test_actor_required_returns_error_for_unauthenticated() {
        let ctx = Context::unauthenticated();
        let result = ctx.actor_required();

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(
            err,
            lucid_common::api::error::Error::Unauthenticated { .. }
        ));
    }
}
