use std::collections::BTreeSet;

use lucid_common::api::{ResourceType, error::Error};
use tracing::trace;
use uuid::Uuid;

use crate::{authn, context::OpContext};

#[derive(Clone, Debug)]
pub struct RoleSet {
    roles: BTreeSet<(ResourceType, Uuid, String)>,
}

impl RoleSet {
    pub fn new() -> Self {
        Self {
            roles: BTreeSet::new(),
        }
    }

    pub fn has_role(
        &self,
        resource_type: ResourceType,
        uuid: Uuid,
        role_name: &str,
    ) -> bool {
        self.roles.contains(&(
            resource_type,
            uuid,
            role_name.to_string(),
        ))
    }

    fn insert(
        &mut self,
        resource_type: ResourceType,
        uuid: Uuid,
        role_name: impl Into<String>,
    ) {
        self.roles.insert((
            resource_type,
            uuid,
            role_name.into(),
        ));
    }
}

pub async fn load_roles_for_resource_tree<R>(
    resource: &R,
    opctx: &OpContext,
    authn: &authn::Context,
    roleset: &mut RoleSet,
) -> Result<(), Error>
where
    R: ApiResource,
{
    // If roles can be assigned directly on this resource, load them.
    if let Some(with_roles) = resource.as_resource_with_roles() {
        let resource_type = resource.resource_type();
        let resource_id = with_roles.resource_id();
        load_directly_attached_roles(
            opctx,
            authn,
            resource_type,
            resource_id,
            roleset,
        )
        .await?;

        // If roles can be conferred by another resource, load that resource's
        // roles, too.
        if let Some((resource_type, resource_id)) =
            with_roles.conferred_roles_by(authn)?
        {
            load_directly_attached_roles(
                opctx,
                authn,
                resource_type,
                resource_id,
                roleset,
            )
            .await?;
        }
    }

    // If this resource has a parent, the user's roles on the parent
    // might grant them access to this resource.  We have to fetch
    // those, too.  This process is recursive up to the root.
    //
    // (In general, there could be another resource with _any_ kind of
    // relationship to this one that grants them a role that grants
    // access to this resource.  In practice, we only use "parent", and
    // it's clearer to just call this "parent" than
    // "related_resources_whose_roles_might_grant_access_to_this".)
    if let Some(parent) = resource.parent() {
        parent.load_roles(opctx, authn, roleset).await?;
    }

    Ok(())
}

async fn load_directly_attached_roles(
    opctx: &OpContext,
    authn: &authn::Context,
    resource_type: ResourceType,
    resource_id: Uuid,
    roleset: &mut RoleSet,
) -> Result<(), Error> {
    // If the user is authenticated ...
    if let Some(actor) = authn.actor() {
        // ... then fetch all the roles for this user that are associated with
        // this resource.
        trace!(
            actor = actor,
            resource_type = resource_type,
            resource_id = resource_id.to_string(),
            "loading roles"
        );

        let Some((identity_id, identity_type)) =
            actor.id_and_type_for_role_assignment()
        else {
            trace!(
                actor = actor,
                resource_type = resource_type,
                resource_id = resource_id.to_string(),
                "actor cannot have roles",
            );
            return Ok(());
        };

        let roles = opctx
            .datastore()
            .role_asgn_list_for(
                opctx,
                identity_type,
                identity_id,
                resource_type,
                resource_id,
            )
            .await?;

        // Add each role to the output roleset.
        for role_asgn in roles {
            assert_eq!(resource_type.to_string(), role_asgn.resource_type);
            roleset.insert(resource_type, resource_id, &role_asgn.role_name);
        }
    }

    Ok(())
}
