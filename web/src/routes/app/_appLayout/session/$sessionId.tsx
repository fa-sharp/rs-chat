import { createFileRoute } from "@tanstack/react-router";
import { useCallback, useMemo } from "react";

import ChatMessageInput from "@/components/chat/ChatMessageInput";
import ChatMessages from "@/components/chat/ChatMessages";
import { ChatStreamingMessages } from "@/components/chat/messages";
import ErrorComponent from "@/components/Error";
import { ChatMessageList } from "@/components/ui/chat/chat-message-list";
import { Skeleton } from "@/components/ui/skeleton";
import { useChatInputState } from "@/hooks/useChatInputState";
import { useProviders } from "@/lib/api/provider";
import {
  chatSessionQueryKey,
  getChatSession,
  useGetChatSession,
} from "@/lib/api/session";
import { useTools } from "@/lib/api/tool";
import type { components } from "@/lib/api/types";
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

  const onSubmit = useCallback(
    (input: components["schemas"]["SendChatInput"]) => {
      onUserSubmit(sessionId, input);
    },
    [onUserSubmit, sessionId],
  );

  const currentStream = useMemo(
    () => streamedChats[sessionId],
    [streamedChats, sessionId],
  );

  const lastAssistantMessage = useMemo(
    () =>
      session?.messages.findLast(
        (m) =>
          m.role === "Assistant" &&
          !!m.meta.assistant?.provider_id &&
          !!m.meta.assistant?.provider_options,
      ),
    [session?.messages],
  );

  const inputState = useChatInputState({
    providers,
    initialProviderId: lastAssistantMessage?.meta.assistant?.provider_id,
    initialOptions: lastAssistantMessage?.meta.assistant?.provider_options,
    initialToolIds: session?.session.meta.tools,
    isGenerating: currentStream?.status === "streaming",
    onSubmit,
    sessionId,
  });

  return (
    <div className="flex-1 grid grid-rows-[minmax(0,1fr)_auto] gap-4 px-2 pb-2 md:px-4 md:pb-4 overflow-hidden">
      <ChatMessageList>
        <ChatMessages
          user={user}
          messages={session?.messages || []}
          providers={providers}
          tools={tools}
          sessionId={sessionId}
          onGetAgenticResponse={inputState.onSubmitWithoutUserMessage}
          isStreaming={currentStream?.status === "streaming"}
        />
        <ChatStreamingMessages
          sessionId={sessionId}
          currentStream={currentStream}
        />
      </ChatMessageList>
      <ChatMessageInput inputState={inputState} />
    </div>
  );
}
