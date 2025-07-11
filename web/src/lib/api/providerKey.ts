import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { client } from "./client";
import type { components } from "./types";

const queryKey = ["providerKeys"];

export const useProviderKeys = () =>
  useQuery({
    queryKey,
    queryFn: async () => {
      const response = await client.GET("/provider_key/");
      if (response.error) {
        throw new Error(response.error.message);
      }
      return response.data;
    },
  });

export const useCreateProviderKey = () => {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: async ({
      key,
      provider,
    }: components["schemas"]["ProviderKeyInput"]) => {
      const response = await client.POST("/provider_key/", {
        body: { key, provider },
      });
      if (response.error) {
        throw new Error(response.error.message);
      }
      return response.data;
    },
    onSettled: () => queryClient.invalidateQueries({ queryKey }),
  });
};

export const useDeleteProviderKey = () => {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: async (id: string) => {
      const response = await client.DELETE("/provider_key/{api_key_id}", {
        params: { path: { api_key_id: id } },
      });
      if (response.error) {
        throw new Error(response.error.message);
      }
      return response.data;
    },
    onSettled: () => queryClient.invalidateQueries({ queryKey }),
  });
};
