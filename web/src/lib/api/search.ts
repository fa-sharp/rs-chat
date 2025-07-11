import { useQuery } from "@tanstack/react-query";

import { client } from "./client";

export const useSearchChats = (searchQuery?: string) =>
  useQuery({
    staleTime: 10 * 1000, // 10 seconds
    queryKey: ["search", { query: searchQuery }] as const,
    queryFn: async ({ queryKey: [_, { query }] }) => {
      if (!query) return [];
      const res = await client.GET("/session/search", {
        params: { query: { query } },
      });
      if (res.error) throw new Error(res.error.message);
      return res.data;
    },
    placeholderData: (prev) => prev,
  });
