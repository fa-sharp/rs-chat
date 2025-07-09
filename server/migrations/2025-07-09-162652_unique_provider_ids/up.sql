ALTER TABLE users
ADD CONSTRAINT github_id_unique UNIQUE (github_id),
ADD CONSTRAINT google_id_unique UNIQUE (google_id),
ADD CONSTRAINT discord_id_unique UNIQUE (discord_id);
