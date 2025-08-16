import { createFileRoute } from "@tanstack/react-router";
import { useCallback, useMemo } from "react";

import {
  ChatMessageInput,
  ChatMessages,
  ChatStreamingMessages,
  ChatStreamingToolCalls,
} from "@/components/chat";
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
import {
  useStreamingChats,
  useStreamingTools,
} from "@/lib/context/StreamingContext";

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
  const { streamedTools, onToolExecute, onToolCancel } = useStreamingTools();

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

  /** Whether we just finished executing tool call(s) and are ready to get an agentic response */
  const canGetAgenticResponse = useMemo(() => {
    const lastMessage = session?.messages.at(-1);
    if (lastMessage?.role !== "Tool" || !lastMessage.meta.tool_call)
      return false;
    const toolCall = lastMessage.meta.tool_call;
    const toolCallMessage = session?.messages.findLast(
      (m) =>
        m.role === "Assistant" &&
        m.meta.assistant?.tool_calls?.some((tc) => tc.id === toolCall.id),
    );
    const allToolCallsExecuted =
      toolCallMessage?.meta.assistant?.tool_calls?.every((tc) =>
        session?.messages.some(
          (m) => m.role === "Tool" && m.meta.tool_call?.id === tc.id,
        ),
      );
    return !!toolCallMessage && !!allToolCallsExecuted;
  }, [session?.messages]);

  const inputState = useChatInputState({
    providers,
    initialProviderId: lastAssistantMessage?.meta.assistant?.provider_id,
    initialOptions: lastAssistantMessage?.meta.assistant?.provider_options,
    initialToolIds: session?.session.meta.tools,
    isGenerating: currentStream?.status === "streaming",
    canGetAgenticResponse,
    onSubmit,
    sessionId,
  });

  return (
    <div className="flex-1 grid grid-rows-[minmax(0,1fr)_auto] gap-4 px-2 pb-2 md:px-4 md:pb-4 overflow-hidden">
      <ChatMessageList className="pb-2">
        <ChatMessages
          user={user}
          messages={session?.messages || []}
          providers={providers}
          tools={tools}
          sessionId={sessionId}
          onToolExecute={onToolExecute}
          isStreaming={currentStream?.status === "streaming"}
        />
        <ChatStreamingMessages
          sessionId={sessionId}
          currentStream={currentStream}
        />
        <ChatStreamingToolCalls
          sessionId={sessionId}
          streamedTools={streamedTools}
          toolCalls={session?.messages
            .filter((m) => m.role === "Assistant")
            .flatMap((m) => m.meta.assistant?.tool_calls || [])}
          tools={tools}
          onCancel={onToolCancel}
        />
      </ChatMessageList>
      <ChatMessageInput inputState={inputState} />
    </div>
  );
}
