import { memo, useCallback } from "react";

import { useDeleteChatMessage } from "@/lib/api/session";
import type { components } from "@/lib/api/types";
import ChatMessage from "./messages/ChatMessage";

interface Props {
  user?: components["schemas"]["ChatRsUser"];
  messages: Array<components["schemas"]["ChatRsMessage"]>;
  providers?: Array<components["schemas"]["ChatRsProvider"]>;
  tools?: components["schemas"]["GetAllToolsResponse"];
  onToolExecute: (
    messageId: string,
    sessionId: string,
    toolCallId: string,
  ) => void;
  isStreaming?: boolean;
  sessionId: string;
}

/** Displays all (non-streaming) messages in the session */
export default memo(function ChatMessages({
  user,
  messages,
  providers,
  tools,
  onToolExecute,
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

  const onExecuteToolCall = useCallback(
    (messageId: string, toolCallId: string) => {
      onToolExecute(messageId, sessionId, toolCallId);
    },
    [onToolExecute, sessionId],
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
        onDeleteMessage={onDeleteMessage}
      />
    ));
});
