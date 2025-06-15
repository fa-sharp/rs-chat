DROP INDEX chat_sessions_user_id_idx;

ALTER TABLE chat_sessions
DROP COLUMN user_id;

DROP TABLE users;
