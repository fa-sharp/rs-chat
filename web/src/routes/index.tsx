import { createFileRoute } from "@tanstack/react-router";

import Header from "@/components/Header";
import ChatMessageInput from "@/components/main/ChatMessageInput";
import ChatMessages from "@/components/main/ChatMessages";
import { AppSidebar } from "@/components/Sidebar";
import { SidebarInset, SidebarProvider } from "@/components/ui/sidebar";
import { useStreamingChat } from "@/lib/api/chat";
import { useGetChatSession } from "@/lib/api/session";

export const Route = createFileRoute("/")({
  component: App,
});

const sessionId = "f29dd136-c4de-41fb-89cf-c16c40025b05";

function App() {
  const { data } = useGetChatSession(sessionId);
  const { streamingMessage, onUserSubmit, isGenerating } =
    useStreamingChat(sessionId);

  return (
    <SidebarProvider>
      <AppSidebar />
      <SidebarInset className="overflow-hidden">
        <Header />
        <div className="flex-1 grid grid-rows-[minmax(0,1fr)_auto] gap-4 p-0 md:p-2 overflow-hidden">
          <ChatMessages
            messages={data?.messages || []}
            streamedResponse={streamingMessage}
            isGenerating={isGenerating && streamingMessage === ""}
          />
          <div className="w-full px-4 pb-4">
            <ChatMessageInput
              onSubmit={onUserSubmit}
              isGenerating={isGenerating}
            />
          </div>
        </div>
      </SidebarInset>
    </SidebarProvider>
  );
}
