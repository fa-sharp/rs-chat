CREATE TYPE llm_provider AS ENUM('anthropic', 'openai', 'ollama', 'deepseek', 'google', 'openrouter');

ALTER TABLE secrets
ADD COLUMN provider llm_provider NOT NULL DEFAULT 'openai';
