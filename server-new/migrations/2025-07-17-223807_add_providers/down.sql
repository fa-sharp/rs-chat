ALTER TABLE secrets
DROP COLUMN name;

DROP TABLE providers;

ALTER TABLE secrets
RENAME TO api_keys;
