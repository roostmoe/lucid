use newtype_uuid_macros::impl_typed_uuid_kinds;

pub use newtype_uuid::{GenericUuid, TypedUuid, TypedUuidTag, TypedUuidKind, TagError, ParseError};

impl_typed_uuid_kinds! {
    kinds = {
        OrganisationId = {},
        UserId = {},
        GroupId = {},
        RoleId = {},
        RoleBindingId = {},
    }
}
