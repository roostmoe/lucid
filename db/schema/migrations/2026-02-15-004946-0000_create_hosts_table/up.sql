CREATE TABLE inventory_hosts (
  "id" UUID NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
	"created_at" TIMESTAMPTZ NOT NULL DEFAULT now(),
	"updated_at" TIMESTAMPTZ NOT NULL DEFAULT now(),
	"deleted_at" TIMESTAMPTZ
);
SELECT diesel_manage_updated_at('inventory_hosts');
