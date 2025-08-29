import { useCallback, useState } from "react";

import type { components } from "../api/types";

export interface StreamingChat {
  text: string;
  errors: string[];
  toolCalls: components["schemas"]["ChatRsToolCall"][];
  status: "streaming" | "completed";
}
const initialChatState = (): StreamingChat => ({
  text: "",
  errors: [],
  toolCalls: [],
  status: "streaming",
});

export interface StreamingToolExecution {
  result: string;
  logs: string[];
  debugLogs: string[];
  error?: string;
  status: "streaming" | "completed" | "error";
}
const initialToolState = (): StreamingToolExecution => ({
  result: "",
  logs: [],
  debugLogs: [],
  status: "streaming",
});

export function useStreamManagerState() {
  const [currentChatStreams, setCurrentChatStreams] = useState<{
    [sessionId: string]: StreamingChat | undefined;
  }>({});

  const [streamedTools, setStreamedTools] = useState<{
    [toolCallId: string]: StreamingToolExecution | undefined;
  }>({});

  const [activeToolStreams, setActiveToolStreams] = useState<{
    [toolCallId: string]: { close: () => void } | undefined;
  }>({});

  const initSession = useCallback((sessionId: string) => {
    setCurrentChatStreams((prev) => ({
      ...prev,
      [sessionId]: initialChatState(),
    }));
  }, []);

  const addTextChunk = useCallback((sessionId: string, text: string) => {
    setCurrentChatStreams((prev) => ({
      ...prev,
      [sessionId]: {
        ...(prev[sessionId] || initialChatState()),
        text: (prev[sessionId]?.text || "") + text,
      },
    }));
  }, []);

  const addErrorChunk = useCallback((sessionId: string, error: string) => {
    setCurrentChatStreams((prev) => ({
      ...prev,
      [sessionId]: {
        ...(prev[sessionId] || initialChatState()),
        errors: [...(prev[sessionId]?.errors || []), error],
      },
    }));
  }, []);

  const addToolCallChunk = useCallback(
    (sessionId: string, toolCall: string) => {
      setCurrentChatStreams((prev) => ({
        ...prev,
        [sessionId]: {
          ...(prev[sessionId] || initialChatState()),
          toolCalls: [
            ...(prev[sessionId]?.toolCalls || []),
            JSON.parse(toolCall),
          ],
        },
      }));
    },
    [],
  );

  const setSessionCompleted = useCallback((sessionId: string) => {
    setCurrentChatStreams((prev) => ({
      ...prev,
      [sessionId]: {
        ...(prev[sessionId] || initialChatState()),
        status: "completed",
      },
    }));
  }, []);

  const clearSession = useCallback((sessionId: string) => {
    setCurrentChatStreams((prev) => ({
      ...prev,
      [sessionId]: undefined,
    }));
  }, []);

  const addActiveToolStream = useCallback(
    (toolCallId: string, close: () => void) => {
      setActiveToolStreams((prev) => ({
        ...prev,
        [toolCallId]: { close },
      }));
    },
    [],
  );

  /** Add tool execution result chunk */
  const addToolResult = useCallback((toolCallId: string, result: string) => {
    setStreamedTools((prev) => ({
      ...prev,
      [toolCallId]: {
        ...(prev?.[toolCallId] || initialToolState()),
        result: (prev?.[toolCallId]?.result || "") + result,
      },
    }));
  }, []);

  /** Add tool execution log */
  const addToolLog = useCallback((toolCallId: string, log: string) => {
    setStreamedTools((prev) => ({
      ...prev,
      [toolCallId]: {
        ...(prev?.[toolCallId] || initialToolState()),
        logs: [...(prev?.[toolCallId]?.logs || []), log],
      },
    }));
  }, []);

  /** Add tool execution debug log */
  const addToolDebug = useCallback((toolCallId: string, debug: string) => {
    setStreamedTools((prev) => ({
      ...prev,
      [toolCallId]: {
        ...(prev?.[toolCallId] || initialToolState()),
        debugLogs: [...(prev?.[toolCallId]?.debugLogs || []), debug],
      },
    }));
  }, []);

  /** Add tool execution error */
  const addToolError = useCallback((toolCallId: string, error: string) => {
    setStreamedTools((prev) => ({
      ...prev,
      [toolCallId]: {
        ...(prev?.[toolCallId] || initialToolState()),
        error,
        status: "error",
      },
    }));
  }, []);

  /** Set tool execution status */
  const setToolStatus = useCallback(
    (toolCallId: string, status: "streaming" | "completed" | "error") => {
      setStreamedTools((prev) => ({
        ...prev,
        [toolCallId]: {
          ...(prev?.[toolCallId] || initialToolState()),
          status,
        },
      }));
    },
    [],
  );

  /** Clear active tool execution */
  const clearTool = useCallback((toolCallId: string) => {
    setStreamedTools((prev) => ({
      ...prev,
      [toolCallId]: undefined,
    }));
    setActiveToolStreams((prev) => ({
      ...prev,
      [toolCallId]: undefined,
    }));
  }, []);

  return {
    currentChatStreams,
    initSession,
    addTextChunk,
    addErrorChunk,
    addToolCallChunk,
    clearSession,
    setSessionCompleted,
    addToolLog,
    addToolDebug,
    addToolError,
    addToolResult,
    activeToolStreams,
    addActiveToolStream,
    clearTool,
    setToolStatus,
    streamedTools,
  };
}
