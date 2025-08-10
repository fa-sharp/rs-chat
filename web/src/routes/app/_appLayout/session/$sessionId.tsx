import { createFileRoute } from "@tanstack/react-router";

import ChatMessageInput from "@/components/chat/ChatMessageInput";
import ChatMessages from "@/components/chat/ChatMessages";
import ErrorComponent from "@/components/Error";
import { Skeleton } from "@/components/ui/skeleton";
import { useChatInputState } from "@/hooks/useChatInputState";
import { useProviders } from "@/lib/api/provider";
import {
  chatSessionQueryKey,
  getChatSession,
  useGetChatSession,
} from "@/lib/api/session";
import { useTools } from "@/lib/api/tool";
import { useStreamingChats } from "@/lib/context/StreamingContext";

export const Route = createFileRoute("/app/_appLayout/session/$sessionId")({
  component: RouteComponent,
  errorComponent: ErrorComponent,
  pendingComponent: () => (
    <div className="flex-1">
      <div className="flex flex-col space-y-10 p-8 md:p-24">
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
  const { data: session } = useGetChatSession(sessionId);
  const { data: providers } = useProviders();
  const { data: tools } = useTools();
  const { streamedChats, onUserSubmit } = useStreamingChats();

  const inputState = useChatInputState({
    providers,
    initialProviderId: session?.messages.findLast(
      (m) => m.role === "Assistant" && !!m.meta.assistant?.provider_id,
    )?.meta.assistant?.provider_id,
    initialOptions: session?.messages.findLast(
      (m) => m.role === "Assistant" && !!m.meta.assistant?.provider_options,
    )?.meta.assistant?.provider_options,
    initialToolIds: session?.session.meta.tools,
    isGenerating: streamedChats[sessionId]?.status === "streaming",
    onSubmit: (input) => onUserSubmit(sessionId, input),
    sessionId,
  });

  return (
    <div className="flex-1 grid grid-rows-[minmax(0,1fr)_auto] gap-4 p-0 md:p-2 md:pt-0 overflow-hidden">
      <ChatMessages
        user={user}
        messages={session?.messages || []}
        providers={providers}
        tools={tools}
        sessionId={sessionId}
        onGetAgenticResponse={inputState.onSubmitWithoutUserMessage}
        error={streamedChats[sessionId]?.error}
        streamedResponse={streamedChats[sessionId]?.content}
        isWaitingForAssistant={
          streamedChats[sessionId]?.status === "streaming" &&
          streamedChats[sessionId]?.content === ""
        }
        isCompleted={streamedChats[sessionId]?.status === "completed"}
      />
      <div className="w-full px-4 pb-4">
        <ChatMessageInput inputState={inputState} />
      </div>
    </div>
  );
}
