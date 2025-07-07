ALTER TABLE users
DROP COLUMN proxy_username;

ALTER TABLE users
ALTER COLUMN github_id
SET NOT NULL;
