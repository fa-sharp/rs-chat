import { ChevronDown, ChevronUp, PlayCircle } from "lucide-react";
import { Suspense, useState } from "react";

import type { components } from "@/lib/api/types";
import { cn } from "@/lib/utils";
import { Button } from "../ui/button";
import ChatFancyMarkdown from "./ChatFancyMarkdown";

export default function ChatMessageToolCalls({
  messages,
  toolCalls,
  onExecuteAll,
  isExecuting,
}: {
  messages: components["schemas"]["ChatRsMessage"][];
  isExecuting: boolean;
  onExecuteAll: () => void;
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
        {!expanded
          ? toolCalls.map((toolCall) => (
              <div key={toolCall.id} className="flex gap-2">
                <div className="flex items-center gap-2">
                  <div className="font-semibold">{toolCall.tool_name}</div>
                  {!messages.some(
                    (m) => m.meta.executed_tool_call?.id === toolCall.id,
                  ) && (
                    <Button size="sm" disabled={isExecuting}>
                      <PlayCircle />
                      Execute
                    </Button>
                  )}
                </div>
                <pre className="text-nowrap">
                  {JSON.stringify(toolCall.parameters)}
                </pre>
              </div>
            ))
          : toolCalls.map((toolCall) => (
              <div key={toolCall.id}>
                <div className="flex items-center gap-2">
                  <div className="font-semibold">{toolCall.tool_name}</div>
                  {!messages.some(
                    (m) => m.meta.executed_tool_call?.id === toolCall.id,
                  ) && (
                    <Button size="sm" disabled={isExecuting}>
                      <PlayCircle />
                      Execute
                    </Button>
                  )}
                </div>
                <div className="text-muted-foreground">
                  Tool call ID: {toolCall.id}
                </div>
                <div className="text-muted-foreground">
                  Tool ID: {toolCall.tool_id}
                </div>
                <Suspense fallback={<div>Loading...</div>}>
                  <ChatFancyMarkdown>
                    {`\`\`\`json\n${JSON.stringify(toolCall.parameters, null, 2)}\n\`\`\``}
                  </ChatFancyMarkdown>
                </Suspense>
              </div>
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
