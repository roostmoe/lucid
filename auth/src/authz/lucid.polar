actor AnyActor {}
actor AuthenticatedActor {}

# For any resource, `actor` can perform action `action` on it if they're
# authenticated and their role(s) give them the corresponding permission on that
# resource.
allow(actor: AnyActor, action: Action, resource) if
    actor.authenticated and
    has_permission(actor.authn_actor.unwrap(), action.to_perm(), resource);

# Define role relationships
has_role(actor: AuthenticatedActor, role: String, resource: Resource)
	if resource.has_role(actor, role);

allow(actor: AnyActor, action: Action, resource) if
  actor.authenticated and
  has_permission(actor.authn_actor.unwrap(), action.to_perm(), resource);

has_role(actor: AuthenticatedActor, role: String, resource: Resource)
  if resource.has_role(actor, role);

resource Organisation {
	permissions = [
	  "get",
	  "list",
	  "create",
	  "update",
	  "delete",
	];

	roles = [
	  "admin",
	  "viewer",
	];

	# Roles implied by other roles on this resource
	"viewer" if "admin";

	# Permissions granted directly by roles on this resource
	"get" if "viewer";
	"list" if "viewer";

	"create" if "admin";
	"update" if "admin";
	"delete" if "admin";
}

resource User {
	permissions = [
	  "get",
	  "list",
	  "create",
	  "update",
	  "delete",
	];

	roles = [
	  "admin",
	  "viewer",
	];

	# Roles implied by other roles on this resource
	"viewer" if "admin";

	# Permissions granted directly by roles on this resource
	"get" if "viewer";
	"list" if "viewer";

	"create" if "admin";
	"update" if "admin";
	"delete" if "admin";
}

resource Database {
	permissions = ["query"];
}

# All authenticated users have the "query" permission on the database.
has_permission(_actor: AuthenticatedActor, "query", _resource: Database);
