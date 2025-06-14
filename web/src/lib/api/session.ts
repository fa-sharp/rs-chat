import { useQuery } from "@tanstack/react-query";
import { client } from "./client";

export const useChatSession = (sessionId: string) =>
  useQuery({
    queryKey: ["chatSession", { sessionId }],
    queryFn: async () => {
      const response = await client.GET("/session/{session_id}", {
        params: { path: { session_id: sessionId } },
      });
      if (response.error) {
        throw new Error(response.error.message);
      }
      return response.data;
    },
  });
