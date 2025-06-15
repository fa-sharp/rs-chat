import { useQuery } from "@tanstack/react-query";
import { client } from "./client";

export async function getChatSession(sessionId: string) {
  const response = await client.GET("/session/{session_id}", {
    params: { path: { session_id: sessionId } },
  });
  if (response.error) {
    throw new Error(response.error.message);
  }
  return response.data;
}

export const chatSessionQueryKey = (sessionId: string) => [
  "chatSession",
  { sessionId },
];

export const useGetChatSession = (sessionId: string) =>
  useQuery({
    queryKey: chatSessionQueryKey(sessionId),
    queryFn: () => getChatSession(sessionId),
  });

export async function getRecentChatSessions() {
  const response = await client.GET("/session/");
  if (response.error) {
    throw new Error(response.error.message);
  }
  return response.data;
}

export const useGetRecentChatSessions = () =>
  useQuery({
    queryKey: ["recentChatSessions"],
    queryFn: () => getRecentChatSessions(),
  });
