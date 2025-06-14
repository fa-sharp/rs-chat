import Markdown from "react-markdown";
import { Separator } from "@radix-ui/react-separator";
import { createFileRoute } from "@tanstack/react-router";
import remarkGfm from "remark-gfm";
import rehypeHighlight from "rehype-highlight";

import { AppSidebar } from "@/components/Sidebar";
import {
  Breadcrumb,
  BreadcrumbItem,
  BreadcrumbLink,
  BreadcrumbList,
  BreadcrumbPage,
  BreadcrumbSeparator,
} from "@/components/ui/breadcrumb";
import {
  ChatBubble,
  ChatBubbleAction,
  ChatBubbleAvatar,
  ChatBubbleMessage,
} from "@/components/ui/chat/chat-bubble";
import { ChatMessageList } from "@/components/ui/chat/chat-message-list";
import {
  SidebarInset,
  SidebarProvider,
  SidebarTrigger,
} from "@/components/ui/sidebar";
import {
  useCallback,
  useState,
  type ChangeEventHandler,
  type FormEventHandler,
} from "react";
import { ChatInput } from "@/components/ui/chat/chat-input";
import { Button } from "@/components/ui/button";
import { CornerDownLeft, Mic, Paperclip } from "lucide-react";

export const Route = createFileRoute("/")({
  component: App,
});

const sampleMessages = [
  {
    id: "1",
    content: "Hello!",
    role: "User",
    timestamp: new Date(),
  },
  {
    id: "2",
    content: "How are you?",
    role: "Assistant",
    timestamp: new Date(),
  },
  {
    id: "3",
    content: `
Here's some sample code:
\`\`\`jsx
import React from 'react';

const App = () => {
  return (
    <div>
      <h1>Hello, World!</h1>
    </div>
  );
};

export default App;
\`\`\`
    `,
    role: "User",
    timestamp: new Date(),
  },
];

function App() {
  const [messages, setMessages] = useState(sampleMessages);

  const [enterKeyShouldSubmit, setEnterKeyShouldSubmit] = useState(true);

  const [isGenerating, setIsGenerating] = useState(false);
  const [inputValue, setInputValue] = useState("");
  const handleInputChange: ChangeEventHandler<HTMLTextAreaElement> =
    useCallback((ev) => {
      setInputValue(ev.target.value);
    }, []);

  const onKeyDown = (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
    if (e.key === "Enter" && !e.shiftKey && enterKeyShouldSubmit) {
      e.preventDefault();
      if (
        isGenerating // || isLoading || !input
      )
        return;
      setIsGenerating(true);
      onSubmit(e as unknown as React.FormEvent<HTMLFormElement>);
    }
  };

  const onSubmit: FormEventHandler<HTMLFormElement> = useCallback(
    (ev) => {
      ev.preventDefault();
      if (!inputValue) return;
      setMessages((messages) => [
        ...messages,
        {
          id: crypto.randomUUID(),
          content: inputValue,
          role: "User",
          timestamp: new Date(),
        },
      ]);
      setInputValue("");
    },
    [inputValue],
  );

  return (
    <SidebarProvider>
      <AppSidebar />
      <SidebarInset className="overflow-hidden">
        <header className="flex h-16 shrink-0 items-center gap-2 border-b px-4">
          <SidebarTrigger className="-ml-1" />
          <Separator
            orientation="vertical"
            className="mr-2 data-[orientation=vertical]:h-4"
          />
          <Breadcrumb>
            <BreadcrumbList>
              <BreadcrumbItem className="hidden md:block">
                <BreadcrumbLink href="#">
                  Building Your Application
                </BreadcrumbLink>
              </BreadcrumbItem>
              <BreadcrumbSeparator className="hidden md:block" />
              <BreadcrumbItem>
                <BreadcrumbPage>Data Fetching</BreadcrumbPage>
              </BreadcrumbItem>
            </BreadcrumbList>
          </Breadcrumb>
        </header>
        <div className="grid grid-rows-[minmax(0,1fr)_auto] gap-4 p-4 overflow-hidden">
          <ChatMessageList>
            {messages.map((message, index) => (
              <ChatBubble
                key={index}
                variant={message.role == "User" ? "sent" : "received"}
              >
                <ChatBubbleAvatar
                  src=""
                  fallback={message.role == "User" ? "👨🏽" : "🤖"}
                />
                <ChatBubbleMessage>
                  <Markdown
                    key={index}
                    remarkPlugins={[remarkGfm]}
                    rehypePlugins={[rehypeHighlight]}
                  >
                    {message.content}
                  </Markdown>
                  {message.role === "Assistant" &&
                    messages.length - 1 === index && (
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

            {/* Loading */}
            {isGenerating && (
              <ChatBubble variant="received">
                <ChatBubbleAvatar src="" fallback="🤖" />
                <ChatBubbleMessage isLoading />
              </ChatBubble>
            )}
          </ChatMessageList>
          <div className="w-full px-4 pb-4">
            <form
              // ref={formRef}
              onSubmit={onSubmit}
              className="relative rounded-lg border bg-background focus-within:ring-1 focus-within:ring-ring"
            >
              <ChatInput
                value={inputValue}
                onKeyDown={onKeyDown}
                onChange={handleInputChange}
                placeholder="Type your message here..."
                className="rounded-lg bg-background border-0 shadow-none focus-visible:ring-0"
              />
              <div className="flex items-center gap-2 p-3 pt-0">
                <Button
                  type="button"
                  variant={enterKeyShouldSubmit ? "default" : "outline"}
                  size="icon"
                  title="Toggle Enter key behavior"
                  onClick={() => setEnterKeyShouldSubmit((prev) => !prev)}
                >
                  <CornerDownLeft className="size-3.5" />
                  <span className="sr-only">Toggle Enter key</span>
                </Button>

                <Button type="button" variant="ghost" size="icon">
                  <Mic className="size-4" />
                  <span className="sr-only">Use Microphone</span>
                </Button>

                <Button
                  // disabled={!input || isLoading}
                  type="submit"
                  size="sm"
                  className="ml-auto gap-1.5"
                >
                  Send Message
                  <CornerDownLeft className="size-3.5" />
                </Button>
              </div>
            </form>
          </div>
        </div>
      </SidebarInset>
    </SidebarProvider>
  );
}
