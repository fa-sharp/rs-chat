import { QueryClient } from "@tanstack/react-query";
import createClient from "openapi-fetch";
import type { paths } from "./types";

export const client = createClient<paths>({
  baseUrl: import.meta.env.VITE_API_URL || "/api",
});

export const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      staleTime: 30 * 1000, // 30 seconds to stale data
    },
  },
});
