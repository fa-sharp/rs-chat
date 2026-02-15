import { useMutation, useQuery } from "@tanstack/react-query";
import { EventSourceParserStream } from "eventsource-parser/stream";

import { client } from "./client";

async function getCurrentStreams() {
  const res = await client.GET("/chat/streams");
  if (res.error) {
    throw new Error(res.error.message);
  }
  return res.data;
}

export const useGetCurrentStreams = (enabled: boolean) =>
  useQuery({
    enabled,
    queryKey: ["serverStreams"],
    queryFn: getCurrentStreams,
  });

export async function createChatStream(
  sessionId: string,
  {
    onText,
    onToolCall,
    onError,
  }: {
    onText: (part: string) => void;
    onToolCall: (toolCall: string) => void;
    onError: (error: string) => void;
  },
  streamAccess?: {
    url: string;
    token: string;
  },
) {
  if (!streamAccess) {
    const res = await client.GET("/chat/{session_id}/stream", {
      params: { path: { session_id: sessionId } },
    });
    if (res.error) {
      throw new Error(res.error.message);
    }
    streamAccess = res.data;
  }

  const streamUrl = new URL(streamAccess.url);
  streamUrl.searchParams.append("token", streamAccess.token);
  const abortController = new AbortController();
  const sseStream = await fetch(streamUrl, {
    signal: abortController.signal,
  });
  if (!sseStream.ok) {
    throw new Error(
      `Failed to fetch SSE stream, status ${sseStream.status}, message: ${await sseStream.text()}`,
    );
  }
  if (!sseStream.body) {
    throw new Error("No data received from SSE stream");
  }

  return {
    stream: async () => {
      if (!sseStream.body) return;
      const eventStream = sseStream.body
        .pipeThrough(new TextDecoderStream())
        .pipeThrough(new EventSourceParserStream())
        .getReader();

      while (true) {
        const { done, value } = await eventStream.read();
        if (done) break;

        switch (value.event) {
          case "text":
            onText(value.data);
            break;
          case "error":
            onError(value.data);
            break;
          case "tool_call":
            onToolCall(value.data);
            break;
          case "start":
          case "pending_tool_call":
          case "ping":
            break;
          case "end":
          case "cancel":
            break;
          default:
            console.warn(`Unknown event type: ${value.event}`);
            break;
        }
      }
      try {
        abortController.abort();
      } catch (error) {
        console.warn("Error closing event stream:", error);
      }
    },
  };
}

export const useCancelChatStream = (sessionId?: string) =>
  useMutation({
    mutationFn: async () => {
      if (!sessionId) return;
      const res = await client.POST("/chat/{session_id}/cancel", {
        params: { path: { session_id: sessionId } },
      });
      if (res.error) {
        throw new Error(res.error.message);
      }
    },
  });
