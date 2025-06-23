CREATE OR REPLACE FUNCTION chat_messages_update_session_updated_at () RETURNS TRIGGER AS $$
BEGIN
  UPDATE chat_sessions SET updated_at = NOW() WHERE id = NEW.session_id;
  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER chat_messages_update_session_updated_at BEFORE INSERT
OR
UPDATE ON chat_messages FOR EACH ROW
EXECUTE FUNCTION chat_messages_update_session_updated_at ();
