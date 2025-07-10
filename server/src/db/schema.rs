// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::query_builder::QueryId, Clone, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "chat_message_role"))]
    pub struct ChatMessageRole;

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
    chat_sessions (id) {
        id -> Uuid,
        title -> Varchar,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        user_id -> Uuid,
    }
}

diesel::table! {
    users (id) {
        id -> Uuid,
        github_id -> Nullable<Varchar>,
        name -> Varchar,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        sso_username -> Nullable<Text>,
        google_id -> Nullable<Text>,
        discord_id -> Nullable<Text>,
        avatar_url -> Nullable<Text>,
        oidc_id -> Nullable<Text>,
    }
}

diesel::joinable!(api_keys -> users (user_id));
diesel::joinable!(chat_messages -> chat_sessions (session_id));
diesel::joinable!(chat_sessions -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    api_keys,
    chat_messages,
    chat_sessions,
    users,
);
