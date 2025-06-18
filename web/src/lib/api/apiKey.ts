import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { client } from "./client";
import type { components } from "./types";

export const useApiKeys = () =>
  useQuery({
    queryKey: ["apiKeys"],
    queryFn: async () => {
      const response = await client.GET("/api_key/");
      if (response.error) {
        throw new Error(response.error.message);
      }
      return response.data;
    },
  });

export const useCreateApiKey = () => {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: async ({
      key,
      provider,
    }: components["schemas"]["ApiKeyInput"]) => {
      const response = await client.POST("/api_key/", {
        body: { key, provider },
      });
      if (response.error) {
        throw new Error(response.error.message);
      }
      return response.data;
    },
    onSettled: () => queryClient.invalidateQueries({ queryKey: ["apiKeys"] }),
  });
};

export const useDeleteApiKey = () => {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: async (id: string) => {
      const response = await client.DELETE("/api_key/{api_key_id}", {
        params: { path: { api_key_id: id } },
      });
      if (response.error) {
        throw new Error(response.error.message);
      }
      return response.data;
    },
    onSettled: () => queryClient.invalidateQueries({ queryKey: ["apiKeys"] }),
  });
};
