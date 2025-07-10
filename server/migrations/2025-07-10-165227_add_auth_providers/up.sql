-- Migration: Add support for multiple auth providers and avatars
ALTER TABLE users
ALTER COLUMN github_id
DROP NOT NULL,
ADD COLUMN sso_username TEXT,
ADD COLUMN google_id TEXT,
ADD COLUMN discord_id TEXT,
ADD COLUMN oidc_id TEXT,
ADD COLUMN avatar_url TEXT;

-- Add unique constraints for all provider IDs
ALTER TABLE users
ADD CONSTRAINT github_id_unique UNIQUE (github_id),
ADD CONSTRAINT google_id_unique UNIQUE (google_id),
ADD CONSTRAINT discord_id_unique UNIQUE (discord_id),
ADD CONSTRAINT oidc_id_unique UNIQUE (oidc_id);
