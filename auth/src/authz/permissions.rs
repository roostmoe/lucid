//! Permission string constants for every resource and action.
//!
//! Convention: `"resource.action"` in snake_case.
//!
//! These are the *only* permission strings that should appear in role
//! definitions and authorisation checks. If you add a new resource, add a
//! module here.

pub mod organisations {
    pub const CREATE: &str = "organisations.create";
    pub const READ: &str = "organisations.read";
    pub const UPDATE: &str = "organisations.update";
    pub const DELETE: &str = "organisations.delete";
    pub const LIST: &str = "organisations.list";
}

pub mod users {
    pub const CREATE: &str = "users.create";
    pub const READ: &str = "users.read";
    pub const UPDATE: &str = "users.update";
    pub const DELETE: &str = "users.delete";
    pub const LIST: &str = "users.list";
}

pub mod roles {
    pub const READ: &str = "roles.read";
    pub const LIST: &str = "roles.list";
}

pub mod role_bindings {
    pub const CREATE: &str = "role_bindings.create";
    pub const READ: &str = "role_bindings.read";
    pub const UPDATE: &str = "role_bindings.update";
    pub const DELETE: &str = "role_bindings.delete";
    pub const LIST: &str = "role_bindings.list";
}
