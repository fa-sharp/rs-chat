// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::query_builder::QueryId, Clone, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "chat_message_role"))]
    pub struct ChatMessageRole;

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
    auth_sessions (id) {
        id -> Uuid,
        user_id -> Nullable<Uuid>,
        data -> Jsonb,
        expires_at -> Timestamptz,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
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
        meta -> Jsonb,
    }
}

diesel::table! {
    external_api_tools (id) {
        id -> Uuid,
        user_id -> Uuid,
        data -> Jsonb,
        secret_1 -> Nullable<Uuid>,
        secret_2 -> Nullable<Uuid>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    files (id) {
        id -> Uuid,
        user_id -> Uuid,
        session_id -> Nullable<Uuid>,
        path -> Text,
        file_type -> Text,
        content_type -> Text,
        size -> Int4,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    llm_logs (id) {
        id -> Int4,
        kind -> Text,
        user_id -> Uuid,
        provider_id -> Nullable<Int4>,
        session_id -> Nullable<Uuid>,
        message_id -> Nullable<Uuid>,
        model -> Text,
        request_id -> Nullable<Text>,
        input_tokens -> Nullable<Int4>,
        output_tokens -> Nullable<Int4>,
        cost -> Nullable<Numeric>,
        status -> Text,
        error -> Nullable<Text>,
        meta -> Nullable<Jsonb>,
        started_at -> Timestamptz,
        completed_at -> Nullable<Timestamptz>,
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
        openai_subtype -> Nullable<Text>,
    }
}

diesel::table! {
    secrets (id) {
        id -> Uuid,
        user_id -> Uuid,
        ciphertext -> Bytea,
        nonce -> Bytea,
        created_at -> Timestamptz,
        name -> Text,
    }
}

diesel::table! {
    system_tools (id) {
        id -> Uuid,
        user_id -> Uuid,
        data -> Jsonb,
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
diesel::joinable!(auth_sessions -> users (user_id));
diesel::joinable!(chat_messages -> chat_sessions (session_id));
diesel::joinable!(chat_sessions -> users (user_id));
diesel::joinable!(external_api_tools -> users (user_id));
diesel::joinable!(files -> chat_sessions (session_id));
diesel::joinable!(files -> users (user_id));
diesel::joinable!(llm_logs -> chat_messages (message_id));
diesel::joinable!(llm_logs -> chat_sessions (session_id));
diesel::joinable!(llm_logs -> providers (provider_id));
diesel::joinable!(llm_logs -> users (user_id));
diesel::joinable!(providers -> secrets (api_key_id));
diesel::joinable!(providers -> users (user_id));
diesel::joinable!(secrets -> users (user_id));
diesel::joinable!(system_tools -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    app_api_keys,
    auth_sessions,
    chat_messages,
    chat_sessions,
    external_api_tools,
    files,
    llm_logs,
    providers,
    secrets,
    system_tools,
    users,
);
