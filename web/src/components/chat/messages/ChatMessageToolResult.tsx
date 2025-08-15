import { ChevronDown, ChevronUp } from "lucide-react";
import { lazy, Suspense, useState } from "react";
import Markdown from "react-markdown";

import { getToolIcon, getToolTypeLabel } from "@/components/ToolsManager";
import { Button } from "@/components/ui/button";
import {
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
} from "@/components/ui/collapsible";
import type { components } from "@/lib/api/types";
import { cn, escapeBackticks } from "@/lib/utils";
import ChatMessageToolLogs from "./ChatMessageToolLogs";

const ChatFancyMarkdown = lazy(() => import("./ChatFancyMarkdown"));

export default function ChatMessageToolResult({
  message,
  tools,
}: {
  message: components["schemas"]["ChatRsMessage"];
  tools?: components["schemas"]["ChatRsToolPublic"][];
}) {
  const [showOutput, setShowOutput] = useState(false);

  const tool = tools?.find(
    (tool) => tool.id === message.meta.tool_call?.tool_id,
  );
  return (
    <div className="flex flex-col gap-1">
      {/* Tool header */}
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2 font-semibold">
          {tool && getToolIcon(tool)}
          <span
            className={cn(
              message.meta.tool_call?.is_error && "text-destructive-foreground",
            )}
          >
            {tool ? getToolTypeLabel(tool) : "Tool"}: {tool?.name || "Unknown"}
          </span>
        </div>
      </div>
      {message.meta.tool_call?.errors &&
        message.meta.tool_call.errors.length > 0 && (
          <div className="rounded-md bg-destructive/10 p-3 text-destructive-foreground">
            <div className="font-semibold">Error</div>
            <div className="text-sm">
              {message.meta.tool_call.errors.at(-1)}
            </div>
          </div>
        )}

      {/* Output */}
      <Collapsible open={showOutput} onOpenChange={setShowOutput}>
        <CollapsibleTrigger asChild>
          <Button
            variant="outline"
            size="sm"
            className="w-full justify-between"
          >
            <span className="flex items-center gap-1">
              {showOutput ? <ChevronUp /> : <ChevronDown />}Output
            </span>
          </Button>
        </CollapsibleTrigger>
        <CollapsibleContent>
          <Suspense
            fallback={<Markdown>{formatToolResponse(message)}</Markdown>}
          >
            <ChatFancyMarkdown>{formatToolResponse(message)}</ChatFancyMarkdown>
          </Suspense>
        </CollapsibleContent>
      </Collapsible>
      {!showOutput && (
        <div className="prose-pre:m-0">
          <pre className="text-nowrap">
            {message.content.includes("\n")
              ? `${message.content.slice(0, message.content.indexOf("\n"))}...`
              : message.content}
          </pre>
        </div>
      )}
      {/* Logs */}
      {message.meta.tool_call?.logs &&
        message.meta.tool_call.logs.length > 0 && (
          <ChatMessageToolLogs logs={message.meta.tool_call?.logs} />
        )}
    </div>
  );
}

function formatToolResponse(message: components["schemas"]["ChatRsMessage"]) {
  return `\`\`\`${message.content.startsWith("{") ? "json" : "text"}\n${escapeBackticks(message.content)}\n\`\`\``;
}
