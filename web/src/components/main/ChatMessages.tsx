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
import type { components } from "@/lib/api/types";

interface Props {
  isGenerating: boolean;
  messages: Array<components["schemas"]["ChatRsMessage"]>;
  streamedResponse?: string;
}

export default function ChatMessages({
  messages,
  isGenerating,
  streamedResponse,
}: Props) {
  return (
    <ChatMessageList>
      {messages.map((message) => (
        <ChatBubble
          key={message.id}
          variant={message.role == "User" ? "sent" : "received"}
        >
          <ChatBubbleAvatar
            src=""
            fallback={message.role == "User" ? "🧑🏽‍💻" : "🤖"}
          />
          <ChatBubbleMessage className="prose prose-sm md:prose-base">
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
            {/* {message.role === "Assistant" && messages.length - 1 === index && (
              <div className="flex items-center mt-1.5 gap-1">
                {!isGenerating && (
                      <>
                        {ChatAiIcons.map((icon, iconIndex) => {
                          const Icon = icon.icon;
                          return (
                            <ChatBubbleAction
                              variant="outline"
                              className="size-5"
                              key={iconIndex}
                              icon={<Icon className="size-3" />}
                              onClick={
                                () => {}
                                // handleActionClick(icon.label, index)
                              }
                            />
                          );
                        })}
                      </>
                    )}
              </div>
            )} */}
          </ChatBubbleMessage>
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
          <ChatBubbleMessage className="prose outline-2 outline-ring">
            <Markdown
              key="streaming"
              // remarkPlugins={[remarkGfm]}
              // rehypePlugins={[
              //   rehypeHighlight,
              //   [rehypeHighlightCodeLines, { showLineNumbers: true }],
              // ]}
            >
              {streamedResponse}
            </Markdown>
          </ChatBubbleMessage>
        </ChatBubble>
      )}
    </ChatMessageList>
  );
}
