import { createFileRoute } from "@tanstack/react-router";
import { useCallback, useEffect, useState } from "react";

import Header from "@/components/Header";
import ChatMessageInput from "@/components/main/ChatMessageInput";
import ChatMessages from "@/components/main/ChatMessages";
import { AppSidebar } from "@/components/Sidebar";
import { SidebarInset, SidebarProvider } from "@/components/ui/sidebar";
import { useStreamedChatResponse } from "@/lib/api/chat";
import { useChatSession } from "@/lib/api/session";
import { queryClient } from "@/lib/api/client";
import type { components } from "@/lib/api/types";

export const Route = createFileRoute("/")({
  component: App,
});

const sessionId = "8314d62e-f9dd-43bb-b082-333029c9e0ab";
function App() {
  const chatSession = useChatSession(sessionId);

  const [streamInput, setStreamInput] = useState("");
  const streamedChatResponse = useStreamedChatResponse(streamInput, sessionId);
  useEffect(() => {
    if (streamedChatResponse.status === "complete") {
      const maxRetries = 3;
      const retryDelay = 1000; // 1 second

      const updateChatSession = async (retryCount = 0) => {
        try {
          await queryClient.cancelQueries({
            queryKey: ["chatSession", { sessionId }],
          });
          await queryClient.invalidateQueries({
            queryKey: ["chatSession", { sessionId }],
          });
          // Check if the chat session has been updated
          const updatedData = queryClient.getQueryData<{
            messages: components["schemas"]["ChatRsMessage"][];
          }>(["chatSession", { sessionId }]);
          const hasNewAssistantMessage = updatedData?.messages?.some(
            (msg) =>
              msg.role === "Assistant" &&
              new Date(msg.created_at).getTime() > Date.now() - 10000, // Within last 10 seconds
          );
          if (!hasNewAssistantMessage && retryCount < maxRetries) {
            setTimeout(() => updateChatSession(retryCount + 1), retryDelay);
            return;
          }
          setStreamInput("");
        } catch (error) {
          if (retryCount < maxRetries) {
            setTimeout(() => updateChatSession(retryCount + 1), retryDelay);
          } else {
            setStreamInput("");
          }
        }
      };

      updateChatSession();
    }
  }, [streamedChatResponse]);

  const onSubmit = useCallback((message: string) => {
    setStreamInput(message);
    queryClient.setQueryData<{
      messages: components["schemas"]["ChatRsMessage"][];
    }>(["chatSession", { sessionId }], (oldData: any) => {
      if (!oldData) return {};
      return {
        ...oldData,
        messages: [
          ...oldData.messages,
          {
            id: crypto.randomUUID(),
            content: message,
            role: "User",
            timestamp: new Date(),
          },
        ],
      };
    });
  }, []);

  return (
    <SidebarProvider>
      <AppSidebar />
      <SidebarInset className="overflow-hidden">
        <Header />
        <div className="flex-1 grid grid-rows-[minmax(0,1fr)_auto] gap-4 p-0 md:p-2 overflow-hidden">
          <ChatMessages
            messages={chatSession.data?.messages || []}
            streamedResponse={streamedChatResponse.message}
            isGenerating={
              (streamedChatResponse.status === "pending" ||
                streamedChatResponse.status === "streaming") &&
              streamedChatResponse.message === ""
            }
          />
          <div className="w-full px-4 pb-4">
            <ChatMessageInput
              onSubmit={onSubmit}
              isGenerating={
                streamedChatResponse.status === "pending" ||
                streamedChatResponse.status === "streaming"
              }
            />
          </div>
        </div>
      </SidebarInset>
    </SidebarProvider>
  );
}
