ALTER TABLE "console_sessions"
    ADD COLUMN "organisation_id" UUID NOT NULL REFERENCES "organisations" ("id") ON DELETE CASCADE;
