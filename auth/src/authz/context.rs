use std::{collections::BTreeSet, sync::Arc};

use futures::future::BoxFuture;
use lucid_common::api::error::Error;
use oso::{Oso, OsoError};
use tracing::debug;

use crate::{authn, authz::{Action, actor::AnyActor, roles::RoleSet}, context::OpContext, storage::Storage};

pub struct Authz {
    oso: Oso,
    class_names: BTreeSet<String>,
}

impl Authz {
    pub(crate) fn is_allowed<R>(
        &self,
        actor: &AnyActor,
        action: Action,
        resource: &R,
    ) -> Result<bool, OsoError>
    where
        R: oso::ToPolar + Clone,
    {
        self.oso.is_allowed(actor.clone(), action, resource.clone())
    }

    pub fn into_class_names(self) -> BTreeSet<String> {
        self.class_names
    }
}

#[derive(Clone)]
pub struct Context {
    authn: Arc<authn::Context>,
    authz: Arc<Authz>,
    datastore: Arc<dyn Storage>,
}

impl Context {
    pub(crate) fn datastore(&self) -> &Arc<dyn Storage> {
        &self.datastore
    }

    pub async fn authorize<Resource>(
        &self,
        opctx: &OpContext,
        action: Action,
        resource: Resource,
    ) -> Result<(), Error>
    where
        Resource: AuthorizedResource + Clone,
    {
        let class_name = &resource.polar_class().name;
        if !self.authz.class_names.contains(class_name) {
            return Err(Error::internal_error(&format!(
                "attempted authz check on unregistered resource: {:?}",
                class_name,
            )));
        }

        let mut roles = RoleSet::new();
        resource.load_roles(opctx, &self.authn, &mut roles).await?;
        debug!(?roles, "roles");
        let actor = AnyActor::new(&self.authn, roles);
        let is_authn = self.authn.actor().is_some();
        match self.authz.is_allowed(&actor, action, &resource) {
            Ok(true) => Ok(()),
            Err(error) => Err(Error::internal_error(&format!(
                "failed to compute authorization: {:#}",
                error,
            ))),
            Ok(false) => {
                Err(if !is_authn {
                    Error::Unauthenticated {
                        internal_message: "authorization failed for unauthenticated request".into(),
                    }
                } else {
                    resource.on_unauthorized(
                        &self.authz,
                        Error::Forbidden,
                        actor,
                        action,
                    )
                })
            }
        }
    }
}

pub trait AuthorizedResource: oso::ToPolar + Send + Sync + 'static {
    /// Find all roles for the user described in `authn` that might be used to
    /// make an authorization decision on `self` (a resource)
    ///
    /// You can imagine that this function would first find roles that are
    /// explicitly associated with this resource in the database.  Then it would
    /// also find roles associated with its parent, since, for example, an
    /// Silo Administrator can access things within Projects in the
    /// silo.  This process continues up the hierarchy.
    ///
    /// That's how this works for most resources.  There are other kinds of
    /// resources (like the Database itself) that aren't stored in the database
    /// and for which a different mechanism might be used.
    fn load_roles<'fut>(
        &'fut self,
        opctx: &'fut OpContext,
        authn: &'fut authn::Context,
        roleset: &'fut mut RoleSet,
    ) -> BoxFuture<'fut, Result<(), Error>>;

    /// Invoked on authz failure to determine the final authz result
    ///
    /// This is used for some resources to check if the actor should be able to
    /// even see them and produce an appropriate error if not
    fn on_unauthorized(
        &self,
        authz: &Authz,
        error: Error,
        actor: AnyActor,
        action: Action,
    ) -> Error;

    /// Returns the Polar class that implements this resource
    fn polar_class(&self) -> oso::Class;
}
