// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::query_builder::QueryId, Clone, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "chat_message_role"))]
    pub struct ChatMessageRole;

    #[derive(diesel::query_builder::QueryId, Clone, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "llm_provider"))]
    pub struct LlmProvider;

    #[derive(diesel::query_builder::QueryId, Clone, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "tsvector", schema = "pg_catalog"))]
    pub struct Tsvector;
}

diesel::table! {
    app_api_keys (id) {
        id -> Uuid,
        user_id -> Uuid,
        name -> Text,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::ChatMessageRole;
    use super::sql_types::Tsvector;

    chat_messages (id) {
        id -> Uuid,
        session_id -> Uuid,
        role -> ChatMessageRole,
        content -> Text,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        meta -> Jsonb,
        search_vector -> Tsvector,
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
    providers (id) {
        id -> Int4,
        name -> Text,
        provider_type -> Text,
        user_id -> Uuid,
        base_url -> Nullable<Text>,
        default_model -> Text,
        api_key_id -> Nullable<Uuid>,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::LlmProvider;

    secrets (id) {
        id -> Uuid,
        user_id -> Uuid,
        provider -> LlmProvider,
        ciphertext -> Bytea,
        nonce -> Bytea,
        created_at -> Timestamptz,
        name -> Text,
    }
}

diesel::table! {
    tools (id) {
        id -> Uuid,
        user_id -> Uuid,
        name -> Text,
        description -> Text,
        config -> Jsonb,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
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
        oidc_id -> Nullable<Text>,
        avatar_url -> Nullable<Text>,
    }
}

diesel::joinable!(app_api_keys -> users (user_id));
diesel::joinable!(chat_messages -> chat_sessions (session_id));
diesel::joinable!(chat_sessions -> users (user_id));
diesel::joinable!(providers -> secrets (api_key_id));
diesel::joinable!(providers -> users (user_id));
diesel::joinable!(secrets -> users (user_id));
diesel::joinable!(tools -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    app_api_keys,
    chat_messages,
    chat_sessions,
    providers,
    secrets,
    tools,
    users,
);
