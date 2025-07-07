-- Migration: Alter users table to make github_id nullable and add proxy_username column
ALTER TABLE users
ALTER COLUMN github_id
DROP NOT NULL;

ALTER TABLE users
ADD COLUMN proxy_username TEXT;
