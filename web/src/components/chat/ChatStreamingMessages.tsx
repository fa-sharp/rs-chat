import { Bot } from "lucide-react";
import { useEffect } from "react";
import Markdown from "react-markdown";

import useSmoothStreaming from "@/hooks/useSmoothStreaming";
import type { StreamedChat } from "@/lib/context";
import { cn } from "@/lib/utils";
import {
  ChatBubble,
  ChatBubbleAvatar,
  ChatBubbleMessage,
} from "../ui/chat/chat-bubble";
import { proseAssistantClasses, proseClasses } from "./messages/proseStyles";

interface Props {
  sessionId: string;
  currentStream?: StreamedChat;
}

/** Displays currently streaming assistant responses and errors */
export default function ChatStreamingMessages({
  sessionId,
  currentStream,
}: Props) {
  const {
    displayedText: streamingMessage,
    complete,
    reset,
  } = useSmoothStreaming(currentStream?.content);
  useEffect(() => {
    if (currentStream?.status === "completed") complete();
  }, [currentStream?.status, complete]);
  useEffect(() => {
    if (!currentStream?.content) reset();
  }, [currentStream?.content, reset]);
  useEffect(() => {
    if (sessionId) reset();
  }, [sessionId, reset]);

  return (
    <>
      {currentStream?.status === "streaming" &&
        currentStream?.content === "" && (
          <ChatBubble variant="received">
            <ChatBubbleAvatar
              fallback={<Bot className="size-4" />}
              className="animate-pulse"
            />
            <ChatBubbleMessage isLoading />
          </ChatBubble>
        )}

      {streamingMessage && (
        <ChatBubble variant="received" layout="ai">
          <ChatBubbleAvatar
            fallback={<Bot className="size-4" />}
            className="animate-pulse"
          />
          <ChatBubbleMessage
            className={cn(
              proseClasses,
              proseAssistantClasses,
              "outline-2 outline-ring",
            )}
          >
            <Markdown key="streaming">{streamingMessage}</Markdown>
          </ChatBubbleMessage>
        </ChatBubble>
      )}

      {currentStream?.error && (
        <ChatBubble variant="received">
          <ChatBubbleAvatar fallback={<Bot className="size-4" />} />
          <ChatBubbleMessage className="text-destructive-foreground">
            {currentStream.error}
          </ChatBubbleMessage>
        </ChatBubble>
      )}
    </>
  );
}
