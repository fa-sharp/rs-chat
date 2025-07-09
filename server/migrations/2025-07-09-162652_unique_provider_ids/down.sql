ALTER TABLE users
DROP CONSTRAINT github_id_unique,
DROP CONSTRAINT google_id_unique,
DROP CONSTRAINT discord_id_unique;
