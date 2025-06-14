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

interface Props {
  isGenerating: boolean;
  messages: Array<{
    id: string;
    content: string;
    role: string;
    timestamp: Date;
  }>;
}

export default function ChatMessages({ messages, isGenerating }: Props) {
  return (
    <ChatMessageList>
      {messages.map((message, index) => (
        <ChatBubble
          key={index}
          variant={message.role == "User" ? "sent" : "received"}
        >
          <ChatBubbleAvatar
            src=""
            fallback={message.role == "User" ? "🧑🏽‍💻" : "🤖"}
          />
          <ChatBubbleMessage className="prose">
            <Markdown
              key={index}
              remarkPlugins={[remarkGfm]}
              rehypePlugins={[
                rehypeHighlight,
                [rehypeHighlightCodeLines, { showLineNumbers: true }],
              ]}
            >
              {message.content}
            </Markdown>
            {message.role === "Assistant" && messages.length - 1 === index && (
              <div className="flex items-center mt-1.5 gap-1">
                {/* {!isGenerating && (
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
                    )} */}
              </div>
            )}
          </ChatBubbleMessage>
        </ChatBubble>
      ))}

      {isGenerating && (
        <ChatBubble variant="received">
          <ChatBubbleAvatar src="" fallback="🤖" />
          <ChatBubbleMessage isLoading />
        </ChatBubble>
      )}
    </ChatMessageList>
  );
}
