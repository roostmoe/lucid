//! Authentication and authorization primitives.
//!
//! This module provides Lucid's Role-Based Access Control (RBAC) system through
//! the [`Caller`], [`Role`], and [`Permission`] types.
//!
//! # Overview
//!
//! - **[`Caller`]**: Who is making the request (user, agent, service account, system)
//! - **[`Role`]**: What level of access they have (Admin, Viewer, etc.)
//! - **[`Permission`]**: What specific actions they can perform (read, write, delete)
//!
//! # Quick Start
//!
//! ```
//! use lucid_common::caller::{Caller, Role, Permission};
//!
//! let caller = Caller::User {
//!     id: "user123".into(),
//!     display_name: "Alice".into(),
//!     email: "alice@example.com".into(),
//!     roles: vec![Role::Viewer],
//! };
//!
//! // Check permissions
//! if caller.can(Permission::HostsRead) {
//!     println!("Can view hosts");
//! }
//!
//! // Require permissions (fails with error if missing)
//! caller.require(Permission::HostsWrite)?;
//! # Ok::<(), lucid_common::caller::CallerError>(())
//! ```
//!
//! # See Also
//!
//! For detailed documentation on how authentication and authorization work in Lucid,
//! see `docs/ARCHITECTURE_AUTH.adoc` in the repository root.

use std::fmt::{self, Display};
use thiserror::Error;

/// Fine-grained permissions for Lucid's RBAC system.
///
/// Permissions are atomic capabilities that control access to specific operations.
/// They're grouped by resource type (hosts, users, service accounts) and action
/// (read, write, delete).
///
/// # Examples
///
/// ```
/// use lucid_common::caller::Permission;
///
/// // Check if a permission allows reading
/// match Permission::HostsRead {
///     Permission::HostsRead | Permission::UsersRead => println!("read-only"),
///     _ => println!("write or delete"),
/// }
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Permission {
    /// View host inventory, metadata, and telemetry
    HostsRead,
    /// Create and update hosts
    HostsWrite,
    /// Delete hosts from inventory
    HostsDelete,

    /// View user profiles and roles
    UsersRead,
    /// Create and update users
    UsersWrite,
    /// Delete user accounts
    UsersDelete,

    /// View service account details
    ServiceAccountsRead,
    /// Create and update service accounts
    ServiceAccountsWrite,
    /// Delete service accounts
    ServiceAccountsDelete,
}

impl Permission {
    pub fn as_str(&self) -> &'static str {
        match self {
            Permission::HostsRead => "hosts:read",
            Permission::HostsWrite => "hosts:write",
            Permission::HostsDelete => "hosts:delete",
            Permission::UsersRead => "users:read",
            Permission::UsersWrite => "users:write",
            Permission::UsersDelete => "users:delete",
            Permission::ServiceAccountsRead => "service_accounts:read",
            Permission::ServiceAccountsWrite => "service_accounts:write",
            Permission::ServiceAccountsDelete => "service_accounts:delete",
        }
    }
}

/// Roles bundle permissions together for easier assignment.
///
/// Instead of assigning individual permissions, you assign roles to callers.
/// Each role grants a curated set of permissions appropriate for that access level.
///
/// # Available Roles
///
/// - **Admin**: Full access to all resources and operations
/// - **Viewer**: Read-only access to hosts, users, and service accounts
///
/// # Examples
///
/// ```
/// use lucid_common::caller::{Role, Permission};
///
/// let admin = Role::Admin;
/// assert!(admin.has_permission(Permission::HostsDelete));
///
/// let viewer = Role::Viewer;
/// assert!(viewer.has_permission(Permission::HostsRead));
/// assert!(!viewer.has_permission(Permission::HostsWrite));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    /// Full administrative access - all permissions granted
    Admin,
    /// Read-only access to all resources
    Viewer,
}

impl Role {
    pub fn permissions(&self) -> &'static [Permission] {
        match self {
            Role::Admin => &[
                Permission::HostsRead,
                Permission::HostsWrite,
                Permission::HostsDelete,
                Permission::UsersRead,
                Permission::UsersWrite,
                Permission::UsersDelete,
                Permission::ServiceAccountsRead,
                Permission::ServiceAccountsWrite,
                Permission::ServiceAccountsDelete,
            ],
            Role::Viewer => &[
                Permission::HostsRead,
                Permission::UsersRead,
                Permission::ServiceAccountsRead,
            ],
        }
    }

    pub fn has_permission(&self, permission: Permission) -> bool {
        self.permissions().contains(&permission)
    }
}

/// Authenticated identity that can make API requests.
///
/// `Caller` represents who is making a request and what they're allowed to do.
/// All API operations receive a `Caller` and check permissions before proceeding.
///
/// # Variants
///
/// - **User**: Human user authenticated via session token
/// - **Agent**: Host agent reporting telemetry (future: agent-specific permissions)
/// - **ServiceAccount**: API token for automation/integrations
/// - **System**: Internal operations with unrestricted access
///
/// # Permission Checking
///
/// Use [`can()`](Caller::can) to check permissions without failing:
/// ```
/// # use lucid_common::caller::{Caller, Permission, Role};
/// let caller = Caller::User {
///     id: "user123".into(),
///     display_name: "Alice".into(),
///     email: "alice@example.com".into(),
///     roles: vec![Role::Viewer],
/// };
///
/// if caller.can(Permission::HostsRead) {
///     // fetch hosts
/// }
/// ```
///
/// Use [`require()`](Caller::require) to enforce permissions and fail with CallerError:
/// ```
/// # use lucid_common::caller::{Caller, Permission, Role};
/// # let caller = Caller::User {
/// #     id: "user123".into(),
/// #     display_name: "Alice".into(),
/// #     email: "alice@example.com".into(),
/// #     roles: vec![Role::Admin],
/// # };
/// caller.require(Permission::HostsWrite)?; // fails if missing permission
/// // proceed with write operation
/// # Ok::<(), lucid_common::caller::CallerError>(())
/// ```
///
/// # Creating Callers
///
/// Callers are typically created by:
/// - Auth extractors (from session tokens, API keys, etc.)
/// - Database models via `DbUser::to_caller()`
/// - System-level operations using `Caller::System`
#[derive(Debug, Clone)]
pub enum Caller {
    User {
        id: String,
        display_name: String,
        email: String,
        roles: Vec<Role>,
    },
    Agent {
        id: String,
        name: String,
        roles: Vec<Role>,
    },
    System,
    ServiceAccount {
        id: String,
        name: String,
        description: Option<String>,
        roles: Vec<Role>,
    },
}

impl Caller {
    pub fn id(&self) -> &str {
        match self {
            Caller::User { id, .. }
            | Caller::Agent { id, .. }
            | Caller::ServiceAccount { id, .. } => id,
            Caller::System => "system",
        }
    }

    pub fn display_name(&self) -> Option<&str> {
        match self {
            Caller::User { display_name, .. } => Some(display_name),
            Caller::Agent { name, .. } => Some(name),
            Caller::ServiceAccount { name, .. } => Some(name),
            Caller::System => None,
        }
    }

    pub fn kind(&self) -> &'static str {
        match self {
            Caller::User { .. } => "user",
            Caller::Agent { .. } => "agent",
            Caller::System => "system",
            Caller::ServiceAccount { .. } => "service_account",
        }
    }

    pub fn has_role(&self, role: Role) -> bool {
        match self {
            Caller::User { roles, .. }
            | Caller::Agent { roles, .. }
            | Caller::ServiceAccount { roles, .. } => roles.contains(&role),
            Caller::System => true,
        }
    }

    /// Check if caller has a specific permission without failing.
    ///
    /// Returns `true` if the caller's roles include this permission.
    /// System callers always return `true`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use lucid_common::caller::{Caller, Permission};
    /// let caller = Caller::System;
    /// assert!(caller.can(Permission::HostsDelete));
    /// ```
    pub fn can(&self, permission: Permission) -> bool {
        match self {
            Caller::System => true,
            Caller::User { roles, .. }
            | Caller::Agent { roles, .. }
            | Caller::ServiceAccount { roles, .. } => {
                roles.iter().any(|r| r.has_permission(permission))
            }
        }
    }

    /// Require a permission or return an error.
    ///
    /// Use this at the start of operations that need specific permissions.
    /// Returns `Ok(())` if allowed, `Err(CallerError::Forbidden)` if not.
    ///
    /// # Examples
    ///
    /// ```
    /// # use lucid_common::caller::{Caller, Permission, Role};
    /// # let caller = Caller::User {
    /// #     id: "user123".into(),
    /// #     display_name: "Alice".into(),
    /// #     email: "alice@example.com".into(),
    /// #     roles: vec![Role::Viewer],
    /// # };
    /// // This will fail because Viewer doesn't have write permission
    /// let result = caller.require(Permission::HostsWrite);
    /// assert!(result.is_err());
    /// ```
    pub fn require(&self, permission: Permission) -> Result<(), CallerError> {
        if self.can(permission) {
            Ok(())
        } else {
            Err(CallerError::forbidden(permission.as_str()))
        }
    }

    /// Require a specific role or return an error.
    ///
    /// Less common than permission checks, but useful when you need
    /// to restrict operations to specific roles rather than individual permissions.
    ///
    /// # Examples
    ///
    /// ```
    /// # use lucid_common::caller::{Caller, Role};
    /// let caller = Caller::System;
    /// assert!(caller.require_role(Role::Admin).is_ok()); // System has all roles
    /// ```
    pub fn require_role(&self, role: Role) -> Result<(), CallerError> {
        if self.has_role(role) {
            Ok(())
        } else {
            Err(CallerError::Forbidden {
                permission: format!("role:{:?}", role),
            })
        }
    }
}

impl Display for Caller {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Caller::User {
                id, display_name, ..
            } => {
                write!(f, "User({id}, {display_name})")
            }
            Caller::Agent { id, name, .. } => {
                write!(f, "Agent({id}, {name})")
            }
            Caller::System => write!(f, "System"),
            Caller::ServiceAccount { id, name, .. } => {
                write!(f, "ServiceAccount({id}, {name})")
            }
        }
    }
}

/// Errors that occur during authentication or authorization.
#[derive(Debug, Error)]
pub enum CallerError {
    /// Authentication failed - invalid or missing credentials
    #[error("Unauthorized: {reason}")]
    Unauthorized { reason: String },

    /// Authorization failed - authenticated but lacks permission
    #[error("Missing permission: {permission}")]
    Forbidden { permission: String },

    /// Catch-all for unexpected errors
    #[error("An unspecified error occurred: {0}")]
    Anyhow(#[from] anyhow::Error),
}

impl CallerError {
    pub fn unauthorized(reason: Option<String>) -> Self {
        Self::Unauthorized {
            reason: reason.unwrap_or_else(|| "No reason provided".to_string()),
        }
    }

    pub fn forbidden(permission: &str) -> Self {
        Self::Forbidden {
            permission: permission.into(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn test_user() -> Caller {
        Caller::User {
            id: "user123".to_string(),
            display_name: "Test User".to_string(),
            email: "test@example.com".to_string(),
            roles: vec![Role::Viewer],
        }
    }

    fn test_admin() -> Caller {
        Caller::User {
            id: "admin456".to_string(),
            display_name: "Admin User".to_string(),
            email: "admin@example.com".to_string(),
            roles: vec![Role::Admin],
        }
    }

    #[test]
    fn caller_id_returns_correct_value() {
        let caller = test_user();
        assert_eq!(caller.id(), "user123");
        assert_eq!(Caller::System.id(), "system");
    }

    #[test]
    fn caller_display_name_returns_correct_value() {
        let caller = test_user();
        assert_eq!(caller.display_name(), Some("Test User"));
        assert_eq!(Caller::System.display_name(), None);
    }

    #[test]
    fn caller_kind_returns_correct_string() {
        assert_eq!(test_user().kind(), "user");
        assert_eq!(Caller::System.kind(), "system");
    }

    #[test]
    fn viewer_can_read_but_not_write() {
        let caller = test_user();
        assert!(caller.can(Permission::HostsRead));
        assert!(!caller.can(Permission::HostsWrite));
        assert!(!caller.can(Permission::HostsDelete));
    }

    #[test]
    fn admin_can_do_everything() {
        let caller = test_admin();
        assert!(caller.can(Permission::HostsRead));
        assert!(caller.can(Permission::HostsWrite));
        assert!(caller.can(Permission::HostsDelete));
        assert!(caller.can(Permission::UsersRead));
        assert!(caller.can(Permission::UsersWrite));
    }

    #[test]
    fn system_can_do_everything() {
        let caller = Caller::System;
        assert!(caller.can(Permission::HostsRead));
        assert!(caller.can(Permission::HostsWrite));
        assert!(caller.can(Permission::HostsDelete));
        assert!(caller.can(Permission::ServiceAccountsDelete));
    }

    #[test]
    fn require_fails_on_missing_permission() {
        let caller = test_user();
        assert!(caller.require(Permission::HostsWrite).is_err());
    }

    #[test]
    fn require_succeeds_on_present_permission() {
        let caller = test_user();
        assert!(caller.require(Permission::HostsRead).is_ok());
    }

    #[test]
    fn has_role_works_correctly() {
        let viewer = test_user();
        let admin = test_admin();

        assert!(viewer.has_role(Role::Viewer));
        assert!(!viewer.has_role(Role::Admin));
        assert!(admin.has_role(Role::Admin));
        assert!(Caller::System.has_role(Role::Admin));
    }

    #[test]
    fn display_formats_correctly() {
        let caller = test_user();
        assert_eq!(format!("{}", caller), "User(user123, Test User)");
        assert_eq!(format!("{}", Caller::System), "System");
    }
}
