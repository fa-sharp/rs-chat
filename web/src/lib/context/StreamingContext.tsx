import { useQueryClient } from "@tanstack/react-query";
import { createContext, useCallback, useContext, useState } from "react";

import { streamChat } from "../api/chat";
import { chatSessionQueryKey, recentSessionsQueryKey } from "../api/session";
import type { components } from "../api/types";

export interface StreamedChat {
  content: string;
  error?: string;
  status: "streaming" | "completed";
}

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
              timestamp: new Date(),
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

/** Manage ongoing chat streams */
const useChatStreamManager = () => {
  const [streamedChats, setStreamedChats] = useState<{
    [sessionId: string]: StreamedChat | undefined;
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
  const refetchSessionForNewResponse = useCallback(
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
          refetchSessionForNewResponse(sessionId).then(() =>
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
      refetchSessionForNewResponse,
    ],
  );

  return {
    startStream,
    streamedChats,
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
