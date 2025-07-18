-- Rename API keys table to secrets
ALTER TABLE api_keys
RENAME TO secrets;

-- Create providers table
CREATE TABLE providers (
  id SERIAL PRIMARY KEY,
  name TEXT NOT NULL,
  provider_type TEXT NOT NULL,
  user_id UUID NOT NULL REFERENCES users (id),
  base_url TEXT,
  api_key_id UUID REFERENCES secrets (id),
  created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

SELECT
  diesel_manage_updated_at ('providers');

CREATE INDEX idx_providers_user_id ON providers (user_id);

-- Create default Lorem, OpenAI, Anthropic, and OpenRouter providers for every user_id, with existing API keys if found
INSERT INTO
  providers (provider_type, name, user_id, base_url, api_key_id)
SELECT
  'openai',
  'OpenAI',
  id,
  NULL,
  (
    SELECT
      id
    FROM
      secrets
    WHERE
      provider = 'openai'
      AND user_id = users.id
  )
FROM
  users;

INSERT INTO
  providers (provider_type, name, user_id, base_url, api_key_id)
SELECT
  'openai',
  'OpenRouter',
  id,
  'https://openrouter.ai/api/v1',
  (
    SELECT
      id
    FROM
      secrets
    WHERE
      provider = 'openrouter'
      AND user_id = users.id
  )
FROM
  users;

INSERT INTO
  providers (provider_type, name, user_id, base_url, api_key_id)
SELECT
  'lorem',
  'Lorem',
  id,
  NULL,
  NULL
FROM
  users;

INSERT INTO
  providers (provider_type, name, user_id, base_url, api_key_id)
SELECT
  'anthropic',
  'Anthropic',
  id,
  NULL,
  (
    SELECT
      id
    FROM
      secrets
    WHERE
      provider = 'anthropic'
      AND user_id = users.id
  )
FROM
  users;
