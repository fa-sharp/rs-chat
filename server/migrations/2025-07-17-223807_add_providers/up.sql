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
  default_model TEXT NOT NULL,
  api_key_id UUID REFERENCES secrets (id),
  created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_providers_user_id ON providers (user_id);

-- Create providers for users with existing API keys
INSERT INTO
  providers (provider_type, name, user_id, base_url, default_model, api_key_id)
SELECT
  'openai',
  'OpenAI',
  secrets.user_id,
  NULL,
  'gpt-4o-mini',
  id
FROM
  secrets
WHERE
  secrets.provider = 'openai';

INSERT INTO
  providers (provider_type, name, user_id, base_url, default_model, api_key_id)
SELECT
  'openai',
  'OpenRouter',
  secrets.user_id,
  'https://openrouter.ai/api/v1',
  'openai/gpt-4o-mini',
  id
FROM
  secrets
WHERE
  secrets.provider = 'openrouter';

INSERT INTO
  providers (provider_type, name, user_id, base_url, default_model, api_key_id)
SELECT
  'anthropic',
  'Anthropic',
  secrets.user_id,
  NULL,
  'claude-3-7-sonnet-latest',
  id
FROM
  secrets
WHERE
  secrets.provider = 'anthropic';

-- Add name to secrets table
ALTER TABLE secrets
ADD COLUMN name TEXT NOT NULL DEFAULT 'api_key';

UPDATE secrets
SET
  name = secrets.provider || '_api_key';
