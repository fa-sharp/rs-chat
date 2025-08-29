# RsChat Architecture

## Overview

RsChat is a real-time chat application that provides resumable streaming conversations with LLM providers. The architecture is designed for high performance, scalability across multiple server instances, and resilient streaming that can survive network interruptions.

## Core Architecture

### Frontend (React/TypeScript)
- **Location**: `web/`
- **Streaming**: Server-Sent Events (SSE)
- **State Management**: React Context for streaming state (`web/src/lib/context/StreamingContext.tsx`)
- **API Integration**: Type-safe API calls (generated from OpenAPI: `web/src/lib/api/types.d.ts`)

### Backend (Rust/Rocket)
- **Location**: `server/`
- **Framework**: Rocket with async/await support
- **Database**: PostgreSQL for persistent storage
- **Cache/Streaming**: Redis for stream management and caching

## LLM Streaming Architecture

### Dual-Stream Approach

RsChat uses a hybrid streaming architecture that provides both real-time performance and cross-instance resumability:

1. **Server**: Redis Streams for resumability and multi-instance support
2. **Client**: Server-Sent Events (SSE) read from the Redis streams

### Key Components

#### 1. LlmStreamWriter (`server/src/stream/llm_writer.rs`)

The core component that processes LLM provider streams and manages Redis stream output.

**Key Features:**
- **Batching**: Accumulates chunks from the provider stream, up to a max length or timeout
- **Background Pings**: Sends regular keepalive pings
- **Database Integration**: Saves final responses to PostgreSQL

#### 2. Redis and SSE Stream Structure

**Redis Key for Chat Streams**: `user:{user_id}:chat:{session_id}`

**Chat Stream Message Types**:
- `start`: Stream initialization
- `text`: Accumulated text chunks
- `tool_call`: LLM tool invocations (JSON stringified)
- `error`: Error messages
- `ping`: Keepalive messages
- `end`: Stream completion
- `cancel`: Stream cancellation

#### 3. Stream Lifecycle

```
Client Request → SSE Connection → LlmStreamWriter.create()
                     ↓
LLM Provider Stream → Batching Data Chunks → Redis XADD
                     ↓
Background Ping Task (intervals)
                     ↓
Stream End → Database Save → Redis DEL
```


### Resumability Features

#### Cross-Instance Support
- Redis streams provide shared state across server instances
- Background ping tasks maintain stream liveness
- Stream cancellation detected via Redis XADD failures

## Data Flow

### 1. New Chat Request
```
Client → POST /api/chat/{session_id}
       → Send request to LLM Provider
       → SSE Response Stream created
       → LlmStreamWriter.create()
       → Redis Stream created
```

### 2. Stream Processing
```
LLM Chunk → Process text, tool calls, usage, and error chunks
          → Batching Logic
          → Redis XADD (if conditions met)
          → Continue SSE Stream
```

### 3. Stream Completion
```
LLM End → Final Database Save
        → Redis Stream End Event
        → Redis Stream Cleanup
        → SSE Connection Close
```

### 4. Reconnection/Resume
```
Client Reconnect → Check ongoing streams via GET /api/chat/streams
                 → Reconnect to stream (if active)
```
