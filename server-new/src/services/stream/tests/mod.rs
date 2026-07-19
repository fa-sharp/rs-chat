// use futures::StreamExt;
// use reqwest_websocket::WebSocket;
// use uuid::Uuid;

// use crate::services::stream::{StreamService, writer::LlmStreamWriter};

// mod utils;
// use utils::*;

// async fn create_test_writer(
//     user_id: &Uuid,
//     session_id: &Uuid,
// ) -> (String, WebSocket, LlmStreamWriter) {
//     let key = StreamService::chat_stream_key(user_id, session_id);
//     let tini = setup_tini_client();
//     tini.stream_start(&key).await.expect("should start stream");
//     let ws = tini
//         .stream_writer_ws(&key)
//         .await
//         .expect("should connect to WebSocket for adding events");

//     (key, ws, LlmStreamWriter::new())
// }

// #[tokio::test]
// async fn stream_writer_basic_functionality() {
//     let user_id = Uuid::new_v4();
//     let session_id = Uuid::new_v4();
//     let tini = setup_tini_client();
//     let (key, ws, mut writer) = create_test_writer(&user_id, &session_id).await;
//     let (ws_writer, ws_reader) = ws.split();

//     // Create stream
//     assert!(tini.stream_exists(&key).await.unwrap());

//     // Create Lorem provider and get stream
//     let lorem = LoremProvider::new();
//     let stream = lorem
//         .chat_stream(vec![], None, &LlmProviderOptions::default())
//         .await
//         .expect("Failed to create lorem stream");

//     // Process the stream
//     let LlmOutput {
//         text,
//         tool_calls,
//         usage,
//         errors,
//         cancelled,
//         ..
//     } = writer.process(stream, ws_writer, ws_reader).await;

//     // Verify results
//     assert!(text.is_some());
//     let text = text.unwrap();
//     assert!(!text.is_empty());
//     assert!(text.contains("Lorem ipsum"));
//     assert!(text.contains("dolor sit"));

//     assert!(tool_calls.is_none());
//     assert!(usage.is_none());
//     assert!(errors.is_some()); // Lorem provider generates some test errors
//     assert!(!cancelled);

//     // End stream
//     assert!(tini.stream_end(&key).await.is_ok());

//     // Stream should be deleted after end
//     assert!(!tini.stream_exists(&key).await.unwrap());
// }

// #[tokio::test]
// async fn stream_writer_batching() {
//     let user_id = Uuid::new_v4();
//     let session_id = Uuid::new_v4();
//     let tini = setup_tini_client();
//     let (key, ws, mut writer) = create_test_writer(&user_id, &session_id).await;
//     let (ws_writer, ws_reader) = ws.split();

//     // Create a custom stream with small chunks to test batching
//     let chunks = vec![
//         "Hello", " ", "world", "!", " ", "This", " ", "is", " ", "a", " ", "test",
//     ];
//     let chunk_stream = tokio_stream::iter(
//         chunks
//             .into_iter()
//             .map(|text| Ok(LlmStreamChunk::Text(text.into()))),
//     );

//     let stream: LlmStream = Box::pin(chunk_stream);
//     let LlmOutput {
//         text, cancelled, ..
//     } = writer.process(stream, ws_writer, ws_reader).await;

//     assert!(text.is_some());
//     let text = text.unwrap();
//     assert_eq!(text, "Hello world! This is a test");
//     assert!(!cancelled);

//     tini.stream_end(&key).await.ok();
// }

// #[tokio::test]
// async fn stream_writer_error_handling() {
//     let user_id = Uuid::new_v4();
//     let session_id = Uuid::new_v4();
//     let tini = setup_tini_client();
//     let (key, ws, mut writer) = create_test_writer(&user_id, &session_id).await;
//     let (ws_writer, ws_reader) = ws.split();

//     // Create a stream that produces an error
//     let error_stream = tokio_stream::iter(vec![
//         Ok(LlmStreamChunk::Text("Hello".to_string())),
//         Err(LlmStreamError::ProviderError("Test error".into())),
//         Ok(LlmStreamChunk::Text(" World".to_string())),
//     ]);

//     let stream: LlmStream = Box::pin(error_stream);
//     let LlmOutput {
//         text,
//         errors,
//         cancelled,
//         ..
//     } = writer.process(stream, ws_writer, ws_reader).await;

//     assert!(text.is_some());
//     let text = text.unwrap();
//     assert_eq!(text, "Hello World");

//     assert!(errors.is_some());
//     let errors = errors.unwrap();
//     assert!(!errors.is_empty());
//     assert!(errors.iter().any(|e| e.contains("Test error")));

//     assert!(!cancelled);

//     tini.stream_end(&key).await.ok();
// }

// #[tokio::test]
// async fn stream_writer_cancel() {
//     let user_id = Uuid::new_v4();
//     let session_id = Uuid::new_v4();
//     let tini = setup_tini_client();
//     let (key, ws, mut writer) = create_test_writer(&user_id, &session_id).await;
//     let (ws_writer, ws_reader) = ws.split();

//     assert!(tini.stream_exists(&key).await.unwrap());

//     let stream = LoremProvider::new()
//         .chat_stream(vec![], None, &LlmProviderOptions::default())
//         .await
//         .expect("Failed to create lorem stream");
//     let process_fut = writer.process(stream, ws_writer, ws_reader);

//     // Cancel the stream after 2 seconds
//     tokio::time::sleep(std::time::Duration::from_secs(2)).await;
//     tini.stream_cancel(&key).await.unwrap();

//     // process() response should show that stream was cancelled
//     let LlmOutput {
//         errors, cancelled, ..
//     } = process_fut.await;
//     assert!(cancelled);
//     assert!(errors.unwrap().last().unwrap().contains("cancelled"));

//     // Stream should be deleted after cancel
//     assert!(!tini.stream_exists(&key).await.unwrap());
// }

// #[tokio::test]
// async fn stream_writer_usage_tracking() {
//     let user_id = Uuid::new_v4();
//     let session_id = Uuid::new_v4();
//     let tini = setup_tini_client();
//     let (key, ws, mut writer) = create_test_writer(&user_id, &session_id).await;
//     let (ws_writer, ws_reader) = ws.split();

//     assert!(tini.stream_exists(&key).await.unwrap());

//     // Create a stream with usage information
//     let usage_stream = tokio_stream::iter(vec![
//         Ok(LlmStreamChunk::Text("Hello".into())),
//         Ok(LlmStreamChunk::Usage(LlmUsage {
//             input_tokens: Some(10),
//             output_tokens: Some(5),
//             cost: Some(0.001),
//         })),
//         Ok(LlmStreamChunk::Text(" World".into())),
//         Ok(LlmStreamChunk::Usage(LlmUsage {
//             input_tokens: None,     // Should not override
//             output_tokens: Some(7), // Should update
//             cost: Some(0.002),      // Should update
//         })),
//     ]);

//     let stream: LlmStream = Box::pin(usage_stream);
//     let LlmOutput {
//         text,
//         usage,
//         cancelled,
//         ..
//     } = writer.process(stream, ws_writer, ws_reader).await;

//     assert!(text.is_some());
//     assert_eq!(text.unwrap(), "Hello World");

//     assert!(usage.is_some());
//     let usage = usage.unwrap();
//     assert_eq!(usage.input_tokens, Some(10));
//     assert_eq!(usage.output_tokens, Some(7));
//     assert_eq!(usage.cost, Some(0.002));

//     assert!(!cancelled);

//     tini.stream_end(&key).await.ok();
// }

// #[tokio::test]
// async fn redis_stream_entries() {
//     let user_id = Uuid::new_v4();
//     let session_id = Uuid::new_v4();
//     let tini = setup_tini_client();
//     let (key, ws, mut writer) = create_test_writer(&user_id, &session_id).await;
//     let (ws_writer, ws_reader) = ws.split();

//     assert!(tini.stream_exists(&key).await.unwrap());

//     // Verify start event was written
//     let info = tini
//         .stream_info(&key)
//         .await
//         .expect("Failed to check stream")
//         .expect("Stream not found");
//     assert_eq!(info.length, 1);

//     // Create a simple stream
//     let stream = tokio_stream::iter(vec![Ok(LlmStreamChunk::Text("Test chunk".into()))]).boxed();
//     writer.process(stream, ws_writer, ws_reader).await;
//     drop(writer);

//     // Should have start + text entries
//     tokio::time::sleep(std::time::Duration::from_secs(1)).await;
//     let info = tini
//         .stream_info(&key)
//         .await
//         .expect("Failed to check stream")
//         .expect("Stream not found");
//     assert_eq!(info.length, 2);

//     tini.stream_end(&key).await.ok();
// }
