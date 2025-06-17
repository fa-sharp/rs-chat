import { useDeleteChatMessage } from "@/lib/api/session";
import type { components } from "@/lib/api/types";
import { cn } from "@/lib/utils";
import { useCallback, useRef, useState } from "react";
import Markdown from "react-markdown";
import rehypeHighlight from "rehype-highlight";
import rehypeHighlightCodeLines from "rehype-highlight-code-lines";
import remarkGfm from "remark-gfm";
import {
  ChatBubble,
  ChatBubbleAvatar,
  ChatBubbleMessage,
} from "../ui/chat/chat-bubble";
import { ChatMessageList } from "../ui/chat/chat-message-list";
import { CopyButton, DeleteButton } from "./ChatMessageActions";
import type { ReactNode } from "@tanstack/react-router";
import { Check, Copy } from "lucide-react";
import { Button } from "../ui/button";

interface Props {
  isGenerating: boolean;
  user?: components["schemas"]["ChatRsUser"];
  messages: Array<components["schemas"]["ChatRsMessage"]>;
  streamedResponse?: string;
  error?: string;
}

const proseClasses =
  "prose prose-sm dark:prose-invert prose-pre:bg-primary-foreground";
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
            <MarkdownWithPlugins>{message.content}</MarkdownWithPlugins>
            {message.role === "Assistant" && (
              <div className="flex items-center gap-1 opacity-65 hover:opacity-100 focus-within:opacity-100">
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

function MarkdownWithPlugins({ children }: { children: ReactNode }) {
  return (
    <Markdown
      remarkPlugins={[remarkGfm]}
      rehypePlugins={[
        rehypeHighlight,
        [rehypeHighlightCodeLines, { showLineNumbers: true }],
      ]}
      components={{
        pre: CodeWrapper,
      }}
    >
      {children}
    </Markdown>
  );
}

/** Wrapper for code blocks with copy button */
function CodeWrapper({ children }: { children?: ReactNode }) {
  const ref = useRef<HTMLPreElement>(null);
  const [isCopied, setIsCopied] = useState(false);

  const handleCopy = async () => {
    if (ref.current) {
      try {
        await navigator.clipboard.writeText(
          ref.current.innerText.slice(5).trim(), // Slice off the text of the copy button
        );
        setIsCopied(true);
        setTimeout(() => setIsCopied(false), 2000);
      } catch (error) {
        console.error("Failed to copy text:", error);
      }
    }
  };

  if (!children) return null;
  return (
    <div className="not-prose">
      <pre ref={ref} className="relative">
        <Button
          className="absolute top-2 right-2 opacity-85 hover:opacity-100"
          onClick={handleCopy}
          variant="outline"
          size="sm"
        >
          {isCopied ? (
            <Check className="size-4 text-green-600" />
          ) : (
            <Copy className="size-3" />
          )}
          Copy
        </Button>
        {children}
      </pre>
    </div>
  );
}
