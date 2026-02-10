use futures::{FutureExt, future::BoxFuture};
use lucid_auth_macros::authz_resource;
use lucid_common::api::{ResourceType, error::{Error, LookupType}};
use lucid_uuid_kinds::{GenericUuid, OrganisationIdUuid, UserIdUuid};
use oso::PolarClass;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    authn,
    authz::{
        Action, 
        AnyActor,
        AuthenticatedActor,
        AuthorizedResource,
        Authz,
        RoleSet,
        oso_generic::Init,
        roles::load_roles_for_resource_tree,
    },
    context::OpContext,
};

pub trait ApiResource:
    std::fmt::Debug + oso::ToPolar + Send + Sync + 'static
{
    /// If roles can be assigned to this resource, return this object as a
    /// [`ApiResourceWithRoles`]
    ///
    /// If roles cannot be assigned to this resource, return `None`
    fn as_resource_with_roles(&self) -> Option<&dyn ApiResourceWithRoles>;

    /// If this resource has a parent in the API hierarchy whose assigned roles
    /// can affect access to this resource, return the parent resource.
    /// Otherwise, return `None`.
    fn parent(&self) -> Option<&dyn AuthorizedResource>;

    fn resource_type(&self) -> ResourceType;
    fn lookup_type(&self) -> &LookupType;

    /// Returns an error as though this resource were not found, suitable for
    /// use when an actor should not be able to see that this resource exists.
    fn not_found(&self) -> Error {
        self.lookup_type().clone().into_not_found(self.resource_type())
    }
}

/// Describes an authz resource on which we allow users to assign roles
pub trait ApiResourceWithRoles: ApiResource {
    fn resource_id(&self) -> Uuid;

    /// Returns an optional other resource whose roles should be fetched along
    /// with this resource
    ///
    /// This exists to support the behavior that Silo-level roles can confer
    /// Fleet-level roles.  That is, it's possible to set configuration on the
    /// Silo that means "if a person has the 'admin' role on this Silo, then
    /// they also get the 'admin' role on the Fleet."  In order to implement
    /// this, if such a policy exists on the user's Silo, then we have to load a
    /// user's roles on that Silo whenever we would load the roles for the
    /// Fleet.
    ///
    /// Note this differs from "parent" in that it's not recursive.  With
    /// "parent", all of the roles that might affect the parent will be fetched,
    /// which include all of _its_ parents.  With this function, we only fetch
    /// this one resource's directly-attached roles.
    fn conferred_roles_by(
        &self,
        authn: &authn::Context,
    ) -> Result<Option<(ResourceType, Uuid)>, Error>;
}

/// Describes the specific roles for an `ApiResourceWithRoles`
pub trait ApiResourceWithRolesType: ApiResourceWithRoles {
    type AllowedRoles: serde::Serialize
        + serde::de::DeserializeOwned
        + Clone;
}

impl<T> AuthorizedResource for T
where
    T: ApiResource + oso::PolarClass + Clone,
{
    fn load_roles<'fut>(
        &'fut self,
        opctx: &'fut OpContext,
        authn: &'fut authn::Context,
        roleset: &'fut mut RoleSet,
    ) -> BoxFuture<'fut, Result<(), Error>> {
        load_roles_for_resource_tree(self, opctx, authn, roleset).boxed()
    }

    fn on_unauthorized(
        &self,
        authz: &Authz,
        error: Error,
        actor: AnyActor,
        action: Action,
    ) -> Error {
        if action == Action::Get {
            return self.not_found();
        }

        // If the user failed an authz check, and they can't even read this
        // resource, then we should produce a 404 rather than a 401/403.
        match authz.is_allowed(&actor, Action::Get, self) {
            Err(error) => Error::internal_error(&format!(
                "failed to compute read authorization to determine visibility: \
                {:#}",
                error
            )),
            Ok(false) => self.not_found(),
            Ok(true) => error,
        }
    }

    fn polar_class(&self) -> oso::Class {
        Self::get_polar_class()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Organisation {
    key: OrganisationIdUuid,
    lookup_type: LookupType,
}

impl Organisation {
    pub fn id(&self) -> OrganisationIdUuid {
        self.key.clone().into()
    }
}

impl Eq for Organisation {}
impl PartialEq for Organisation {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
}

impl PolarClass for Organisation {
    fn get_polar_class_builder() -> oso::ClassBuilder<Self> {
        oso::Class::builder()
            .with_equality_check()
    }
}

impl ApiResource for Organisation {
    fn parent(&self) -> Option<&dyn AuthorizedResource> {
        None
    }

    fn resource_type(&self) -> ResourceType {
        ResourceType::Organisation
    }

    fn lookup_type(&self) -> &LookupType {
        &self.lookup_type
    }

    fn as_resource_with_roles(
        &self,
    ) -> Option<&dyn ApiResourceWithRoles> {
        None
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct User {
    key: UserIdUuid,
    lookup_type: LookupType,
}

impl User {
    pub fn new(id: UserIdUuid) -> Self {
        Self {
            key: id.clone(),
            lookup_type: LookupType::ById(id.into_untyped_uuid()),
        }
    }

    pub fn id(&self) -> UserIdUuid {
        self.key.clone().into()
    }
}

impl Eq for User {}
impl PartialEq for User {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
}

impl PolarClass for User {
    fn get_polar_class_builder() -> oso::ClassBuilder<Self> {
        oso::Class::builder()
            .with_equality_check()
    }
}

impl ApiResource for User {
    fn parent(&self) -> Option<&dyn AuthorizedResource> {
        None
    }

    fn resource_type(&self) -> ResourceType {
        ResourceType::Organisation
    }

    fn lookup_type(&self) -> &LookupType {
        &self.lookup_type
    }

    fn as_resource_with_roles(
        &self,
    ) -> Option<&dyn ApiResourceWithRoles> {
        None
    }
}

/// Represents the database itself to Polar
///
/// This exists so that we can have roles with no access to the database at all.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Database;
/// Singleton representing the `Database` itself for authz purposes
pub const DATABASE: Database = Database;

impl oso::PolarClass for Database {
    fn get_polar_class_builder() -> oso::ClassBuilder<Self> {
        oso::Class::builder().add_method(
            "has_role",
            |_d: &Database, _actor: AuthenticatedActor, _role: String| {
                // There is an explicit rule in the Oso policy granting the
                // appropriate roles on "Database" to the appropriate actors.
                // We don't need to grant anything extra here.
                false
            },
        )
    }
}

impl AuthorizedResource for Database {
    fn load_roles<'fut>(
        &'fut self,
        _: &'fut OpContext,
        _: &'fut authn::Context,
        _: &'fut mut RoleSet,
    ) -> BoxFuture<'fut, Result<(), Error>> {
        // We don't use (database) roles to grant access to the database.  The
        // role assignment is hardcoded for all authenticated users.  See the
        // "has_role" Polar method above.
        //
        // Instead of this, we could modify this function to insert into
        // `RoleSet` the "database user" role.  However, this doesn't fit into
        // the type signature of roles supported by RoleSet.  RoleSet is really
        // for roles on database objects -- it assumes they have a ResourceType
        // and id, neither of which is true for `Database`.
        futures::future::ready(Ok(())).boxed()
    }

    fn on_unauthorized(
        &self,
        _: &Authz,
        error: Error,
        _: AnyActor,
        _: Action,
    ) -> Error {
        error
    }

    fn polar_class(&self) -> oso::Class {
        Self::get_polar_class()
    }
}

authz_resource! {
    name = Host,
    parent = Organisation,
    primary_key = Uuid,
    roles_allowed = true,
    polar_snippet = InTenant,
}
