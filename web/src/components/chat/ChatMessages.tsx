import { useCallback } from "react";
import Markdown from "react-markdown";

import {
  ChatBubble,
  ChatBubbleAvatar,
  ChatBubbleMessage,
} from "../ui/chat/chat-bubble";
import { ChatMessageList } from "../ui/chat/chat-message-list";

import { CopyButton, DeleteButton, InfoButton } from "./ChatMessageActions";
import { useDeleteChatMessage } from "@/lib/api/session";
import type { components } from "@/lib/api/types";
import { cn } from "@/lib/utils";
import ChatFancyMarkdown from "./ChatFancyMarkdown";

interface Props {
  isGenerating: boolean;
  user?: components["schemas"]["ChatRsUser"];
  messages: Array<components["schemas"]["ChatRsMessage"]>;
  streamedResponse?: string;
  error?: string;
}

const proseClasses =
  "prose prose-sm md:prose-base dark:prose-invert prose-pre:bg-primary-foreground prose-hr:my-3 prose-li:my-1";
const proseUserClasses = "prose-code:text-primary-foreground";
const proseAssistantClasses = "prose-code:text-secondary-foreground";

export default function ChatMessages({
  user,
  messages,
  isGenerating,
  streamedResponse,
  error,
}: Props) {
  const { mutate: deleteMessage } = useDeleteChatMessage();
  const onDeleteMessage = useCallback(
    (sessionId: string, messageId: string) => {
      deleteMessage({ sessionId, messageId });
    },
    [],
  );

  return (
    <ChatMessageList>
      {messages.map((message) => (
        <ChatBubble
          key={message.id}
          variant={message.role == "User" ? "sent" : "received"}
        >
          <ChatBubbleAvatar
            src={
              message.role === "User" && user
                ? `https://avatars.githubusercontent.com/u/${user.github_id}`
                : ""
            }
            fallback={message.role == "User" ? "🧑🏽‍💻" : "🤖"}
          />
          <ChatBubbleMessage
            className={cn(
              proseClasses,
              message.role == "User" && proseUserClasses,
              message.role == "Assistant" && proseAssistantClasses,
            )}
          >
            <ChatFancyMarkdown>{message.content}</ChatFancyMarkdown>
            {message.role === "Assistant" && (
              <div className="flex items-center gap-2 opacity-65 hover:opacity-100 focus-within:opacity-100">
                <InfoButton meta={message.meta} />
                <CopyButton message={message.content} />
                <DeleteButton
                  onDelete={() =>
                    onDeleteMessage(message.session_id, message.id)
                  }
                />
              </div>
            )}
          </ChatBubbleMessage>
        </ChatBubble>
      ))}

      {isGenerating && (
        <ChatBubble variant="received">
          <ChatBubbleAvatar fallback="🤖" className="animate-pulse" />
          <ChatBubbleMessage isLoading />
        </ChatBubble>
      )}

      {streamedResponse && (
        <ChatBubble variant="received">
          <ChatBubbleAvatar fallback="🤖" className="animate-pulse" />
          <ChatBubbleMessage
            className={cn(
              proseClasses,
              proseAssistantClasses,
              "outline-2 outline-ring",
            )}
          >
            <Markdown key="streaming">{streamedResponse}</Markdown>
          </ChatBubbleMessage>
        </ChatBubble>
      )}

      {error && (
        <ChatBubble variant="received">
          <ChatBubbleAvatar fallback="🤖" />
          <ChatBubbleMessage className="text-destructive-foreground">
            {error}
          </ChatBubbleMessage>
        </ChatBubble>
      )}
    </ChatMessageList>
  );
}
