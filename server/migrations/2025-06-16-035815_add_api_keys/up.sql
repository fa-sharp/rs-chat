CREATE TYPE llm_provider AS ENUM('anthropic', 'openai', 'ollama', 'deepseek', 'google', 'openrouter');

CREATE TABLE api_keys (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  user_id UUID NOT NULL REFERENCES users (id),
  provider llm_provider NOT NULL,
  ciphertext BYTEA NOT NULL,
  nonce BYTEA NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX api_keys_user_id_idx ON api_keys (user_id);
