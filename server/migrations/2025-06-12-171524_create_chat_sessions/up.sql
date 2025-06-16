CREATE TABLE chat_sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid (),
    title VARCHAR NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW (),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW ()
);

SELECT
    diesel_manage_updated_at ('chat_sessions');

CREATE TYPE chat_message_role AS ENUM ('user', 'assistant', 'system');

CREATE TABLE chat_messages (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid (),
    session_id UUID NOT NULL REFERENCES chat_sessions (id) ON UPDATE CASCADE ON DELETE CASCADE,
    role chat_message_role NOT NULL,
    content TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW (),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW ()
);

CREATE INDEX chat_messages_session_id_idx ON chat_messages (session_id);

SELECT
    diesel_manage_updated_at ('chat_messages');
