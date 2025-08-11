import { memo, useCallback } from "react";

import { useDeleteChatMessage } from "@/lib/api/session";
import { useExecuteAllTools, useExecuteTool } from "@/lib/api/tool";
import type { components } from "@/lib/api/types";
import ChatMessage from "./messages/ChatMessage";

interface Props {
  user?: components["schemas"]["ChatRsUser"];
  messages: Array<components["schemas"]["ChatRsMessage"]>;
  providers?: Array<components["schemas"]["ChatRsProvider"]>;
  tools?: Array<components["schemas"]["ChatRsToolPublic"]>;
  onGetAgenticResponse: () => void;
  isStreaming?: boolean;
  sessionId: string;
}

/** Displays all (non-streaming) messages in the session */
export default memo(function ChatMessages({
  user,
  messages,
  providers,
  tools,
  onGetAgenticResponse,
  isStreaming,
  sessionId,
}: Props) {
  const { mutate: deleteMessage } = useDeleteChatMessage();
  const onDeleteMessage = useCallback(
    (messageId: string) => {
      sessionId && deleteMessage({ sessionId, messageId });
    },
    [deleteMessage, sessionId],
  );

  const executeToolCall = useExecuteTool();
  const executeAllToolCalls = useExecuteAllTools();

  const onExecuteToolCall = useCallback(
    (messageId: string, toolCallId: string) => {
      executeToolCall.mutate(
        { messageId, toolCallId },
        {
          onSuccess: () => {
            const allToolCallsCompleted = messages
              .find((m) => m.id === messageId)
              ?.meta.assistant?.tool_calls?.every(
                (tc) =>
                  tc.id === toolCallId ||
                  messages.find(
                    (m) => m.role === "Tool" && m.meta.tool_call?.id === tc.id,
                  ),
              );
            if (allToolCallsCompleted) {
              onGetAgenticResponse();
            }
          },
        },
      );
    },
    [executeToolCall, onGetAgenticResponse, messages],
  );

  const onExecuteAllToolCalls = useCallback(
    (messageId: string) => {
      executeAllToolCalls.mutate(
        { messageId },
        {
          onSuccess: () => onGetAgenticResponse(),
        },
      );
    },
    [executeAllToolCalls, onGetAgenticResponse],
  );

  return messages
    .filter(
      (message, idx) =>
        !isStreaming ||
        !(message.meta.assistant?.partial && idx === messages.length - 1), // don't show the partial assistant response if still streaming
    )
    .map((message) => (
      <ChatMessage
        key={message.id}
        message={message}
        user={user}
        providers={providers}
        tools={tools}
        executedToolCalls={message.meta.assistant?.tool_calls?.filter((tc) =>
          messages.some((m) => m.meta.tool_call?.id === tc.id),
        )}
        onExecuteToolCall={onExecuteToolCall}
        onExecuteAllToolCalls={onExecuteAllToolCalls}
        isExecutingTool={
          executeToolCall.isPending || executeAllToolCalls.isPending
        }
        onDeleteMessage={onDeleteMessage}
      />
    ));
});
