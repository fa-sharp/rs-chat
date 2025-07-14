-- Remove 'tool' from chat_message_role
ALTER TYPE chat_message_role
RENAME TO chat_message_role_old;

CREATE TYPE chat_message_role AS ENUM('user', 'assistant', 'system');

ALTER TABLE chat_messages
ALTER COLUMN role TYPE chat_message_role USING role::text::chat_message_role;

DROP TYPE chat_message_role_old;

-- Drop tools table
DROP TABLE tools;
