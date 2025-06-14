import { useCallback, useEffect, useState } from "react";
import { useQueryClient } from "@tanstack/react-query";
import { SSE } from "sse.js";
import type { components } from "./types";

export const useStreamingChat = (sessionId: string) => {
  const queryClient = useQueryClient();
  const [streamInput, setStreamInput] = useState("");
  const streamedChatResponse = useStreamedResponse(streamInput, sessionId);

  useEffect(() => {
    if (streamedChatResponse.status === "complete") {
      const maxRetries = 3;
      const retryDelay = 1000; // 1 second

      const updateChatSession = async (retryCount = 0) => {
        try {
          // Refetch chat session
          await queryClient.invalidateQueries({
            queryKey: ["chatSession", { sessionId }],
          });

          // Check if the chat session has been updated
          const updatedData = queryClient.getQueryData<{
            messages: components["schemas"]["ChatRsMessage"][];
          }>(["chatSession", { sessionId }]);
          const hasNewAssistantMessage = updatedData?.messages?.some(
            (msg) =>
              msg.role === "Assistant" &&
              new Date(msg.created_at).getTime() > Date.now() - 10000, // Within last 10 seconds
          );
          if (!hasNewAssistantMessage && retryCount < maxRetries) {
            setTimeout(() => updateChatSession(retryCount + 1), retryDelay);
            return;
          }
          setStreamInput("");
        } catch (error) {
          if (retryCount < maxRetries) {
            setTimeout(() => updateChatSession(retryCount + 1), retryDelay);
          } else {
            setStreamInput("");
          }
        }
      };

      updateChatSession();
    }
  }, [streamedChatResponse.status]);

  const onUserSubmit = useCallback((message: string) => {
    setStreamInput(message);
    queryClient.setQueryData<{
      messages: components["schemas"]["ChatRsMessage"][];
    }>(["chatSession", { sessionId }], (oldData: any) => {
      if (!oldData) return {};
      return {
        ...oldData,
        messages: [
          ...oldData.messages,
          {
            id: crypto.randomUUID(),
            content: message,
            role: "User",
            timestamp: new Date(),
          },
        ],
      };
    });
  }, []);

  return {
    onUserSubmit,
    streamingStatus: streamedChatResponse.status,
    streamingMessage: streamedChatResponse.message,
    streamingErrors: streamedChatResponse.errors,
    isGenerating:
      streamedChatResponse.status === "pending" ||
      streamedChatResponse.status === "streaming",
  };
};

const useStreamedResponse = (input?: string, sessionId?: string) => {
  const [message, setMessage] = useState<string>("");
  const [errors, setErrors] = useState<string[]>([]);
  const [status, setStatus] = useState<
    "idle" | "pending" | "streaming" | "complete"
  >("idle");

  const resetState = () => {
    setMessage("");
    setStatus("idle");
  };

  useEffect(() => {
    if (!sessionId || !input) return;

    resetState();
    setErrors([]);
    setStatus("pending");

    const body: components["schemas"]["SendChatInput"] = {
      message: input,
      provider: "Lorem",
    };
    const source = new SSE(`/api/chat/${sessionId}`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      payload: JSON.stringify(body),
      debug: import.meta.env.DEV,
    });
    const chatListener = (event: { data: string }) => {
      setMessage((prev) => prev + event.data);
    };
    const errorListener = (
      error: string | { responseCode: number; data: string },
    ) => {
      setErrors((errors) => [
        ...errors,
        typeof error === "string" ? error : error.data,
      ]);
    };
    const stateListener = (e: { readyState: number }) => {
      if (e.readyState === SSE.OPEN) {
        setStatus("streaming");
      } else if (e.readyState === SSE.CLOSED) {
        setStatus("complete");
      }
    };

    source.addEventListener("chat", chatListener);
    source.addEventListener("error", errorListener);
    source.addEventListener("readystatechange", stateListener);

    return () => {
      source.removeEventListener("chat", chatListener);
      source.removeEventListener("error", errorListener);
      source.removeEventListener("readystatechange", stateListener);
      resetState();
    };
  }, [sessionId, input]);

  return { message, errors, status };
};
