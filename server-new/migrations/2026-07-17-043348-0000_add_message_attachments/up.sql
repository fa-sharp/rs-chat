-- Create table tracking file attachments to messaages
CREATE TABLE message_attachments (
  message_id uuid REFERENCES chat_messages (id) ON DELETE CASCADE,
  file_id uuid REFERENCES files (id) ON DELETE CASCADE,
  PRIMARY KEY (message_id, file_id)
);
