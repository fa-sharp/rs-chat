CREATE TABLE system_tools (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  user_id UUID NOT NULL REFERENCES users (id),
  data JSONB NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

SELECT
  diesel_manage_updated_at ('system_tools');

CREATE TABLE external_api_tools (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  user_id UUID NOT NULL REFERENCES users (id),
  data JSONB NOT NULL,
  secret_1 UUID REFERENCES secrets (id) ON UPDATE CASCADE ON DELETE SET NULL,
  secret_2 UUID REFERENCES secrets (id) ON UPDATE CASCADE ON DELETE SET NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

SELECT
  diesel_manage_updated_at ('external_api_tools');
