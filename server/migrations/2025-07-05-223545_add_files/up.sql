CREATE TYPE file_storage AS ENUM('local', 's3');

CREATE TYPE file_content_type AS ENUM('jpg', 'png', 'gif', 'webp', 'pdf');

CREATE TABLE files (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  user_id UUID NOT NULL REFERENCES users (id),
  name TEXT NOT NULL,
  content_type file_content_type NOT NULL,
  storage file_storage NOT NULL,
  path TEXT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE chat_messages_attachments (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  message_id UUID NOT NULL REFERENCES chat_messages (id) ON UPDATE CASCADE ON DELETE CASCADE,
  file_id UUID NOT NULL REFERENCES files (id) ON UPDATE CASCADE ON DELETE CASCADE
);
