// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::query_builder::QueryId, Clone, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "chat_message_role"))]
    pub struct ChatMessageRole;
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::ChatMessageRole;

    chat_messages (id) {
        id -> Uuid,
        session_id -> Uuid,
        role -> ChatMessageRole,
        content -> Text,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    chat_sessions (id) {
        id -> Uuid,
        title -> Varchar,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::joinable!(chat_messages -> chat_sessions (session_id));

diesel::allow_tables_to_appear_in_same_query!(
    chat_messages,
    chat_sessions,
);
