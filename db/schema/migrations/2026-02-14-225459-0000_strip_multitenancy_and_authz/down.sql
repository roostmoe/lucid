-- Reverse users table changes
ALTER TABLE "users" DROP CONSTRAINT IF EXISTS "users_email_unique";
ALTER TABLE "users" DROP COLUMN IF EXISTS "is_owner";
ALTER TABLE "users" DROP COLUMN IF EXISTS "display_name";
ALTER TABLE "users" DROP CONSTRAINT IF EXISTS "users_external_id_unique";
ALTER TABLE "users" ALTER COLUMN "external_id" DROP NOT NULL;
ALTER TABLE "users" ADD COLUMN "password_hash" TEXT;

-- Recreate identity principal type
CREATE TYPE "identity_principal_type" AS ENUM ('user');

-- Recreate organisations table
CREATE TABLE "organisations"(
	"id" UUID NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
	"name" TEXT NOT NULL,
	"display_name" TEXT NOT NULL,
	"created_at" TIMESTAMPTZ NOT NULL DEFAULT now(),
	"updated_at" TIMESTAMPTZ NOT NULL DEFAULT now()
);
SELECT diesel_manage_updated_at('organisations');

-- Recreate organisation_users junction table
CREATE TABLE "organisation_users"(
	"user_id" UUID NOT NULL REFERENCES "users" ("id") ON DELETE CASCADE,
	"organisation_id" UUID NOT NULL REFERENCES "organisations" ("id") ON DELETE CASCADE,
	PRIMARY KEY("user_id", "organisation_id")
);

-- Recreate console_sessions table
CREATE TABLE "console_sessions"(
	"id" UUID NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
	"user_id" UUID NOT NULL REFERENCES "users" ("id") ON DELETE CASCADE,
	"organisation_id" UUID NOT NULL REFERENCES "organisations" ("id") ON DELETE CASCADE,
	"token" TEXT NOT NULL,
	"created_at" TIMESTAMPTZ NOT NULL DEFAULT now(),
	"updated_at" TIMESTAMPTZ NOT NULL DEFAULT now(),
	"last_seen_at" TIMESTAMPTZ NOT NULL DEFAULT now()
);
SELECT diesel_manage_updated_at('console_sessions');

-- Recreate role_bindings table
CREATE TABLE "role_bindings" (
	"id" UUID NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
	"role_name" TEXT NOT NULL,
	"organisation_id" UUID NOT NULL REFERENCES "organisations" ("id") ON DELETE CASCADE,
	"principal_id" UUID NOT NULL,
	"principal_type" identity_principal_type NOT NULL,
	"resource_id" UUID NOT NULL,
	"resource_type" TEXT NOT NULL,
	"created_at" TIMESTAMPTZ NOT NULL DEFAULT now(),
	"updated_at" TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Recreate organisation_roles table
CREATE TABLE "organisation_roles" (
	"id" UUID NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
	"organisation_id" UUID NOT NULL REFERENCES "organisations" ("id") ON DELETE CASCADE,
	"name" TEXT NOT NULL,
	"display_name" TEXT NOT NULL,
	"description" TEXT,
	"permissions" TEXT[] NOT NULL DEFAULT '{}',
	"created_at" TIMESTAMPTZ NOT NULL DEFAULT now(),
	"updated_at" TIMESTAMPTZ NOT NULL DEFAULT now()
);
SELECT diesel_manage_updated_at('organisation_roles');
