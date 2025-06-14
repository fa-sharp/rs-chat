import { useEffect, useState } from "react";
import { SSE } from "sse.js";
import type { components } from "./types";

export const useStreamedChatResponse = (input?: string, sessionId?: string) => {
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
      provider: "Anthropic",
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
