actor AnyActor {}

actor AuthenticatedActor {}

allow(actor: AnyActor, action: Action, resource) if
  actor.authenticated and
  has_permission(actor.authn_actor.unwrap(), action.to_perm(), resource);

has_role(actor: AuthenticatedActor, role: String, resource: Resource)
  if resource.has_role(actor, role);
