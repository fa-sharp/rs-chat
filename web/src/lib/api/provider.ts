import { useQuery } from "@tanstack/react-query";

import { client } from "./client";
import type { components } from "./types";

export const useGetProviderInfo = (
  type?: components["schemas"]["ChatRsApiKeyProviderType"],
) =>
  useQuery({
    enabled: !!type,
    staleTime: Infinity,
    queryKey: ["providerInfo", { type }] as const,
    queryFn: async ({ queryKey }) => {
      if (!queryKey[1].type) return;
      const res = await client.GET("/provider/", {
        params: { query: { provider_type: queryKey[1].type } },
      });
      if (res.error) throw new Error(res.error.message);
      return res.data;
    },
  });
