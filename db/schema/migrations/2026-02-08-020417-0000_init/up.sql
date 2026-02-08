CREATE TYPE "identity_principal_type" AS ENUM ('user', 'group');

CREATE TABLE "users"(
	"id" UUID NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
	"email" TEXT NOT NULL,
	"external_id" TEXT,
	"password_hash" TEXT,
	"created_at" TIMESTAMPTZ NOT NULL DEFAULT now(),
	"updated_at" TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE "console_sessions"(
	"id" UUID NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
	"user_id" UUID NOT NULL REFERENCES "users" ("id") ON DELETE CASCADE,
	"token" TEXT NOT NULL,
	"created_at" TIMESTAMPTZ NOT NULL DEFAULT now(),
	"updated_at" TIMESTAMPTZ NOT NULL DEFAULT now(),
	"last_seen_at" TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE "organisations"(
	"id" UUID NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
	"name" TEXT NOT NULL,
	"display_name" TEXT NOT NULL,
	"created_at" TIMESTAMPTZ NOT NULL DEFAULT now(),
	"updated_at" TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE "organisation_users"(
	"user_id" UUID NOT NULL REFERENCES "users" ("id") ON DELETE CASCADE,
	"organisation_id" UUID NOT NULL REFERENCES "organisations" ("id") ON DELETE CASCADE,
	PRIMARY KEY("user_id", "organisation_id")
);

CREATE TABLE "organisation_roles"(
	"id" UUID NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
	"organisation_id" UUID NOT NULL REFERENCES "organisations" ("id") ON DELETE CASCADE,
	"name" TEXT NOT NULL,
	"display_name" TEXT NOT NULL,
	"description" TEXT,
  "permissions" TEXT[] NOT NULL DEFAULT '{}',
	"created_at" TIMESTAMPTZ NOT NULL DEFAULT now(),
	"updated_at" TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE "role_bindings"(
	"id" UUID NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
	"role_name" TEXT NOT NULL,
	"organisation_id" UUID NOT NULL REFERENCES "organisations" ("id") ON DELETE CASCADE,
	"principal_id" UUID NOT NULL,
	"principal_type" identity_principal_type NOT NULL,
	"created_at" TIMESTAMPTZ NOT NULL DEFAULT now(),
	"updated_at" TIMESTAMPTZ NOT NULL DEFAULT now()
);
