import { createContext, useCallback, useEffect } from "react";

import { createChatStream } from "@/lib/api/chat";
import { client } from "@/lib/api/client";
import { streamToolExecution } from "@/lib/api/tool";
import type { components } from "@/lib/api/types";
import { useStreamManagerData } from "./streamManagerData";
import { useStreamManagerState } from "./streamManagerState";

export function useStreamManager() {
  const {
    currentChatStreams,
    initSession,
    clearSession,
    setSessionCompleted,
    streamedTools,
    addTextChunk,
    addToolCallChunk,
    addErrorChunk,
    activeToolStreams,
    addActiveToolStream,
    addToolLog,
    addToolDebug,
    addToolResult,
    addToolError,
    clearTool,
    setToolStatus,
  } = useStreamManagerState();

  const {
    refetchSessionForNewAssistantMessage,
    refetchSessionForNewToolMessage,
    serverStreams,
  } = useStreamManagerData();

  const startStream = useCallback(
    async (sessionId: string) => {
      clearSession(sessionId);
      initSession(sessionId);

      createChatStream(sessionId, {
        onText: (text) => addTextChunk(sessionId, text),
        onToolCall: (toolCall) => addToolCallChunk(sessionId, toolCall),
        onError: (error) => addErrorChunk(sessionId, error),
      })
        .then((chatStream) => {
          chatStream.stream().finally(() => {
            setSessionCompleted(sessionId);
            refetchSessionForNewAssistantMessage(sessionId)
              .then(() => clearSession(sessionId))
              .catch((err: unknown) => {
                console.error("Error refetching messages:", err);
                addErrorChunk(sessionId, "Error refetching chat session.");
              });
          });
        })
        .catch((err: Error) => {
          addErrorChunk(sessionId, `Error starting stream: ${err.message}`);
          setSessionCompleted(sessionId);
          console.error("Error starting stream:", err.message);
        });
    },
    [
      clearSession,
      setSessionCompleted,
      addToolCallChunk,
      addTextChunk,
      addErrorChunk,
      initSession,
      refetchSessionForNewAssistantMessage,
    ],
  );

  // Automatically start any ongoing chat streams
  useEffect(() => {
    if (!serverStreams) return;
    for (const sessionId of serverStreams.sessions) {
      if (!currentChatStreams[sessionId]) {
        startStream(sessionId);
      }
    }
  }, [serverStreams, currentChatStreams, startStream]);

  const startStreamWithInput = useCallback(
    async (
      sessionId: string,
      input: components["schemas"]["SendChatInput"],
    ) => {
      initSession(sessionId);
      const response = await client.POST("/chat/{session_id}", {
        params: { path: { session_id: sessionId } },
        body: input,
      });
      if (response.error) {
        addErrorChunk(sessionId, response.error.message);
        setSessionCompleted(sessionId);
        return;
      }
      await startStream(sessionId);
    },
    [initSession, addErrorChunk, startStream, setSessionCompleted],
  );

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
      addActiveToolStream(toolCallId, stream.close);

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
      addActiveToolStream,
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
    startStreamWithInput,
    streamedChats: currentChatStreams,
    streamedTools,
    startToolExecution,
    cancelToolExecution,
  };
}

export const ChatStreamContext = createContext<
  ReturnType<typeof useStreamManager>
>(
  //@ts-expect-error should be initialized
  null,
);
