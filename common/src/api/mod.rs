use parse_display::{Display, FromStr};
use serde_with::{DeserializeFromStr, SerializeDisplay};

pub mod error;

#[derive(
    Clone,
    Copy,
    Debug,
    DeserializeFromStr,
    Display,
    Eq,
    FromStr,
    Ord,
    PartialEq,
    PartialOrd,
    SerializeDisplay,
)]
#[display(style = "kebab-case")]
pub enum ResourceType {
    Organisation,
    OrganisationUser,
    OrganisationRole,
    RoleBinding,
    User,
}
