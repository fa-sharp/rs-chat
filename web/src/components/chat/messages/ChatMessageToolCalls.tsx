import { ChevronDown, ChevronUp, Loader2, PlayCircle } from "lucide-react";
import { lazy, Suspense, useState } from "react";

import { getToolIcon, getToolTypeLabel } from "@/components/ToolsManager";
import { Button } from "@/components/ui/button";
import type { components } from "@/lib/api/types";
import { useStreamingTools } from "@/lib/context/StreamingContext";
import { cn } from "@/lib/utils";

const ChatFancyMarkdown = lazy(() => import("./ChatFancyMarkdown"));

interface Props {
  toolCalls?: components["schemas"]["ChatRsToolCall"][];
  executedToolCalls?: components["schemas"]["ChatRsToolCall"][];
  tools?: components["schemas"]["ChatRsToolPublic"][];
  onExecute: (toolCallId: string) => void;
}

export default function ChatMessageToolCalls({
  toolCalls,
  executedToolCalls,
  tools,
  onExecute,
}: Props) {
  const { streamedTools } = useStreamingTools();

  const [expanded, setExpanded] = useState(false);

  if (!toolCalls || toolCalls.length === 0) {
    return null;
  }

  return (
    <div className="prose-h3:mt-0 prose-h3:mb-0.5 prose-h3:text-lg prose-pre:my-1">
      <h3 className="flex gap-2">
        Tool Calls ({toolCalls.length})
        <Button
          size="sm"
          variant="outline"
          onClick={() => setExpanded(!expanded)}
        >
          {expanded ? <ChevronUp /> : <ChevronDown />}
          {expanded ? "Collapse" : "Expand"}
        </Button>
      </h3>
      <div className={cn("flex flex-col", expanded && "gap-2")}>
        {toolCalls.map((toolCall) => (
          <ChatMessageToolCall
            key={toolCall.id}
            toolCall={toolCall}
            tools={tools}
            expanded={expanded}
            onExecute={() => onExecute(toolCall.id)}
            canExecute={!executedToolCalls?.some((tc) => tc.id === toolCall.id)}
            isExecuting={streamedTools[toolCall.id]?.status === "streaming"}
          />
        ))}
      </div>
    </div>
  );
}

function ChatMessageToolCall({
  toolCall,
  tools,
  expanded,
  onExecute,
  isExecuting,
  canExecute,
}: {
  toolCall: components["schemas"]["ChatRsToolCall"];
  tools?: components["schemas"]["ChatRsToolPublic"][];
  onExecute: () => void;
  isExecuting: boolean;
  canExecute: boolean;
  expanded: boolean;
}) {
  const tool = tools?.find((tool) => tool.id === toolCall.tool_id);

  return !expanded ? (
    <div className="flex gap-2">
      <div className="flex items-center gap-2">
        <div className="flex items-center gap-1.5 font-semibold">
          {isExecuting ? (
            <Loader2 className="size-4 animate-spin" />
          ) : (
            tool && getToolIcon(tool)
          )}
          {toolCall.tool_name}
        </div>
        {canExecute && (
          <Button
            size="sm"
            loading={isExecuting}
            disabled={isExecuting}
            onClick={onExecute}
          >
            {!isExecuting && <PlayCircle />}
            Execute
          </Button>
        )}
      </div>
      <pre className="text-nowrap">{JSON.stringify(toolCall.parameters)}</pre>
    </div>
  ) : (
    <div key={toolCall.id} className="flex flex-col gap-1 py-2">
      <div className="flex items-center gap-2">
        <div className="flex items-center gap-2 font-semibold">
          {isExecuting ? (
            <Loader2 className="size-4 animate-spin" />
          ) : (
            tool && getToolIcon(tool)
          )}
          {tool && `${getToolTypeLabel(tool)}: `}
          {toolCall.tool_name}
          {isExecuting && (
            <span className="text-xs font-normal text-muted-foreground">
              (executing...)
            </span>
          )}
        </div>
        {canExecute && (
          <Button size="sm" disabled={isExecuting} onClick={onExecute}>
            <PlayCircle />
            Execute
          </Button>
        )}
      </div>
      <div className="font-semibold">Input</div>
      <Suspense fallback={<div>Loading...</div>}>
        <ChatFancyMarkdown>
          {`\`\`\`json\n${JSON.stringify(toolCall.parameters, null, 2)}\n\`\`\``}
        </ChatFancyMarkdown>
      </Suspense>
      {tool?.config.type === "CodeExecutor" &&
        typeof toolCall.parameters.code === "string" &&
        typeof toolCall.parameters.language === "string" && (
          <>
            <div className="font-semibold">Code</div>
            <Suspense fallback={<div>Loading...</div>}>
              <ChatFancyMarkdown>
                {`\`\`\`${toolCall.parameters.language}\n${toolCall.parameters.code}\n\`\`\``}
              </ChatFancyMarkdown>
            </Suspense>
          </>
        )}
      <div className="text-sm text-muted-foreground">
        Tool call ID: {toolCall.id}
        <br />
        Tool ID: {toolCall.tool_id}
      </div>
    </div>
  );
}
