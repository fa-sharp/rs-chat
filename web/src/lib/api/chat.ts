import { useQuery } from "@tanstack/react-query";
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
) {
  const abortController = new AbortController();
  const res = await client.GET("/chat/{session_id}/stream", {
    params: { path: { session_id: sessionId } },
    parseAs: "stream",
    signal: abortController.signal,
  });
  if (res.error) {
    throw new Error(res.error.message);
  }
  if (!res.data) {
    throw new Error("No data received");
  }

  return {
    stream: async () => {
      if (!res.data) return;
      const eventStream = res.data
        .pipeThrough(new TextDecoderStream())
        .pipeThrough(new EventSourceParserStream())
        .getReader();
      loop: while (true) {
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
          case "ping":
            break;
          case "end":
          case "cancel":
            break loop;
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
