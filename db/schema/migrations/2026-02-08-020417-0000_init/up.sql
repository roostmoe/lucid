CREATE TYPE identity_principal_type AS ENUM ('user', 'group');

-- Your SQL goes here
CREATE TABLE "role_bindings"(
	"id" UUID NOT NULL PRIMARY KEY,
	"role_name" TEXT NOT NULL,
	"organisation_id" UUID NOT NULL,
	"principal_id" UUID NOT NULL,
	"principal_type" identity_principal_type NOT NULL,
	"created_at" TIMESTAMPTZ NOT NULL,
	"updated_at" TIMESTAMPTZ NOT NULL
);

CREATE TABLE "organisation_users"(
	"user_id" UUID NOT NULL,
	"organisation_id" UUID NOT NULL,
	PRIMARY KEY("user_id", "organisation_id")
);

CREATE TABLE "organisation_roles"(
	"id" UUID NOT NULL PRIMARY KEY,
	"organisation_id" UUID NOT NULL,
	"name" TEXT NOT NULL,
	"display_name" TEXT NOT NULL,
	"description" TEXT,
	"permissions" TEXT[] NOT NULL,
	"created_at" TIMESTAMPTZ NOT NULL,
	"updated_at" TIMESTAMPTZ NOT NULL
);

CREATE TABLE "users"(
	"id" UUID NOT NULL PRIMARY KEY,
	"email" TEXT NOT NULL,
	"external_id" TEXT,
	"password_hash" TEXT,
	"created_at" TIMESTAMPTZ NOT NULL,
	"updated_at" TIMESTAMPTZ NOT NULL
);

CREATE TABLE "organisations"(
	"id" UUID NOT NULL PRIMARY KEY,
	"name" TEXT NOT NULL,
	"display_name" TEXT NOT NULL,
	"created_at" TIMESTAMPTZ NOT NULL,
	"updated_at" TIMESTAMPTZ NOT NULL
);

