import { useCallback, useContext } from "react";

import { ChatStreamContext } from "./streamManager";

/** Hook to stream tool executions */
export const useStreamingTools = () => {
  const { streamedTools, startToolExecution, cancelToolExecution } =
    useContext(ChatStreamContext);

  /** Execute a tool with streaming */
  const onToolExecute = useCallback(
    (messageId: string, sessionId: string, toolCallId: string) => {
      startToolExecution(messageId, sessionId, toolCallId);
    },
    [startToolExecution],
  );

  /** Cancel a tool execution */
  const onToolCancel = useCallback(
    (sessionId: string, toolCallId: string) => {
      cancelToolExecution(sessionId, toolCallId);
    },
    [cancelToolExecution],
  );

  return {
    streamedTools,
    onToolExecute,
    onToolCancel,
  };
};
