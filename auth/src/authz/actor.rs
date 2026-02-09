use lucid_common::api::ResourceType;
use lucid_uuid_kinds::GenericUuid;
use uuid::Uuid;

use crate::{authn, authz::roles::RoleSet};

#[derive(Clone, Debug)]
pub struct AnyActor {
    actor: Option<authn::Actor>,
    roles: RoleSet,
}

impl AnyActor {
    pub fn new(authn: &authn::Context, roles: RoleSet) -> Self {
        let actor = authn.actor().cloned();
        AnyActor { actor, roles }
    }
}

impl oso::PolarClass for AnyActor {
    fn get_polar_class_builder() -> oso::ClassBuilder<Self> {
        oso::Class::builder()
            .add_attribute_getter("authenticated", |a: &AnyActor| {
                a.actor.is_some()
            })
            .add_attribute_getter("authn_actor", |a: &AnyActor| {
                a.actor.map(|actor| AuthenticatedActor {
                    actor,
                    roles: a.roles.clone(),
                })
            })
    }
}

#[derive(Clone, Debug)]
pub struct AuthenticatedActor {
    actor: authn::Actor,
    roles: RoleSet,
}

impl AuthenticatedActor {
    /// Returns whether this actor has explicitly been granted the given role
    /// for the given resource
    pub fn has_role_resource(
        &self,
        resource_type: ResourceType,
        resource_id: Uuid,
        role: &str,
    ) -> bool {
        self.roles.has_role(resource_type, resource_id, role)
    }
}

impl PartialEq for AuthenticatedActor {
    fn eq(&self, other: &Self) -> bool {
        self.actor == other.actor
    }
}

impl Eq for AuthenticatedActor {}

impl oso::PolarClass for AuthenticatedActor {
    fn get_polar_class_builder() -> oso::ClassBuilder<Self> {
        oso::Class::builder()
            .with_equality_check()
            .add_attribute_getter("is_user", |a: &AuthenticatedActor| {
                match a.actor {
                    authn::Actor::OrganisationUser { .. } => true,
                }
            })
            // .add_attribute_getter("org", |a: &AuthenticatedActor| {
            //     match a.actor {
            //         authn::Actor::OrganisationUser { organisation_id, .. } => Some(organisation_id),
            //     }
            // })
    }
}
