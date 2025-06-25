import React, { Suspense, useCallback, useEffect } from "react";
import Markdown from "react-markdown";

import useSmoothStreaming from "@/hooks/useSmoothStreaming";
import { useDeleteChatMessage } from "@/lib/api/session";
import type { components } from "@/lib/api/types";
import { cn } from "@/lib/utils";
import {
  ChatBubble,
  ChatBubbleAvatar,
  ChatBubbleMessage,
} from "../ui/chat/chat-bubble";
import { ChatMessageList } from "../ui/chat/chat-message-list";
import { CopyButton, DeleteButton, InfoButton } from "./ChatMessageActions";

const ChatFancyMarkdown = React.lazy(() => import("./ChatFancyMarkdown"));

interface Props {
  isGenerating: boolean;
  isCompleted: boolean;
  user?: components["schemas"]["ChatRsUser"];
  messages: Array<components["schemas"]["ChatRsMessage"]>;
  streamedResponse?: string;
  error?: string;
}

const proseClasses =
  "prose prose-sm md:prose-base dark:prose-invert prose-pre:bg-primary-foreground prose-hr:my-3 prose-headings:not-[:first-child]:mt-3 prose-headings:mb-2 " +
  "prose-h1:text-3xl prose-ul:my-3 prose-ol:my-3 prose-p:leading-4 md:prose-p:leading-5.5 prose-li:my-1 prose-li:leading-4 md:prose-li:leading-5.5";
const proseUserClasses = "prose-code:text-primary-foreground";
const proseAssistantClasses = "prose-code:text-secondary-foreground";

export default function ChatMessages({
  user,
  messages,
  isGenerating,
  isCompleted,
  streamedResponse,
  error,
}: Props) {
  const {
    displayedText: animatedText,
    complete,
    reset,
  } = useSmoothStreaming(streamedResponse);
  useEffect(() => {
    if (isCompleted) complete();
  }, [isCompleted, complete]);
  useEffect(() => {
    if (!streamedResponse) reset();
  }, [streamedResponse, reset]);

  const { mutate: deleteMessage } = useDeleteChatMessage();
  const onDeleteMessage = useCallback(
    (sessionId: string, messageId: string) => {
      deleteMessage({ sessionId, messageId });
    },
    [deleteMessage],
  );

  return (
    <ChatMessageList>
      {messages
        .filter(
          (message, idx) =>
            !streamedResponse ||
            !(message.meta?.interrupted && idx === messages.length - 1), // don't show the partial assistant response if still streaming
        )
        .map((message) => (
          <ChatBubble
            key={message.id}
            variant={message.role === "User" ? "sent" : "received"}
          >
            <ChatBubbleAvatar
              src={
                message.role === "User" && user
                  ? `https://avatars.githubusercontent.com/u/${user.github_id}`
                  : ""
              }
              fallback={message.role === "User" ? "🧑🏽‍💻" : "🤖"}
            />
            <ChatBubbleMessage
              className={cn(
                proseClasses,
                message.role === "User" && proseUserClasses,
                message.role === "Assistant" && proseAssistantClasses,
              )}
            >
              <Suspense fallback={<Markdown>{message.content}</Markdown>}>
                <ChatFancyMarkdown>{message.content}</ChatFancyMarkdown>
              </Suspense>
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

      {animatedText && (
        <ChatBubble variant="received">
          <ChatBubbleAvatar fallback="🤖" className="animate-pulse" />
          <ChatBubbleMessage
            className={cn(
              proseClasses,
              proseAssistantClasses,
              "outline-2 outline-ring",
            )}
          >
            <Markdown key="streaming">{animatedText}</Markdown>
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
