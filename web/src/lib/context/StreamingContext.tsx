import { createContext, useContext, useState, useCallback } from "react";
import { useQueryClient } from "@tanstack/react-query";
import type { components } from "../api/types";
import { streamChat } from "../api/chat";
import { chatSessionQueryKey } from "../api/session";

export interface StreamedChat {
  content: string;
  error?: string;
  status: "streaming" | "completed";
}

/** Hook to stream a chat, get chat stream status, etc. */
export const useStreamingChats = () => {
  const queryClient = useQueryClient();
  const { streamedChats, startStream } = useContext(ChatStreamContext);

  // Start stream + optimistic update of user message
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
    [startStream],
  );

  return {
    onUserSubmit,
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

  const invalidateSession = useCallback(async (sessionId: string) => {
    await Promise.allSettled([
      queryClient.invalidateQueries({
        queryKey: chatSessionQueryKey(sessionId),
      }),
      queryClient.invalidateQueries({
        queryKey: ["recentChatSessions"],
      }),
    ]);
  }, []);

  /** Refetch chat session with 1 retry */
  const refetchSessionForNewResponse = useCallback(
    async (sessionId: string) => {
      const retryDelay = 1000; // 1 second
      try {
        // Refetch chat session
        await invalidateSession(sessionId);
        // Check if the chat session has been updated with the new assistant response
        const updatedData = queryClient.getQueryData<{
          messages: components["schemas"]["ChatRsMessage"][];
        }>(["chatSession", { sessionId }]);
        const hasNewAssistantMessage = updatedData?.messages?.some(
          (msg) =>
            msg.role === "Assistant" &&
            new Date(msg.created_at).getTime() > Date.now() - 5000, // Within last 5 seconds
        );
        // Retry if needed
        if (!hasNewAssistantMessage) {
          await new Promise((resolve) => setTimeout(resolve, retryDelay));
          await invalidateSession(sessionId);
        }
      } catch (error) {
        console.error("Error refetching chat session:", error);
        await invalidateSession(sessionId);
      }
    },
    [invalidateSession],
  );

  /** Start a new chat stream */
  const startStream = useCallback(
    (sessionId: string, input: components["schemas"]["SendChatInput"]) => {
      clearChat(sessionId);
      setChatStatus(sessionId, "streaming");
      let stream = streamChat(sessionId, input, {
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
    [],
  );

  return {
    startStream,
    streamedChats,
  };
};
