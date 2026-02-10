use std::sync::LazyLock;

use lucid_uuid_kinds::BuiltInUserUuid;

pub struct UserBuiltinConfig {
    pub id: BuiltInUserUuid,
    pub name: String,
    pub description: &'static str,
}

impl UserBuiltinConfig {
    fn new_static(
        id: &str,
        name: &str,
        description: &'static str,
    ) -> UserBuiltinConfig {
        UserBuiltinConfig {
            id: id
                .parse()
                .expect("invalid built-in user uuid for builtin user id"),
            name: name.to_string(),
            description,
        }
    }
}

/// Internal user used by Nexus when authenticating external requests
pub static USER_EXTERNAL_AUTHN: LazyLock<UserBuiltinConfig> =
    LazyLock::new(|| {
        UserBuiltinConfig::new_static(
            "001de000-05e4-4000-8000-000000000003",
            "external-authn",
            "used by Nexus when authenticating external requests",
        )
    });
