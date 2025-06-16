import ChatMessageInput from "@/components/main/ChatMessageInput";
import ChatMessages from "@/components/main/ChatMessages";
import { Skeleton } from "@/components/ui/skeleton";
import {
  chatSessionQueryKey,
  getChatSession,
  useGetChatSession,
} from "@/lib/api/session";
import { useStreamingChats } from "@/lib/context/StreamingContext";
import { createFileRoute } from "@tanstack/react-router";

export const Route = createFileRoute("/app/_appLayout/session/$sessionId")({
  component: RouteComponent,
  pendingComponent: () => (
    <div className="flex-1">
      <div className="flex flex-col space-y-10 p-48">
        <Skeleton className="h-10 w-full" />
        <Skeleton className="h-10 w-full" />
        <Skeleton className="h-10 w-full" />
        <Skeleton className="h-10 w-full" />
      </div>
    </div>
  ),
  loader: async ({ params, context }) => {
    await context.queryClient.ensureQueryData({
      queryKey: chatSessionQueryKey(params.sessionId),
      queryFn: () => getChatSession(params.sessionId),
    });
  },
});

function RouteComponent() {
  const { user } = Route.useRouteContext();
  const { sessionId } = Route.useParams();
  const { data } = useGetChatSession(sessionId);
  const { streamedChats, onUserSubmit } = useStreamingChats();

  return (
    <div className="flex-1 grid grid-rows-[minmax(0,1fr)_auto] gap-4 p-0 md:p-2 md:pt-0 overflow-hidden">
      <ChatMessages
        user={user}
        messages={data?.messages || []}
        streamedResponse={streamedChats[sessionId]?.content}
        error={streamedChats[sessionId]?.error}
        isGenerating={
          streamedChats[sessionId]?.status === "streaming" &&
          streamedChats[sessionId]?.content === ""
        }
      />
      <div className="w-full px-4 pb-4">
        <ChatMessageInput
          onSubmit={(input) => onUserSubmit(sessionId, input)}
          isGenerating={streamedChats[sessionId]?.status === "streaming"}
        />
      </div>
    </div>
  );
}
