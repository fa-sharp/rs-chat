import { useQueryClient } from "@tanstack/react-query";
import { createContext, useCallback, useContext, useState } from "react";

import { streamChat } from "../api/chat";
import { chatSessionQueryKey, recentSessionsQueryKey } from "../api/session";
import { streamToolExecution } from "../api/tool";
import type { components } from "../api/types";

export interface StreamedChat {
  content: string;
  error?: string;
  status: "streaming" | "completed";
}

export interface StreamedToolExecution {
  result: string;
  logs: string[];
  debugLogs: string[];
  error?: string;
  status: "streaming" | "completed" | "error";
}
const streamedToolExecutionInit = (): StreamedToolExecution => ({
  result: "",
  logs: [],
  debugLogs: [],
  status: "streaming",
});

/** Hook to stream a chat, get chat stream status, etc. */
export const useStreamingChats = () => {
  const queryClient = useQueryClient();
  const { streamedChats, startStream } = useContext(ChatStreamContext);

  /** Start stream + optimistic update of user message */
  const onUserSubmit = useCallback(
    (sessionId: string, input: components["schemas"]["SendChatInput"]) => {
      startStream(sessionId, input);
      if (!input.message) return;

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
              content: input.message,
              role: "User",
              created_at: new Date().toISOString(),
              session_id: sessionId,
              meta: {},
            },
          ],
        };
      });
    },
    [startStream, queryClient],
  );

  return {
    onUserSubmit,
    streamedChats,
  };
};

/** Hook to stream tool executions */
export const useStreamingTools = () => {
  const { streamedTools, startToolExecution, cancelToolExecution } =
    useContext(ChatStreamContext);

  /** Execute a tool with streaming */
  const onToolExecute = useCallback(
    (messageId: string, sessionId: string, toolCallId: string) => {
      startToolExecution(messageId, sessionId, toolCallId);
    },
    [startToolExecution],
  );

  /** Cancel a tool execution */
  const onToolCancel = useCallback(
    (sessionId: string, toolCallId: string) => {
      cancelToolExecution(sessionId, toolCallId);
    },
    [cancelToolExecution],
  );

  return {
    streamedTools,
    onToolExecute,
    onToolCancel,
  };
};

/** Manage ongoing chat streams and tool executions */
const useChatStreamManager = () => {
  const [streamedChats, setStreamedChats] = useState<{
    [sessionId: string]: StreamedChat | undefined;
  }>({});

  const [streamedTools, setStreamedTools] = useState<{
    [toolCallId: string]: StreamedToolExecution | undefined;
  }>({});

  const [activeToolStreams, setActiveToolStreams] = useState<{
    [toolCallId: string]: { close: () => void } | undefined;
  }>({});

  const addChatPart = useCallback((sessionId: string, part: string) => {
    setStreamedChats((prev) => ({
      ...prev,
      [sessionId]: {
        content: (prev?.[sessionId]?.content || "") + part,
        error: prev?.[sessionId]?.error,
        status: "streaming",
      },
    }));
  }, []);

  const addChatError = useCallback((sessionId: string, error: string) => {
    setStreamedChats((prev) => ({
      ...prev,
      [sessionId]: {
        content: prev?.[sessionId]?.content || "",
        status: "streaming",
        error,
      },
    }));
  }, []);

  const setChatStatus = useCallback(
    (sessionId: string, status: "streaming" | "completed") => {
      setStreamedChats((prev) => ({
        ...prev,
        [sessionId]: {
          status,
          content: prev?.[sessionId]?.content || "",
          error: prev?.[sessionId]?.error,
        },
      }));
    },
    [],
  );

  const clearChat = useCallback((sessionId: string) => {
    setStreamedChats((prev) => ({
      ...prev,
      [sessionId]: undefined,
    }));
  }, []);

  const queryClient = useQueryClient();

  const invalidateSession = useCallback(
    async (sessionId: string) => {
      await Promise.allSettled([
        queryClient.invalidateQueries({
          queryKey: chatSessionQueryKey(sessionId),
        }),
        queryClient.invalidateQueries({
          queryKey: recentSessionsQueryKey,
        }),
      ]);
    },
    [queryClient],
  );

  /** Refetch chat session for the new assistant message */
  const refetchSessionForNewAssistantResponse = useCallback(
    async (sessionId: string) => {
      const retryDelay = 1000; // 1 second
      try {
        // Refetch chat session with retry loop
        let hasNewAssistantMessage = false;
        let retryCount = 0;
        const maxRetries = 3;

        while (!hasNewAssistantMessage && retryCount < maxRetries) {
          await invalidateSession(sessionId);

          // Check if the chat session has been updated with the new assistant response
          const updatedData = queryClient.getQueryData<{
            messages: components["schemas"]["ChatRsMessage"][];
          }>(["chatSession", { sessionId }]);
          hasNewAssistantMessage =
            updatedData?.messages?.some(
              (msg) =>
                msg.role === "Assistant" &&
                !msg.meta.assistant?.partial &&
                new Date(msg.created_at).getTime() > Date.now() - 5000, // Within last 5 seconds
            ) || false;

          // Retry if no new assistant message
          if (!hasNewAssistantMessage) {
            retryCount++;
            if (retryCount < maxRetries) {
              await new Promise((resolve) => setTimeout(resolve, retryDelay));
            }
          }
        }
      } catch (error) {
        console.error("Error refetching chat session:", error);
        await invalidateSession(sessionId);
      }
    },
    [invalidateSession, queryClient],
  );

  /** Refetch chat session for the new tool message */
  const refetchSessionForNewToolMessage = useCallback(
    async (sessionId: string, toolCallId: string) => {
      const retryDelay = 1000; // 1 second
      try {
        let hasNewToolMessage = false;
        let retryCount = 0;
        const maxRetries = 3;

        while (!hasNewToolMessage && retryCount < maxRetries) {
          await invalidateSession(sessionId);

          const updatedData = queryClient.getQueryData<{
            messages: components["schemas"]["ChatRsMessage"][];
          }>(["chatSession", { sessionId }]);
          hasNewToolMessage =
            updatedData?.messages?.some(
              (msg) =>
                msg.role === "Tool" && msg.meta.tool_call?.id === toolCallId,
            ) || false;

          if (!hasNewToolMessage) {
            retryCount++;
            if (retryCount < maxRetries) {
              await new Promise((resolve) => setTimeout(resolve, retryDelay));
            }
          }
        }
      } catch (error) {
        console.error("Error refetching chat session:", error);
        await invalidateSession(sessionId);
      }
    },
    [invalidateSession, queryClient],
  );

  /** Start a new chat stream */
  const startStream = useCallback(
    (sessionId: string, input: components["schemas"]["SendChatInput"]) => {
      clearChat(sessionId);
      setChatStatus(sessionId, "streaming");
      const stream = streamChat(sessionId, input, {
        onPart: (part) => {
          addChatPart(sessionId, part);
        },
        onError: (error) => {
          addChatError(sessionId, error);
        },
      });
      stream.start
        .then(() => {
          refetchSessionForNewAssistantResponse(sessionId).then(() =>
            clearChat(sessionId),
          );
        })
        .catch(() => {
          invalidateSession(sessionId).then(() =>
            setChatStatus(sessionId, "completed"),
          );
        });
    },
    [
      clearChat,
      addChatPart,
      addChatError,
      setChatStatus,
      invalidateSession,
      refetchSessionForNewAssistantResponse,
    ],
  );

  /** Add tool execution result chunk */
  const addToolResult = useCallback((toolCallId: string, result: string) => {
    setStreamedTools((prev) => ({
      ...prev,
      [toolCallId]: {
        ...(prev?.[toolCallId] || streamedToolExecutionInit()),
        result: (prev?.[toolCallId]?.result || "") + result,
      },
    }));
  }, []);

  /** Add tool execution log */
  const addToolLog = useCallback((toolCallId: string, log: string) => {
    setStreamedTools((prev) => ({
      ...prev,
      [toolCallId]: {
        ...(prev?.[toolCallId] || streamedToolExecutionInit()),
        logs: [...(prev?.[toolCallId]?.logs || []), log],
      },
    }));
  }, []);

  /** Add tool execution debug log */
  const addToolDebug = useCallback((toolCallId: string, debug: string) => {
    setStreamedTools((prev) => ({
      ...prev,
      [toolCallId]: {
        ...(prev?.[toolCallId] || streamedToolExecutionInit()),
        debugLogs: [...(prev?.[toolCallId]?.debugLogs || []), debug],
      },
    }));
  }, []);

  /** Add tool execution error */
  const addToolError = useCallback((toolCallId: string, error: string) => {
    setStreamedTools((prev) => ({
      ...prev,
      [toolCallId]: {
        ...(prev?.[toolCallId] || streamedToolExecutionInit()),
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
          ...(prev?.[toolCallId] || streamedToolExecutionInit()),
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

  /** Start tool execution stream */
  const startToolExecution = useCallback(
    (messageId: string, sessionId: string, toolCallId: string) => {
      const stream = streamToolExecution(messageId, toolCallId, {
        onResult: (data) => addToolResult(toolCallId, data),
        onLog: (data) => addToolLog(toolCallId, data),
        onDebug: (data) => addToolDebug(toolCallId, data),
        onError: (error) => addToolError(toolCallId, error),
      });

      clearTool(toolCallId);
      setToolStatus(toolCallId, "streaming");
      setActiveToolStreams((prev) => ({
        ...prev,
        [toolCallId]: { close: stream.close },
      }));

      stream.start
        .then(() => setToolStatus(toolCallId, "completed"))
        .catch(() => setToolStatus(toolCallId, "error"))
        .finally(() => {
          refetchSessionForNewToolMessage(sessionId, toolCallId).then(() => {
            clearTool(toolCallId);
          });
        });
    },
    [
      setToolStatus,
      addToolResult,
      addToolLog,
      addToolDebug,
      addToolError,
      clearTool,
      refetchSessionForNewToolMessage,
    ],
  );

  /** Cancel tool execution */
  const cancelToolExecution = useCallback(
    (sessionId: string, toolCallId: string) => {
      const activeStream = activeToolStreams[toolCallId];
      if (activeStream) {
        activeStream.close();
        refetchSessionForNewToolMessage(sessionId, toolCallId).then(() => {
          clearTool(toolCallId);
        });
      }
    },
    [activeToolStreams, clearTool, refetchSessionForNewToolMessage],
  );

  return {
    startStream,
    streamedChats,
    streamedTools,
    startToolExecution,
    cancelToolExecution,
  };
};

const ChatStreamContext = createContext<
  ReturnType<typeof useChatStreamManager>
>(
  //@ts-expect-error should be initialized
  null,
);

export const ChatStreamProvider = ({
  children,
}: {
  children: React.ReactNode;
}) => {
  const chatStreamManager = useChatStreamManager();

  return (
    <ChatStreamContext.Provider value={chatStreamManager}>
      {children}
    </ChatStreamContext.Provider>
  );
};
