-- Add tools table
CREATE TABLE tools (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  user_id UUID NOT NULL REFERENCES users (id),
  name TEXT NOT NULL,
  description TEXT NOT NULL,
  input_schema JSONB NOT NULL,
  data JSONB NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

SELECT
  diesel_manage_updated_at ('tools');

CREATE INDEX tools_user_id_idx ON tools (user_id);

-- Add tool role to chat messages
ALTER TYPE chat_message_role
RENAME TO chat_message_role_old;

CREATE TYPE chat_message_role AS ENUM('user', 'assistant', 'system', 'tool');

ALTER TABLE chat_messages
ALTER COLUMN role TYPE chat_message_role USING role::text::chat_message_role;

DROP TYPE chat_message_role_old;
