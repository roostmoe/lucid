use newtype_uuid_macros::impl_typed_uuid_kinds;

pub use newtype_uuid::{GenericUuid, ParseError, TagError, TypedUuid, TypedUuidKind, TypedUuidTag};

impl_typed_uuid_kinds! {
    settings = {
        schemars08 = {
            rust_type = {
                crate = "lucid-uuid-uinds",
                version = "*",
                path = "lucid_uuid_kinds"
            }
        }
    },

    kinds = {
        User = {},
        Host = {}
    }
}
