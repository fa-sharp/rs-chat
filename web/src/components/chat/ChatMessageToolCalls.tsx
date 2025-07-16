import { ChevronDown, ChevronUp, PlayCircle } from "lucide-react";
import { Suspense, useState } from "react";

import type { components } from "@/lib/api/types";
import { cn } from "@/lib/utils";
import { getToolIcon, getToolTypeLabel } from "../ToolsManager";
import { Button } from "../ui/button";
import ChatFancyMarkdown from "./ChatFancyMarkdown";

export default function ChatMessageToolCalls({
  messages,
  tools,
  toolCalls,
  onExecuteAll,
  isExecuting,
}: {
  messages: components["schemas"]["ChatRsMessage"][];
  isExecuting: boolean;
  onExecuteAll: () => void;
  tools?: components["schemas"]["ChatRsTool"][];
  toolCalls: components["schemas"]["ChatRsMessage"]["meta"]["tool_calls"];
}) {
  const [expanded, setExpanded] = useState(false);

  if (!toolCalls || toolCalls.length === 0) {
    return null;
  }

  return (
    <div className="prose-h3:mt-0 prose-h3:mb-1 prose-pre:my-1">
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
            messages={messages}
            tools={tools}
            expanded={expanded}
            isExecuting={isExecuting}
          />
        ))}
        {!toolCalls.some((toolCall) =>
          messages.some((m) => m.meta.executed_tool_call?.id === toolCall.id),
        ) && (
          <p>
            <Button onClick={onExecuteAll} disabled={isExecuting}>
              <PlayCircle />
              {isExecuting ? "Executing..." : "Execute All"}
            </Button>
          </p>
        )}
      </div>
    </div>
  );
}

function ChatMessageToolCall({
  toolCall,
  messages,
  tools,
  expanded,
  isExecuting,
}: {
  toolCall: NonNullable<
    components["schemas"]["ChatRsMessage"]["meta"]["tool_calls"]
  >[number];
  messages: components["schemas"]["ChatRsMessage"][];
  tools?: components["schemas"]["ChatRsTool"][];
  isExecuting: boolean;
  expanded: boolean;
}) {
  const tool = tools?.find((tool) => tool.id === toolCall.tool_id);

  return !expanded ? (
    <div className="flex gap-2">
      <div className="flex items-center gap-2">
        <div className="flex items-center gap-1.5 font-semibold">
          {tool && getToolIcon(tool)}
          {toolCall.tool_name}
        </div>
        {!messages.some(
          (m) => m.meta.executed_tool_call?.id === toolCall.id,
        ) && (
          <Button size="sm" disabled={isExecuting}>
            <PlayCircle />
            Execute
          </Button>
        )}
      </div>
      <pre className="text-nowrap">{JSON.stringify(toolCall.parameters)}</pre>
    </div>
  ) : (
    <div key={toolCall.id}>
      <div className="flex items-center gap-2">
        <div className="flex items-center gap-2 font-semibold">
          {tool && getToolIcon(tool)}
          {tool && `${getToolTypeLabel(tool)}: `}
          {toolCall.tool_name}
        </div>
        {!messages.some(
          (m) => m.meta.executed_tool_call?.id === toolCall.id,
        ) && (
          <Button size="sm" disabled={isExecuting}>
            <PlayCircle />
            Execute
          </Button>
        )}
      </div>
      <Suspense fallback={<div>Loading...</div>}>
        <ChatFancyMarkdown>
          {`\`\`\`json\n${JSON.stringify(toolCall.parameters, null, 2)}\n\`\`\``}
        </ChatFancyMarkdown>
      </Suspense>
      <div className="text-sm text-muted-foreground">
        Tool call ID: {toolCall.id}
      </div>
      <div className="text-sm text-muted-foreground">
        Tool ID: {toolCall.tool_id}
      </div>
    </div>
  );
}
