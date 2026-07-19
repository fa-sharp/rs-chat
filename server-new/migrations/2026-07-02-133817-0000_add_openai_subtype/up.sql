ALTER TABLE providers
ADD COLUMN openai_subtype TEXT;

UPDATE providers
SET
  openai_subtype = 'openrouter'
WHERE
  base_url = 'https://openrouter.ai/api/v1';
