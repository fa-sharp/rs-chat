import { useQueryClient } from "@tanstack/react-query";
import { useCallback, useContext } from "react";

import type { components } from "../api/types";
import { ChatStreamContext } from "./streamManager";

/** Hook to stream a chat, get chat stream status, etc. */
export const useStreamingChats = () => {
  const queryClient = useQueryClient();
  const { streamedChats, startStreamWithInput } = useContext(ChatStreamContext);

  /** Start stream + optimistic update of user message */
  const onUserSubmit = useCallback(
    async (
      sessionId: string,
      input: components["schemas"]["SendChatInput"],
    ) => {
      startStreamWithInput(sessionId, input);

      if (input.message) {
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
      }
    },
    [startStreamWithInput, queryClient],
  );

  return {
    onUserSubmit,
    streamedChats,
  };
};
