DROP TABLE providers;

ALTER TABLE secrets
RENAME TO api_keys;
