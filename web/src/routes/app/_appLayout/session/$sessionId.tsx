import ChatMessageInput from "@/components/main/ChatMessageInput";
import ChatMessages from "@/components/main/ChatMessages";
import { useStreamingChat } from "@/lib/api/chat";
import {
  chatSessionQueryKey,
  getChatSession,
  useGetChatSession,
} from "@/lib/api/session";
import { createFileRoute } from "@tanstack/react-router";

export const Route = createFileRoute("/app/_appLayout/session/$sessionId")({
  component: RouteComponent,
  loader: async ({ params, context }) => {
    await context.queryClient.ensureQueryData({
      queryKey: chatSessionQueryKey(params.sessionId),
      queryFn: () => getChatSession(params.sessionId),
    });
  },
});

function RouteComponent() {
  const { sessionId } = Route.useParams();
  const { data } = useGetChatSession(sessionId);
  const { streamingMessage, onUserSubmit, isGenerating } =
    useStreamingChat(sessionId);

  return (
    <div className="flex-1 grid grid-rows-[minmax(0,1fr)_auto] gap-4 p-0 md:p-2 overflow-hidden">
      <ChatMessages
        messages={data?.messages || []}
        streamedResponse={streamingMessage}
        isGenerating={isGenerating && streamingMessage === ""}
      />
      <div className="w-full px-4 pb-4">
        <ChatMessageInput onSubmit={onUserSubmit} isGenerating={isGenerating} />
      </div>
    </div>
  );
}
