DROP TRIGGER chat_sessions_search_vector_update on chat_sessions;
DROP TRIGGER chat_messages_search_vector_update on chat_messages;

DROP FUNCTION chat_sessions_search_vector_update;
DROP FUNCTION chat_messages_search_vector_update;

ALTER TABLE chat_messages DROP COLUMN search_vector;
