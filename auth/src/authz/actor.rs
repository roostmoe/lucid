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

impl AuthenticatedActor {}

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
            .add_attribute_getter("org", |a: &AuthenticatedActor| {
                match a.actor {
                    authn::Actor::OrganisationUser { organisation_id, .. } => Some(),
                }
            })
    }
}
