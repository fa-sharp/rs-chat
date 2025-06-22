ALTER TABLE chat_messages
ADD COLUMN search_vector tsvector NOT NULL DEFAULT '';

CREATE INDEX chat_messages_search_vector_idx ON chat_messages USING GIN (search_vector);

UPDATE chat_messages
SET
  search_vector = setweight(
    to_tsvector(
      'english',
      (
        SELECT
          title
        FROM
          chat_sessions
        WHERE
          id = session_id
      )
    ),
    'A'
  ) || setweight(to_tsvector('english', "content"), 'B');

CREATE OR REPLACE FUNCTION chat_messages_search_vector_update () RETURNS trigger AS $$
BEGIN
    NEW.search_vector :=
        setweight(to_tsvector('english', (
            SELECT title FROM chat_sessions WHERE id = NEW.session_id
        )), 'A') || setweight(to_tsvector('english', NEW."content"), 'B');
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION chat_sessions_search_vector_update () RETURNS trigger AS $$
	BEGIN
		IF old.title = new.title THEN RETURN NEW; END IF;
		UPDATE chat_messages
		SET search_vector =
	        setweight(to_tsvector('english', NEW.title), 'A') || setweight(to_tsvector('english', "content"), 'B')
	    WHERE session_id = NEW.id;
	    RETURN NEW;
	END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER chat_messages_search_vector_update BEFORE INSERT
OR
UPDATE ON chat_messages FOR EACH ROW
EXECUTE FUNCTION chat_messages_search_vector_update ();

CREATE TRIGGER chat_sessions_search_vector_update
AFTER INSERT
OR
UPDATE ON chat_sessions FOR EACH ROW
EXECUTE FUNCTION chat_sessions_search_vector_update ();
