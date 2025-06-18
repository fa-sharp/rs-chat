import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { client } from "./client";

export async function getChatSession(sessionId: string) {
  const response = await client.GET("/session/{session_id}", {
    params: { path: { session_id: sessionId } },
  });
  if (response.error) {
    throw new Error(response.error.message);
  }
  return response.data;
}

export const chatSessionQueryKey = (sessionId: string) => [
  "chatSession",
  { sessionId },
];

export const useGetChatSession = (sessionId: string) =>
  useQuery({
    enabled: !!sessionId,
    queryKey: chatSessionQueryKey(sessionId),
    queryFn: () => getChatSession(sessionId),
  });

export async function getRecentChatSessions() {
  const response = await client.GET("/session/");
  if (response.error) {
    throw new Error(response.error.message);
  }
  return response.data;
}

export const useGetRecentChatSessions = () =>
  useQuery({
    queryKey: ["recentChatSessions"],
    queryFn: () => getRecentChatSessions(),
  });

export const useCreateChatSession = () => {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async () => {
      const response = await client.POST("/session/");
      if (response.error) {
        throw new Error(response.error.message);
      }
      return response.data;
    },
    onSuccess: () =>
      queryClient.invalidateQueries({ queryKey: ["recentChatSessions"] }),
  });
};

export const useDeleteChatMessage = () => {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: async ({
      sessionId,
      messageId,
    }: {
      sessionId: string;
      messageId: string;
    }) => {
      const response = await client.DELETE(
        "/session/{session_id}/{message_id}",
        {
          params: { path: { session_id: sessionId, message_id: messageId } },
        },
      );
      if (response.error) {
        throw new Error(response.error.message);
      }
    },
    onSettled: (_data, _error, { sessionId }) =>
      queryClient.invalidateQueries({
        queryKey: chatSessionQueryKey(sessionId),
      }),
  });
};
