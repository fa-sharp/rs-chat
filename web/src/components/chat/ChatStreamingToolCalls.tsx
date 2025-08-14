import { Loader2, Wrench, X } from "lucide-react";
import { lazy, Suspense, useState } from "react";

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
import { cn, escapeBackticks } from "@/lib/utils";

const ChatFancyMarkdown = lazy(() => import("./messages/ChatFancyMarkdown"));

interface Props {
  streamedTools: Record<string, StreamedToolExecution | undefined>;
  toolCalls?: components["schemas"]["ChatRsToolCall"][];
  tools?: components["schemas"]["ChatRsToolPublic"][];
  onCancel: (toolCallId: string) => void;
}

/** Displays currently streaming tool executions */
export default function ChatStreamingToolCalls({
  streamedTools,
  toolCalls,
  tools,
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
            onCancel={() => onCancel(toolCall.id)}
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
  tools?: components["schemas"]["ChatRsToolPublic"][];
  onCancel: () => void;
}) {
  const [showLogs, setShowLogs] = useState(true);
  const [showDebug, setShowDebug] = useState(false);

  const tool = tools?.find((t) => t.id === toolCall.tool_id);
  const isStreaming = streamedTool.status === "streaming";
  const hasError = streamedTool.status === "error" || !!streamedTool.error;
  const hasLogs = streamedTool.logs.length > 0;
  const hasDebugLogs = streamedTool.debugLogs.length > 0;

  // Use smooth streaming for the result output
  const { displayedText: displayedResult } = useSmoothStreaming(
    streamedTool.result,
    {
      baseCharsPerSecond: 80, // Faster than chat since tool output can be more verbose
      bufferSpeedUpThreshold: 50,
    },
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
        className={cn("space-y-3", hasError && "border-destructive/20 ")}
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
            <Button
              size="sm"
              variant="outline"
              onClick={onCancel}
              className="h-6 px-2 text-xs"
            >
              <X className="size-3" />
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
            <div className="rounded-md bg-muted/50 p-3">
              <Suspense fallback={<div className="text-sm">Loading...</div>}>
                <ChatFancyMarkdown>{`\`\`\`text\n${escapeBackticks(displayedResult)}\n\`\`\``}</ChatFancyMarkdown>
              </Suspense>
            </div>
          </div>
        )}

        {/* Logs section */}
        {hasLogs && (
          <Collapsible open={showLogs} onOpenChange={setShowLogs}>
            <CollapsibleTrigger asChild>
              <Button
                variant="outline"
                size="sm"
                className="w-full justify-between"
              >
                <span>Logs ({streamedTool.logs.length})</span>
                <span className="text-xs">{showLogs ? "Hide" : "Show"}</span>
              </Button>
            </CollapsibleTrigger>
            <CollapsibleContent className="space-y-1 pt-2">
              {streamedTool.logs.map((log, index) => (
                <div
                  key={`log-${index}-${log.slice(0, 20)}`}
                  className="rounded bg-muted/30 px-2 py-1 text-xs font-mono"
                >
                  {log}
                </div>
              ))}
            </CollapsibleContent>
          </Collapsible>
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
                <span>Debug ({streamedTool.debugLogs.length})</span>
                <span className="text-xs">{showDebug ? "Hide" : "Show"}</span>
              </Button>
            </CollapsibleTrigger>
            <CollapsibleContent className="space-y-1 pt-2">
              {streamedTool.debugLogs.map((debug, index) => (
                <div
                  key={`debug-${index}-${debug.slice(0, 20)}`}
                  className="rounded bg-muted/20 px-2 py-1 text-xs font-mono text-muted-foreground"
                >
                  {debug}
                </div>
              ))}
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
