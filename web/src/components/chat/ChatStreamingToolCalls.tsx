import { ChevronDown, ChevronUp, Loader2, Wrench, X } from "lucide-react";
import { useMemo, useState } from "react";
import Markdown from "react-markdown";

import { getToolIcon, getToolTypeLabel } from "@/components/ToolsManager";
import { Button } from "@/components/ui/button";
import {
  ChatBubble,
  ChatBubbleAvatar,
  ChatBubbleMessage,
} from "@/components/ui/chat/chat-bubble";
import {
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
} from "@/components/ui/collapsible";
import useSmoothStreaming from "@/hooks/useSmoothStreaming";
import type { components } from "@/lib/api/types";
import type { StreamedToolExecution } from "@/lib/context/StreamingContext";
import { getToolFromToolCall } from "@/lib/tools";
import { cn, escapeBackticks } from "@/lib/utils";
import { useAutoScroll } from "../ui/chat/hooks/useAutoScroll";
import ChatMessageToolLogs from "./messages/ChatMessageToolLogs";

interface Props {
  streamedTools: Record<string, StreamedToolExecution | undefined>;
  toolCalls?: components["schemas"]["ChatRsToolCall"][];
  tools?: components["schemas"]["GetAllToolsResponse"];
  sessionId: string;
  onCancel: (sessionId: string, toolCallId: string) => void;
}

/** Displays currently streaming tool executions */
export default function ChatStreamingToolCalls({
  streamedTools,
  toolCalls,
  tools,
  sessionId,
  onCancel,
}: Props) {
  const streamingToolCalls = toolCalls?.filter(
    (toolCall) => streamedTools[toolCall.id],
  );

  if (!streamingToolCalls || streamingToolCalls.length === 0) {
    return null;
  }

  return (
    <div className="space-y-2">
      {streamingToolCalls.map((toolCall) => {
        const streamedTool = streamedTools[toolCall.id];
        if (!streamedTool) return null;

        return (
          <StreamingToolCall
            key={toolCall.id}
            toolCall={toolCall}
            streamedTool={streamedTool}
            tools={tools}
            onCancel={() => onCancel(sessionId, toolCall.id)}
          />
        );
      })}
    </div>
  );
}

function StreamingToolCall({
  toolCall,
  streamedTool,
  tools,
  onCancel,
}: {
  toolCall: components["schemas"]["ChatRsToolCall"];
  streamedTool: StreamedToolExecution;
  tools?: components["schemas"]["GetAllToolsResponse"];
  onCancel: () => void;
}) {
  const [showDebug, setShowDebug] = useState(false);

  const tool = useMemo(
    () => getToolFromToolCall(toolCall, tools),
    [tools, toolCall],
  );
  const isStreaming = streamedTool.status === "streaming";
  const hasError = streamedTool.status === "error" || !!streamedTool.error;
  const hasLogs = streamedTool.logs.length > 0;
  const hasDebugLogs = streamedTool.debugLogs.length > 0;

  // Use smooth streaming for the result output
  const { displayedText: displayedResult } = useSmoothStreaming(
    streamedTool.result,
    { baseCharsPerSecond: 100 },
  );

  // Get appropriate icon based on status
  const getStatusIcon = () => {
    if (hasError) {
      return <X className="size-4 text-destructive" />;
    }
    if (isStreaming) {
      return <Loader2 className="size-4 animate-spin" />;
    }
    return tool ? getToolIcon(tool) : <Wrench className="size-4" />;
  };

  return (
    <ChatBubble variant="received" layout="ai">
      <ChatBubbleAvatar
        fallback={getStatusIcon()}
        className={cn(
          isStreaming && "animate-pulse",
          hasError && "border-destructive",
        )}
      />
      <ChatBubbleMessage
        className={cn("space-y-2", hasError && "border-destructive/20 ")}
      >
        {/* Tool header */}
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-2 font-semibold">
            {tool && getToolIcon(tool)}
            <span>
              {tool ? getToolTypeLabel(tool) : "Tool"}: {toolCall.tool_name}
            </span>
            {isStreaming && (
              <span className="text-xs font-normal text-muted-foreground">
                Running...
              </span>
            )}
          </div>
          {isStreaming && (
            <Button size="sm" variant="destructive" onClick={onCancel}>
              <X />
              Cancel
            </Button>
          )}
        </div>

        {/* Error message */}
        {hasError && streamedTool.error && (
          <div className="rounded-md bg-destructive/10 p-3 text-destructive-foreground">
            <div className="font-semibold">Error</div>
            <div className="text-sm">{streamedTool.error}</div>
          </div>
        )}

        {/* Tool result/output */}
        {displayedResult && (
          <div className="space-y-2">
            <div className="text-sm font-semibold">Output</div>
            <div className="rounded-md bg-muted/50 p-3 text-sm">
              <Markdown>{`\`\`\`text\n${escapeBackticks(displayedResult)}\n\`\`\``}</Markdown>
            </div>
          </div>
        )}

        {/* Logs section */}
        {hasLogs && (
          <ChatMessageToolLogs logs={streamedTool.logs} initialOpen />
        )}

        {/* Debug logs section */}
        {hasDebugLogs && (
          <Collapsible open={showDebug} onOpenChange={setShowDebug}>
            <CollapsibleTrigger asChild>
              <Button
                variant="outline"
                size="sm"
                className="w-full justify-between"
              >
                <span className="flex items-center gap-1">
                  {showDebug ? <ChevronUp /> : <ChevronDown />}Debug (
                  {streamedTool.debugLogs.length})
                </span>
              </Button>
            </CollapsibleTrigger>
            <CollapsibleContent>
              <DebugLogsContent>
                {streamedTool.debugLogs.map((debug, index) => (
                  <div
                    key={`debug-${index}-${debug.slice(0, 20)}`}
                    className="rounded bg-muted/20 px-2 py-1 text-xs font-mono text-muted-foreground"
                  >
                    {debug}
                  </div>
                ))}
              </DebugLogsContent>
            </CollapsibleContent>
          </Collapsible>
        )}

        {/* Tool call metadata */}
        <div className="text-xs text-muted-foreground">
          Tool call ID: {toolCall.id}
        </div>
      </ChatBubbleMessage>
    </ChatBubble>
  );
}

function DebugLogsContent({ children }: { children: React.ReactNode }) {
  const { scrollRef } = useAutoScroll({ content: children });

  return (
    <div ref={scrollRef} className="space-y-0.5 pt-1 max-h-32 overflow-auto">
      {children}
    </div>
  );
}
