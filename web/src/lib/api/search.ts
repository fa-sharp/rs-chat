import { useQuery } from "@tanstack/react-query";

export const useSearchChats = (searchQuery?: string) =>
  useQuery({
    staleTime: 10 * 1000, // 10 seconds
    enabled: !!searchQuery,
    queryKey: ["search", { query: searchQuery }] as const,
    queryFn: async ({ queryKey: [_, { query }] }) => {
      console.log(`Searching chats for '${query}'`);
      await new Promise((r) => setTimeout(r, 600));
      return [
        `${searchQuery} 1`,
        `${searchQuery} 2`,
        `${searchQuery} other`,
        `${searchQuery} 4`,
      ];
    },
    placeholderData: (prev) => prev,
  });
