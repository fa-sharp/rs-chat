-- Migration: Alter users table to make github_id nullable and add sso_username column
ALTER TABLE users
ALTER COLUMN github_id
DROP NOT NULL;

ALTER TABLE users
ADD COLUMN sso_username TEXT;
