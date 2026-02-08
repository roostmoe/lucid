//! Immutable, built-in role definitions.
//!
//! These roles are seeded into each organisation's `organisation_roles` table
//! when the org is created. They cannot be modified or deleted.

use super::permissions;

/// Static definition of a built-in role.
pub struct BuiltinRole {
    pub name: &'static str,
    pub display_name: &'static str,
    pub description: &'static str,
    pub permissions: &'static [&'static str],
}

/// Organisation Owner — full access to everything within the org.
pub const ORGANISATION_OWNER: BuiltinRole = BuiltinRole {
    name: "system.organisation.owner",
    display_name: "Organisation Owner",
    description: "Full access to all resources within the organisation",
    permissions: &["*"],
};

/// Organisation Admin — manage users, roles, and role bindings.
pub const ORGANISATION_ADMIN: BuiltinRole = BuiltinRole {
    name: "system.organisation.admin",
    display_name: "Organisation Administrator",
    description: "Administrative access within the organisation",
    permissions: &["users.*", "roles.*", "role_bindings.*"],
};

/// Organisation Editor — read + write on domain resources, read-only on
/// identity resources.
pub const ORGANISATION_EDITOR: BuiltinRole = BuiltinRole {
    name: "system.organisation.editor",
    display_name: "Organisation Editor",
    description: "Can create and modify resources, but cannot manage users or roles",
    permissions: &[
        permissions::users::READ,
        permissions::users::LIST,
        permissions::roles::READ,
        permissions::roles::LIST,
    ],
};

/// Organisation Viewer — read-only access.
pub const ORGANISATION_VIEWER: BuiltinRole = BuiltinRole {
    name: "system.organisation.viewer",
    display_name: "Organisation Viewer",
    description: "Read-only access to organisation resources",
    permissions: &[
        permissions::users::READ,
        permissions::users::LIST,
        permissions::roles::READ,
        permissions::roles::LIST,
        permissions::role_bindings::READ,
        permissions::role_bindings::LIST,
    ],
};

/// All built-in roles. Used for seeding and validation.
pub const BUILTIN_ROLES: &[&BuiltinRole] = &[
    &ORGANISATION_OWNER,
    &ORGANISATION_ADMIN,
    &ORGANISATION_EDITOR,
    &ORGANISATION_VIEWER,
];

/// Returns `true` if `role_name` is a built-in role.
pub fn is_builtin_role(role_name: &str) -> bool {
    BUILTIN_ROLES.iter().any(|r| r.name == role_name)
}

/// Look up a built-in role by name.
pub fn get_builtin_role(role_name: &str) -> Option<&'static BuiltinRole> {
    BUILTIN_ROLES.iter().copied().find(|r| r.name == role_name)
}
