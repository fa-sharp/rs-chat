CREATE TABLE auth_sessions (
  id UUID PRIMARY KEY,
  user_id UUID NULL REFERENCES users (id),
  data JSONB NOT NULL,
  expires_at TIMESTAMPTZ NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

SELECT
  diesel_manage_updated_at ('auth_sessions');

CREATE INDEX auth_sessions_user_id_idx ON auth_sessions (user_id);

CREATE INDEX auth_sessions_expires_at_idx ON auth_sessions (expires_at);
