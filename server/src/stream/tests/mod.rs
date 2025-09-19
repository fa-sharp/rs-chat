mod utils;
use rocket::futures::StreamExt;
use utils::*;

use std::time::Duration;
use uuid::Uuid;

use crate::{
    provider::{
        providers::LoremProvider, LlmApiProvider, LlmProviderOptions, LlmStream, LlmStreamChunk,
        LlmStreamError, LlmUsage,
    },
    stream::chat_stream_key,
};

use super::LlmStreamWriter;

async fn create_test_writer(user_id: &Uuid, session_id: &Uuid) -> LlmStreamWriter {
    let key = chat_stream_key(user_id, session_id);
    let tini = setup_tini_client();
    tini.stream_start(&key).await.expect("should start stream");
    let (ws_writer, ws_reader) = tini
        .stream_writer_ws(&key)
        .await
        .expect("should connect to WebSocket for adding events")
        .split();

    LlmStreamWriter::new(ws_writer, ws_reader, user_id, session_id)
}

#[tokio::test]
async fn stream_writer_basic_functionality() {
    let user_id = Uuid::new_v4();
    let session_id = Uuid::new_v4();
    let tini = setup_tini_client();
    let mut writer = create_test_writer(&user_id, &session_id).await;

    // Create stream
    assert!(tini.stream_exists(&writer.key()).await.unwrap());

    // Create Lorem provider and get stream
    let lorem = LoremProvider::new();
    let stream = lorem
        .chat_stream(vec![], None, &LlmProviderOptions::default())
        .await
        .expect("Failed to create lorem stream");

    // Process the stream
    let (text, tool_calls, usage, errors, cancelled) = writer.process(stream).await;

    // Verify results
    assert!(text.is_some());
    let text = text.unwrap();
    assert!(!text.is_empty());
    assert!(text.contains("Lorem ipsum"));
    assert!(text.contains("dolor sit"));

    assert!(tool_calls.is_none());
    assert!(usage.is_none());
    assert!(errors.is_some()); // Lorem provider generates some test errors
    assert!(!cancelled);

    // End stream
    assert!(tini.stream_end(&writer.key()).await.is_ok());

    // Stream should be deleted after end
    assert!(!tini.stream_exists(&writer.key()).await.unwrap());
}

#[tokio::test]
async fn stream_writer_batching() {
    let user_id = Uuid::new_v4();
    let session_id = Uuid::new_v4();
    let tini = setup_tini_client();
    let mut writer = create_test_writer(&user_id, &session_id).await;

    // Create a custom stream with small chunks to test batching
    let chunks = vec![
        "Hello", " ", "world", "!", " ", "This", " ", "is", " ", "a", " ", "test",
    ];
    let chunk_stream = tokio_stream::iter(
        chunks
            .into_iter()
            .map(|text| Ok(LlmStreamChunk::Text(text.into()))),
    );

    let stream: LlmStream = Box::pin(chunk_stream);
    let (text, _, _, _, cancelled) = writer.process(stream).await;

    assert!(text.is_some());
    let text = text.unwrap();
    assert_eq!(text, "Hello world! This is a test");
    assert!(!cancelled);

    tini.stream_end(&writer.key()).await.ok();
}

#[tokio::test]
async fn stream_writer_error_handling() {
    let user_id = Uuid::new_v4();
    let session_id = Uuid::new_v4();
    let tini = setup_tini_client();
    let mut writer = create_test_writer(&user_id, &session_id).await;

    // Create a stream that produces an error
    let error_stream = tokio_stream::iter(vec![
        Ok(LlmStreamChunk::Text("Hello".to_string())),
        Err(LlmStreamError::ProviderError("Test error".into())),
        Ok(LlmStreamChunk::Text(" World".to_string())),
    ]);

    let stream: LlmStream = Box::pin(error_stream);
    let (text, _, _, errors, cancelled) = writer.process(stream).await;

    assert!(text.is_some());
    let text = text.unwrap();
    assert_eq!(text, "Hello World");

    assert!(errors.is_some());
    let errors = errors.unwrap();
    assert!(!errors.is_empty());
    assert!(errors.iter().any(|e| e.contains("Test error")));

    assert!(!cancelled);

    tini.stream_end(&writer.key()).await.ok();
}

#[tokio::test]
async fn stream_writer_timeout() {
    let user_id = Uuid::new_v4();
    let session_id = Uuid::new_v4();
    let tini = setup_tini_client();
    let mut writer = create_test_writer(&user_id, &session_id).await;

    assert!(tini.stream_exists(&writer.key()).await.unwrap());

    // Create a stream that hangs (never yields anything)
    let hanging_stream = tokio_stream::pending::<Result<LlmStreamChunk, LlmStreamError>>();

    let stream: LlmStream = Box::pin(hanging_stream);

    // This should timeout due to LLM_TIMEOUT
    let start = std::time::Instant::now();
    let (text, _, _, errors, cancelled) = writer.process(stream).await;
    let elapsed = start.elapsed();

    // Should complete in roughly LLM_TIMEOUT duration
    assert!(elapsed >= Duration::from_secs(59)); // Allow some margin
    assert!(elapsed < Duration::from_secs(65));

    assert!(text.is_none());
    assert!(errors.is_some());
    let errors = errors.unwrap();
    assert!(errors.iter().any(|e| e.contains("Timeout")));
    assert!(!cancelled); // Timeout is not considered a cancellation

    tini.stream_end(&writer.key()).await.ok();
}

#[tokio::test]
async fn stream_writer_cancel() {
    let user_id = Uuid::new_v4();
    let session_id = Uuid::new_v4();
    let tini = setup_tini_client();
    let writer = create_test_writer(&user_id, &session_id).await;

    assert!(tini.stream_exists(&writer.key()).await.unwrap());

    // Cancel the stream
    tini.stream_cancel(&writer.key()).await.unwrap();

    // Stream should be deleted after cancel
    assert!(!tini.stream_exists(&writer.key()).await.unwrap());
}

#[tokio::test]
async fn stream_writer_usage_tracking() {
    let user_id = Uuid::new_v4();
    let session_id = Uuid::new_v4();
    let tini = setup_tini_client();
    let mut writer = create_test_writer(&user_id, &session_id).await;

    assert!(tini.stream_exists(&writer.key()).await.unwrap());

    // Create a stream with usage information
    let usage_stream = tokio_stream::iter(vec![
        Ok(LlmStreamChunk::Text("Hello".into())),
        Ok(LlmStreamChunk::Usage(LlmUsage {
            input_tokens: Some(10),
            output_tokens: Some(5),
            cost: Some(0.001),
        })),
        Ok(LlmStreamChunk::Text(" World".into())),
        Ok(LlmStreamChunk::Usage(LlmUsage {
            input_tokens: None,     // Should not override
            output_tokens: Some(7), // Should update
            cost: Some(0.002),      // Should update
        })),
    ]);

    let stream: LlmStream = Box::pin(usage_stream);
    let (text, _, usage, _, cancelled) = writer.process(stream).await;

    assert!(text.is_some());
    assert_eq!(text.unwrap(), "Hello World");

    assert!(usage.is_some());
    let usage = usage.unwrap();
    assert_eq!(usage.input_tokens, Some(10));
    assert_eq!(usage.output_tokens, Some(7));
    assert_eq!(usage.cost, Some(0.002));

    assert!(!cancelled);

    tini.stream_end(&writer.key()).await.ok();
}

#[tokio::test]
async fn redis_stream_entries() {
    let user_id = Uuid::new_v4();
    let session_id = Uuid::new_v4();
    let tini = setup_tini_client();
    let mut writer = create_test_writer(&user_id, &session_id).await;
    let key = writer.key().to_owned();

    assert!(tini.stream_exists(&writer.key()).await.unwrap());

    // Verify start event was written
    let info = tini
        .stream_info(&key)
        .await
        .expect("Failed to check stream")
        .expect("Stream not found");
    assert_eq!(info.length, 1);

    // Create a simple stream
    let simple_stream = tokio_stream::iter(vec![Ok(LlmStreamChunk::Text("Test chunk".into()))]);
    let stream: LlmStream = Box::pin(simple_stream);
    writer.process(stream).await;
    writer.flush_chunk().await.ok();

    // Should have start + text entries (ping task may add more)
    let info = tini
        .stream_info(&key)
        .await
        .expect("Failed to check stream")
        .expect("Stream not found");
    assert!(info.length >= 2);

    tini.stream_end(&key).await.ok();
}
