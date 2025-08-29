import { Bot } from "lucide-react";
import { useEffect } from "react";
import Markdown from "react-markdown";

import useSmoothStreaming from "@/hooks/useSmoothStreaming";
import type { StreamingChat } from "@/lib/context";
import { cn } from "@/lib/utils";
import {
  ChatBubble,
  ChatBubbleAvatar,
  ChatBubbleMessage,
} from "../ui/chat/chat-bubble";
import { proseAssistantClasses, proseClasses } from "./messages/proseStyles";

interface Props {
  sessionId: string;
  currentStream?: StreamingChat;
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
  } = useSmoothStreaming(currentStream?.text);
  useEffect(() => {
    if (currentStream?.status === "completed") complete();
  }, [currentStream?.status, complete]);
  useEffect(() => {
    if (!currentStream?.text) reset();
  }, [currentStream?.text, reset]);
  useEffect(() => {
    if (sessionId) reset();
  }, [sessionId, reset]);

  return (
    <>
      {currentStream?.status === "streaming" && currentStream?.text === "" && (
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

      {currentStream &&
        currentStream.errors.length > 0 &&
        currentStream.errors.map((error, idx) => (
          <ChatBubble key={`${error + idx}`} variant="received">
            <ChatBubbleAvatar fallback={<Bot className="size-4" />} />
            <ChatBubbleMessage className="text-destructive-foreground">
              {error}
            </ChatBubbleMessage>
          </ChatBubble>
        ))}
    </>
  );
}
