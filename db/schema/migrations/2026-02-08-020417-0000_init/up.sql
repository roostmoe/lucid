CREATE TYPE "identity_principal_type" AS ENUM ('user');

CREATE TABLE "users"(
	"id" UUID NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
	"email" TEXT NOT NULL,
	"external_id" TEXT,
	"password_hash" TEXT,
  "system_admin" BOOLEAN NOT NULL DEFAULT FALSE,
	"created_at" TIMESTAMPTZ NOT NULL DEFAULT now(),
	"updated_at" TIMESTAMPTZ NOT NULL DEFAULT now()
);
SELECT diesel_manage_updated_at('users');

CREATE TABLE "organisations"(
	"id" UUID NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
	"name" TEXT NOT NULL,
	"display_name" TEXT NOT NULL,
	"created_at" TIMESTAMPTZ NOT NULL DEFAULT now(),
	"updated_at" TIMESTAMPTZ NOT NULL DEFAULT now()
);
SELECT diesel_manage_updated_at('organisations');

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

CREATE TABLE "organisation_users"(
	"user_id" UUID NOT NULL REFERENCES "users" ("id") ON DELETE CASCADE,
	"organisation_id" UUID NOT NULL REFERENCES "organisations" ("id") ON DELETE CASCADE,
	PRIMARY KEY("user_id", "organisation_id")
);

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
