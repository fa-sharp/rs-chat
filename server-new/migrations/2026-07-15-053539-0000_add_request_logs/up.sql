CREATE TABLE logs (
  id SERIAL PRIMARY KEY,
  kind text NOT NULL, -- chat, title, prompt, image, audio, etc.
  user_id uuid NOT NULL REFERENCES users (id) ON DELETE CASCADE,
  provider_id integer REFERENCES providers (id) ON DELETE SET NULL,
  session_id uuid REFERENCES chat_sessions (id) ON DELETE SET NULL,
  message_id uuid REFERENCES chat_messages (id) ON DELETE SET NULL,
  model text NOT NULL,
  request_id text, -- provider request ID
  input_tokens integer,
  output_tokens integer,
  cost numeric(12, 6),
  status text NOT NULL, -- completed, failed, cancelled
  error text,
  started_at timestamptz NOT NULL DEFAULT now(),
  completed_at timestamptz
);

CREATE INDEX logs_user_id_started_at_idx ON logs (user_id, started_at DESC);

CREATE INDEX logs_session_id_idx ON logs (session_id);

CREATE INDEX logs_message_id_idx ON logs (message_id);

CREATE INDEX logs_provider_id_started_at_idx ON logs (provider_id, started_at DESC);
