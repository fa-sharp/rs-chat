import { Bot, Wrench } from "lucide-react";
import React, { Suspense } from "react";
import Markdown from "react-markdown";

import {
  ChatBubble,
  ChatBubbleAvatar,
  ChatBubbleMessage,
} from "@/components/ui/chat/chat-bubble";
import type { components } from "@/lib/api/types";
import { cn, escapeBackticks } from "@/lib/utils";
import { CopyButton, DeleteButton, InfoButton } from "./ChatMessageActions";
import ChatMessageToolCalls from "./ChatMessageToolCalls";
import {
  proseAssistantClasses,
  proseClasses,
  proseUserClasses,
} from "./proseStyles";

const ChatFancyMarkdown = React.lazy(() => import("./ChatFancyMarkdown"));

interface Props {
  message: components["schemas"]["ChatRsMessage"];
  user?: components["schemas"]["ChatRsUser"];
  tools?: components["schemas"]["ChatRsToolPublic"][];
  executedToolCalls?: components["schemas"]["ChatRsToolCall"][];
  onExecuteToolCall: (messageId: string, toolCallId: string) => void;
  onExecuteAllToolCalls: (messageId: string) => void;
  isExecutingTool: boolean;
  providers?: components["schemas"]["ChatRsProvider"][];
  onDeleteMessage: (messageId: string) => void;
}

export default function ChatMessage({
  message,
  user,
  tools,
  executedToolCalls,
  onExecuteToolCall,
  onExecuteAllToolCalls,
  isExecutingTool,
  providers,
  onDeleteMessage,
}: Props) {
  return (
    <ChatBubble
      layout={message.role === "Assistant" ? "ai" : "default"}
      variant={message.role === "User" ? "sent" : "received"}
    >
      <ChatBubbleAvatar
        src={(message.role === "User" && user?.avatar_url) || undefined}
        fallback={
          message.role === "User" ? (
            "🧑🏽‍💻"
          ) : message.role === "Tool" ? (
            <Wrench className="size-4" />
          ) : (
            <Bot className="size-4" />
          )
        }
      />
      <ChatBubbleMessage
        variant={message.role === "User" ? "sent" : "received"}
        layout={message.role === "Assistant" ? "ai" : "default"}
        className={cn(
          proseClasses,
          message.role === "User" && proseUserClasses,
          message.role === "Assistant" && proseAssistantClasses,
        )}
      >
        <Suspense
          fallback={
            <Markdown>
              {message.role === "Tool"
                ? formatToolResponse(message)
                : message.content}
            </Markdown>
          }
        >
          <ChatFancyMarkdown>
            {message.role === "Tool"
              ? formatToolResponse(message)
              : message.content}
          </ChatFancyMarkdown>
        </Suspense>
        {message.role === "Assistant" && (
          <>
            {message.meta.assistant?.tool_calls && (
              <ChatMessageToolCalls
                tools={tools}
                toolCalls={message.meta.assistant.tool_calls}
                executedToolCalls={executedToolCalls}
                onExecute={(id) => onExecuteToolCall(message.id, id)}
                onExecuteAll={() => onExecuteAllToolCalls(message.id)}
                isExecuting={isExecutingTool}
              />
            )}
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-2 opacity-65 hover:opacity-100 focus-within:opacity-100">
                <InfoButton meta={message.meta} providers={providers} />
                <CopyButton message={message.content} />
                <DeleteButton onDelete={() => onDeleteMessage(message.id)} />
              </div>
              <div className="text-xs text-muted-foreground">
                {formatDate(message.created_at)}
              </div>
            </div>
          </>
        )}
        {message.role === "User" && (
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2 opacity-65 hover:opacity-100 focus-within:opacity-100">
              <CopyButton message={message.content} variant="default" />
              <DeleteButton
                onDelete={() => onDeleteMessage(message.id)}
                variant="default"
              />
            </div>
            <div className="text-xs text-muted">
              {formatDate(message.created_at)}
            </div>
          </div>
        )}
        {message.role === "Tool" && (
          <div className="flex items-center justify-between">
            <div className="flex items-center mt-2 gap-2 opacity-65 hover:opacity-100 focus-within:opacity-100">
              <InfoButton meta={message.meta} providers={providers} />
              <CopyButton message={message.content} />
              <DeleteButton onDelete={() => onDeleteMessage(message.id)} />
            </div>
            <div className="text-xs text-muted-foreground">
              {formatDate(message.created_at)}
            </div>
          </div>
        )}
      </ChatBubbleMessage>
    </ChatBubble>
  );
}

function formatToolResponse(message: components["schemas"]["ChatRsMessage"]) {
  return `### Tool Response: ${message.meta.tool_call?.tool_name}\n\`\`\`${message.content.startsWith("{") ? "json" : "text"}\n${escapeBackticks(message.content)}\n\`\`\``;
}

const now = new Date();
const formatDate = (date: string) => {
  const parsedDate = new Date(date);
  const isToday = parsedDate.toDateString() === now.toDateString();
  return new Intl.DateTimeFormat(undefined, {
    year:
      parsedDate.getFullYear() === now.getFullYear() ? undefined : "numeric",
    month: isToday ? undefined : "short",
    day: isToday ? undefined : "numeric",
    hour: "numeric",
    minute: "numeric",
  }).format(parsedDate);
};
