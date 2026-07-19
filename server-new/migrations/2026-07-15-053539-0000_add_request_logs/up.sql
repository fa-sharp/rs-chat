-- Create LLM request logs table
CREATE TABLE llm_logs (
  id SERIAL PRIMARY KEY,
  kind text NOT NULL, -- chat, title, prompt, image, audio, etc.
  user_id uuid NOT NULL REFERENCES users (id) ON DELETE CASCADE,
  provider_id integer REFERENCES providers (id) ON DELETE SET NULL,
  session_id uuid REFERENCES chat_sessions (id) ON DELETE SET NULL,
  message_id uuid REFERENCES chat_messages (id) ON DELETE SET NULL,
  model text NOT NULL,
  input_tokens integer,
  output_tokens integer,
  cost numeric(12, 6),
  ttft_ms integer, -- time to first token in milliseconds
  status text NOT NULL, -- started, completed, failed, cancelled
  meta jsonb NOT NULL DEFAULT '{}', -- max_tokens, temperature, etc.
  started_at timestamptz NOT NULL DEFAULT now(),
  completed_at timestamptz
);

CREATE INDEX llm_logs_user_id_started_at_idx ON llm_logs (user_id, started_at DESC);

CREATE INDEX llm_logs_session_id_idx ON llm_logs (session_id);

CREATE UNIQUE INDEX llm_logs_message_id_unique_idx ON llm_logs (message_id)
WHERE
  message_id IS NOT NULL;

CREATE INDEX llm_logs_provider_id_started_at_idx ON llm_logs (provider_id, started_at DESC);

-- Migrate assistant metadata to LLM request logs table
INSERT INTO
  llm_logs (
    kind,
    user_id,
    session_id,
    message_id,
    provider_id,
    model,
    input_tokens,
    output_tokens,
    cost,
    status,
    meta,
    started_at
  )
SELECT
  'chat',
  chat_sessions.user_id,
  session_id,
  chat_messages.id,
  providers.id,
  coalesce((chat_messages.meta #>> '{assistant,provider_options,model}')::text, ''),
  (chat_messages.meta #>> '{assistant,usage,input_tokens}')::int4,
  (chat_messages.meta #>> '{assistant,usage,output_tokens}')::int4,
  (chat_messages.meta #>> '{assistant,usage,cost}')::numeric(12, 6),
  CASE
    WHEN (chat_messages.meta #>> '{assistant,partial}')::bool THEN 'cancelled'
    WHEN chat_messages.meta @? '$.assistant.errors[0]' THEN 'error'
    ELSE 'completed'
  END,
  jsonb_strip_nulls(
    jsonb_build_object(
      'options',
      chat_messages.meta #> '{assistant,provider_options}',
      'errors',
      chat_messages.meta #> '{assistant,errors}'
    )
  ),
  chat_messages.created_at
FROM
  chat_messages
  JOIN chat_sessions ON chat_sessions.id = chat_messages.session_id
  LEFT JOIN providers ON providers.id = (chat_messages.meta #>> '{assistant,provider_id}')::int4
WHERE
  chat_messages.role = 'assistant';
