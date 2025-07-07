// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::query_builder::QueryId, Clone, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "chat_message_role"))]
    pub struct ChatMessageRole;

    #[derive(diesel::query_builder::QueryId, Clone, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "file_content_type"))]
    pub struct FileContentType;

    #[derive(diesel::query_builder::QueryId, Clone, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "file_storage"))]
    pub struct FileStorage;

    #[derive(diesel::query_builder::QueryId, Clone, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "llm_provider"))]
    pub struct LlmProvider;
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::LlmProvider;

    api_keys (id) {
        id -> Uuid,
        user_id -> Uuid,
        provider -> LlmProvider,
        ciphertext -> Bytea,
        nonce -> Bytea,
        created_at -> Timestamptz,
    }
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
        meta -> Jsonb,
    }
}

diesel::table! {
    chat_messages_attachments (id) {
        id -> Uuid,
        message_id -> Uuid,
        file_id -> Uuid,
    }
}

diesel::table! {
    chat_sessions (id) {
        id -> Uuid,
        title -> Varchar,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        user_id -> Uuid,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::FileContentType;
    use super::sql_types::FileStorage;

    files (id) {
        id -> Uuid,
        user_id -> Uuid,
        name -> Text,
        content_type -> FileContentType,
        storage -> FileStorage,
        path -> Text,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    users (id) {
        id -> Uuid,
        github_id -> Varchar,
        name -> Varchar,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::joinable!(api_keys -> users (user_id));
diesel::joinable!(chat_messages -> chat_sessions (session_id));
diesel::joinable!(chat_messages_attachments -> chat_messages (message_id));
diesel::joinable!(chat_messages_attachments -> files (file_id));
diesel::joinable!(chat_sessions -> users (user_id));
diesel::joinable!(files -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    api_keys,
    chat_messages,
    chat_messages_attachments,
    chat_sessions,
    files,
    users,
);
