import { useQueryClient } from "@tanstack/react-query";
import { useCallback } from "react";

import { useGetCurrentStreams } from "../api/chat";
import { chatSessionQueryKey, recentSessionsQueryKey } from "../api/session";
import type { components } from "../api/types";
import { useGetUser } from "../api/user";

export function useStreamManagerData() {
  const queryClient = useQueryClient();
  const { data: user } = useGetUser();
  const { data: serverStreams } = useGetCurrentStreams(!!user);

  const invalidateSession = useCallback(
    async (sessionId: string) => {
      await Promise.all([
        queryClient.invalidateQueries({
          queryKey: chatSessionQueryKey(sessionId),
        }),
        queryClient.invalidateQueries({
          queryKey: recentSessionsQueryKey,
        }),
        queryClient.invalidateQueries({
          queryKey: ["serverStreams"],
        }),
      ]);
    },
    [queryClient],
  );

  const refetchSessionForNewAssistantMessage = useCallback(
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

  return {
    serverStreams,
    refetchSessionForNewAssistantMessage,
    refetchSessionForNewToolMessage,
  };
}
