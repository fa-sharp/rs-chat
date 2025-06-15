import Markdown from "react-markdown";
import rehypeHighlight from "rehype-highlight";
import rehypeHighlightCodeLines from "rehype-highlight-code-lines";
import remarkGfm from "remark-gfm";
import {
  ChatBubble,
  ChatBubbleAction,
  ChatBubbleActionWrapper,
  ChatBubbleAvatar,
  ChatBubbleMessage,
} from "../ui/chat/chat-bubble";
import { ChatMessageList } from "../ui/chat/chat-message-list";
import type { components } from "@/lib/api/types";
import { cn } from "@/lib/utils";
import { Copy, Delete, Trash2 } from "lucide-react";

interface Props {
  isGenerating: boolean;
  user?: components["schemas"]["ChatRsUser"];
  messages: Array<components["schemas"]["ChatRsMessage"]>;
  streamedResponse?: string;
  error?: string;
}

const proseClasses = "prose prose-sm md:prose-base";

export default function ChatMessages({
  user,
  messages,
  isGenerating,
  streamedResponse,
  error,
}: Props) {
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
              message.role == "User" && "prose-code:text-primary-foreground",
              message.role == "Assistant" &&
                "prose-code:text-secondary-foreground",
            )}
          >
            <Markdown
              key={message.id}
              remarkPlugins={[remarkGfm]}
              rehypePlugins={[
                rehypeHighlight,
                [rehypeHighlightCodeLines, { showLineNumbers: true }],
              ]}
            >
              {message.content}
            </Markdown>
            {/* {message.role === "Assistant" && (
              <div className="flex items-center gap-1">
                <ChatBubbleAction
                  variant="outline"
                  className="size-6"
                  icon={<Copy className="size-3" />}
                  onClick={() => navigator.clipboard.writeText(message.content)}
                />
              </div>
            )} */}
          </ChatBubbleMessage>
          <ChatBubbleActionWrapper className="flex gap-1">
            <ChatBubbleAction
              variant="outline"
              className="size-6"
              icon={<Copy className="size-3" />}
              onClick={() => navigator.clipboard.writeText(message.content)}
            />
            <ChatBubbleAction
              variant="destructive"
              className="size-6"
              icon={<Trash2 className="size-3" />}
              onClick={() => {}} // TODO Delete
            />
          </ChatBubbleActionWrapper>
        </ChatBubble>
      ))}

      {isGenerating && (
        <ChatBubble variant="received">
          <ChatBubbleAvatar fallback="🤖" />
          <ChatBubbleMessage isLoading />
        </ChatBubble>
      )}

      {streamedResponse && (
        <ChatBubble variant="received">
          <ChatBubbleAvatar fallback="🤖" />
          <ChatBubbleMessage
            className={cn(proseClasses, "outline-2 outline-ring")}
          >
            <Markdown key="streaming">{streamedResponse}</Markdown>
          </ChatBubbleMessage>
        </ChatBubble>
      )}

      {error && (
        <ChatBubble variant="received">
          <ChatBubbleAvatar fallback="🤖" />
          <ChatBubbleMessage className="text-destructive">
            {error}
          </ChatBubbleMessage>
        </ChatBubble>
      )}
    </ChatMessageList>
  );
}
