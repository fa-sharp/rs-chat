-- Drop all unique constraints first
ALTER TABLE users
DROP CONSTRAINT IF EXISTS github_id_unique,
DROP CONSTRAINT IF EXISTS google_id_unique,
DROP CONSTRAINT IF EXISTS discord_id_unique,
DROP CONSTRAINT IF EXISTS oidc_id_unique;

-- Drop all added columns and restore github_id constraint
ALTER TABLE users
DROP COLUMN avatar_url,
DROP COLUMN oidc_id,
DROP COLUMN discord_id,
DROP COLUMN google_id,
DROP COLUMN sso_username,
ALTER COLUMN github_id
SET NOT NULL;
