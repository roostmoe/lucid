-- Drop multi-tenancy tables (must drop in correct order due to foreign keys)
DROP TABLE IF EXISTS "console_sessions" CASCADE;
DROP TABLE IF EXISTS "role_bindings" CASCADE;
DROP TABLE IF EXISTS "organisation_users" CASCADE;
DROP TABLE IF EXISTS "organisation_roles" CASCADE;
DROP TABLE IF EXISTS "organisations" CASCADE;

-- Drop the identity principal type enum
DROP TYPE IF EXISTS "identity_principal_type" CASCADE;

-- Modify users table for OIDC-based auth
ALTER TABLE "users" DROP COLUMN IF EXISTS "password_hash";

-- Handle existing rows with NULL external_id by setting a placeholder
-- In production, this would be coordinated with an OIDC migration
UPDATE "users" SET "external_id" = 'migration-placeholder-' || id::TEXT WHERE "external_id" IS NULL;

-- Now make external_id required and unique
ALTER TABLE "users" 
  ALTER COLUMN "external_id" SET NOT NULL,
  ADD CONSTRAINT "users_external_id_unique" UNIQUE ("external_id");

-- Add new columns for OIDC user profile
ALTER TABLE "users" 
  ADD COLUMN "display_name" TEXT,
  ADD COLUMN "is_owner" BOOLEAN NOT NULL DEFAULT FALSE;

-- Make email unique (required for single-user mode)
ALTER TABLE "users" ADD CONSTRAINT "users_email_unique" UNIQUE ("email");
