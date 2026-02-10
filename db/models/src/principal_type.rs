use diesel_derive_enum::DbEnum;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, DbEnum)]
#[db_enum(
    value_style = "kebab-case",
    existing_type_path = "lucid_db_schema::schema::sql_types::IdentityPrincipalType"
)]
pub enum IdentityPrincipalType {
    User,
    BuiltinUser,
}
