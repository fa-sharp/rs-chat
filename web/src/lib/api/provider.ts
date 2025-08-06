import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { client } from "./client";
import type { components } from "./types";

const queryKey = ["providerKeys"];

export const useProviders = () =>
  useQuery({
    queryKey,
    queryFn: async () => {
      const response = await client.GET("/provider/");
      if (response.error) {
        throw new Error(response.error.message);
      }
      return response.data;
    },
  });

export const useProviderModels = (providerId?: number | null) =>
  useQuery({
    enabled: !!providerId,
    queryKey: ["providerModels", { providerId }],
    queryFn: async () => {
      if (!providerId) return [];
      const response = await client.GET("/provider/{provider_id}/models", {
        params: { path: { provider_id: providerId } },
      });
      if (response.error) {
        throw new Error(response.error.message);
      }
      return response.data;
    },
  });

export const useCreateProvider = () => {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: async (body: components["schemas"]["ProviderCreateInput"]) => {
      const response = await client.POST("/provider/", {
        body,
      });
      if (response.error) {
        throw new Error(response.error.message);
      }
      return response.data;
    },
    onSettled: () => queryClient.invalidateQueries({ queryKey }),
  });
};

export const useDeleteProvider = () => {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: async (id: number) => {
      const response = await client.DELETE("/provider/{provider_id}", {
        params: { path: { provider_id: id } },
      });
      if (response.error) {
        throw new Error(response.error.message);
      }
      return response.data;
    },
    onSettled: () => queryClient.invalidateQueries({ queryKey }),
  });
};
