import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { client } from "./client";
import type { components } from "./types";

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

export const recentSessionsQueryKey = ["recentChatSessions"];

export async function getRecentChatSessions() {
  const response = await client.GET("/session/");
  if (response.error) {
    throw new Error(response.error.message);
  }
  return response.data;
}

export const useGetRecentChatSessions = () =>
  useQuery({
    queryKey: recentSessionsQueryKey,
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
      queryClient.invalidateQueries({ queryKey: recentSessionsQueryKey }),
  });
};

export const useUpdateChatSession = () => {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async ({
      sessionId,
      data,
    }: {
      sessionId: string;
      data: components["schemas"]["UpdateSessionInput"];
    }) => {
      const response = await client.PATCH("/session/{session_id}", {
        params: { path: { session_id: sessionId } },
        body: data,
      });
      if (response.error) throw new Error(response.error.message);
      return response.data;
    },
    onMutate: ({ data, sessionId }) => {
      const previousData = queryClient.getQueryData(
        chatSessionQueryKey(sessionId),
      );
      queryClient.setQueryData(
        chatSessionQueryKey(sessionId),
        (oldData: { session: components["schemas"]["ChatRsSession"] }) => {
          if (!oldData) return oldData;
          return {
            ...oldData,
            session: {
              ...oldData.session,
              title: data.title,
            },
          };
        },
      );
      return previousData;
    },
    onSettled: (data) => {
      if (data)
        queryClient.invalidateQueries({
          queryKey: chatSessionQueryKey(data.session_id),
        });
      queryClient.invalidateQueries({ queryKey: recentSessionsQueryKey });
    },
  });
};

export const useDeleteChatSession = () => {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: async ({ sessionId }: { sessionId: string }) => {
      const response = await client.DELETE("/session/{session_id}", {
        params: { path: { session_id: sessionId } },
      });
      if (response.error) throw new Error(response.error.message);
      return response.data;
    },
    onMutate: ({ sessionId }) => {
      const previousData = queryClient.getQueryData(recentSessionsQueryKey);
      queryClient.setQueryData(
        recentSessionsQueryKey,
        (oldData: components["schemas"]["ChatRsSession"][]) => {
          if (!oldData) return oldData;
          return oldData.filter((session) => session.id !== sessionId);
        },
      );
      return previousData;
    },
    onSettled: () =>
      queryClient.invalidateQueries({ queryKey: recentSessionsQueryKey }),
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
