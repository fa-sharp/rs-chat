import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { type ReadyStateEvent, SSE, type SSEvent } from "sse.js";

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

/** Stream tool execution via SSE */
export function streamToolExecution(
  messageId: string,
  toolCallId: string,
  {
    onResult,
    onLog,
    onDebug,
    onError,
  }: {
    onResult: (data: string) => void;
    onLog: (data: string) => void;
    onDebug: (data: string) => void;
    onError: (error: string) => void;
  },
) {
  const source = new SSE(`/api/tool/execute/${messageId}/${toolCallId}`, {
    method: "POST",
  });

  return {
    get readyState() {
      return source.readyState;
    },
    close() {
      source.close();
    },
    start: new Promise<void>((resolve, reject) => {
      const resultListener = (event: SSEvent) => {
        onResult(event.data);
      };

      const logListener = (event: SSEvent) => {
        onLog(event.data);
      };

      const debugListener = (event: SSEvent) => {
        onDebug(event.data);
      };

      const errorListener = (event: SSEvent & { responseCode?: number }) => {
        console.error("Error while streaming tool execution:", event);
        if (event.responseCode) {
          let data: string | undefined;
          try {
            data = JSON.parse(event.data).message;
          } catch {
            data = event.data;
          }
          if (typeof data === "string") {
            onError(data);
          } else {
            onError(`Error code ${event.responseCode}`);
          }
          reject();
        } else {
          onError(
            typeof event.data === "string" ? event.data : "Unknown error",
          );
        }
      };

      const endListener = (event: ReadyStateEvent) => {
        if (event.readyState === SSE.CLOSED) {
          source.removeEventListener("result", resultListener);
          source.removeEventListener("log", logListener);
          source.removeEventListener("debug", debugListener);
          source.removeEventListener("error", errorListener);
          source.removeEventListener("readystatechange", endListener);
          resolve();
        }
      };

      source.addEventListener("result", resultListener);
      source.addEventListener("log", logListener);
      source.addEventListener("debug", debugListener);
      source.addEventListener("error", errorListener);
      source.addEventListener("readystatechange", endListener);
    }),
  };
}
