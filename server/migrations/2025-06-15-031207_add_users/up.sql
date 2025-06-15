CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid (),
    github_id VARCHAR NOT NULL,
    name VARCHAR NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW (),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW ()
);

SELECT
    diesel_manage_updated_at ('users');

ALTER TABLE chat_sessions
ADD COLUMN user_id UUID NOT NULL REFERENCES users (id);

CREATE INDEX chat_sessions_user_id_idx ON chat_sessions (user_id);
