import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { client } from "./client";
import { chatSessionQueryKey } from "./session";
import type { components } from "./types";

const queryKey = ["tools"];

export const useTools = () =>
  useQuery({
    queryKey,
    queryFn: async () => {
      const response = await client.GET("/tool/");
      if (response.error) {
        throw new Error(response.error.message);
      }
      return response.data;
    },
  });

export const useCreateTool = () => {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: async (toolInput: components["schemas"]["ToolInput"]) => {
      const response = await client.POST("/tool/", {
        body: toolInput,
      });
      if (response.error) {
        throw new Error(response.error.message);
      }
      return response.data;
    },
    onSettled: () => queryClient.invalidateQueries({ queryKey }),
  });
};

export const useDeleteTool = () => {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: async (toolId: string) => {
      const response = await client.DELETE("/tool/{tool_id}", {
        params: { path: { tool_id: toolId } },
      });
      if (response.error) {
        throw new Error(response.error.message);
      }
      return response.data;
    },
    onSettled: () => queryClient.invalidateQueries({ queryKey }),
  });
};

export const useExecuteTool = () => {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: async ({
      messageId,
      toolCallId,
    }: {
      messageId: string;
      toolCallId: string;
    }) => {
      const response = await client.POST(
        "/tool/execute/{message_id}/{tool_call_id}",
        {
          params: { path: { message_id: messageId, tool_call_id: toolCallId } },
        },
      );
      if (response.error) {
        throw new Error(response.error.message);
      }
      return response.data;
    },
    onSettled: (data) =>
      data &&
      queryClient.invalidateQueries({
        queryKey: chatSessionQueryKey(data.session_id),
      }),
    onSuccess: (data) => {
      // Optimistic update of tool messages
      queryClient.setQueryData<{
        messages: components["schemas"]["ChatRsMessage"][];
      }>(chatSessionQueryKey(data.session_id), (oldData) =>
        oldData
          ? {
              ...oldData,
              messages: [...oldData.messages, data],
            }
          : undefined,
      );
    },
  });
};
