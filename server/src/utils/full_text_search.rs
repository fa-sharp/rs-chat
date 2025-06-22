use diesel::{prelude::QueryableByName, sql_query};
use diesel_async::RunQueryDsl;
use schemars::JsonSchema;
use uuid::Uuid;

use crate::db::DbConnection;

/// Session matches for a full-text search query of chat titles and messages
#[derive(Debug, Clone, QueryableByName, JsonSchema, serde::Serialize)]
pub struct SessionSearchResult {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    pub session_id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Double)]
    pub session_rank: f64,
    #[diesel(sql_type = diesel::sql_types::Timestamptz)]
    pub session_created_at: chrono::DateTime<chrono::Utc>,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    pub message_matches: i64,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub title_highlight: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub message_highlights: String,
}

/// Performs a full-text search of user's chat titles and messages
pub async fn full_text_query(
    conn: &mut DbConnection,
    user_id: &Uuid,
    query: &str,
    limit: i32,
) -> Result<Vec<SessionSearchResult>, diesel::result::Error> {
    let results: Vec<SessionSearchResult> = sql_query(
    r#"
        WITH search_query AS (
            SELECT plainto_tsquery('english', $1) AS query
        ),
        message_stats AS (
            SELECT
                cm.session_id,
                cs.title,
                cs.created_at,
                cm.content,
                ts_rank(cm.search_vector, sq.query) AS rank,
                COUNT(*) OVER (PARTITION BY cm.session_id) AS message_matches,
                ROW_NUMBER() OVER (
                    PARTITION BY cm.session_id
                    ORDER BY ts_rank(cm.search_vector, sq.query) DESC
                ) AS rank_in_session
            FROM chat_messages cm
                JOIN chat_sessions cs ON cm.session_id = cs.id
                CROSS JOIN search_query sq
            WHERE cm.search_vector @@ sq.query
                AND cs.user_id = $2
        )
        SELECT DISTINCT ON (session_id)
            session_id,
            rank * (1 + LOG(message_matches) * 0.1) AS session_rank,
            created_at AS session_created_at,
            message_matches,
            ts_headline('english', title, sq.query, 'StartSel=§§§HIGHLIGHT_START§§§, StopSel=§§§HIGHLIGHT_END§§§, HighlightAll=true') AS title_highlight,
            ts_headline('english', content, sq.query, 'StartSel=§§§HIGHLIGHT_START§§§, StopSel=§§§HIGHLIGHT_END§§§, MinWords=8, MaxWords=12, MaxFragments=3') AS message_highlights
        FROM message_stats ms
        CROSS JOIN search_query sq
        WHERE rank_in_session = 1  -- Only best message per session
        ORDER BY session_id, rank * (1 + LOG(message_matches) * 0.1) DESC
        LIMIT $3;
    "#,
    )
    .bind::<diesel::sql_types::Text, _>(query)
    .bind::<diesel::sql_types::Uuid, _>(user_id)
    .bind::<diesel::sql_types::Integer, _>(limit)
    .load(conn).await?;

    Ok(results)
}
