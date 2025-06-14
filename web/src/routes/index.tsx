import { createFileRoute } from "@tanstack/react-router";
import { useCallback, useState } from "react";

import Header from "@/components/Header";
import ChatMessageInput from "@/components/main/ChatMessageInput";
import ChatMessages from "@/components/main/ChatMessages";
import { AppSidebar } from "@/components/Sidebar";
import { SidebarInset, SidebarProvider } from "@/components/ui/sidebar";

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
Why is there so much padding holy shit.
## Example
### Sample code

\`\`\`jsx
import React from 'react';

const App = () => {
  return (
    <div>
      <h1>Hello, World! Here's some code in the
      chat.</h1>
    </div>
  );
};

export default App;
\`\`\`
    `,
    role: "Assistant",
    timestamp: new Date(),
  },
];

function App() {
  const [messages, setMessages] = useState(sampleMessages);
  const [isGenerating, setIsGenerating] = useState(false);

  const onSubmit = useCallback((message: string) => {
    setMessages((messages) => [
      ...messages,
      {
        id: crypto.randomUUID(),
        content: message,
        role: "User",
        timestamp: new Date(),
      },
    ]);
  }, []);

  return (
    <SidebarProvider>
      <AppSidebar />
      <SidebarInset className="overflow-hidden">
        <Header />
        <div className="grid grid-rows-[minmax(0,1fr)_auto] gap-4 p-4 overflow-hidden">
          <ChatMessages messages={messages} isGenerating={isGenerating} />
          <div className="w-full px-4 pb-4">
            <ChatMessageInput onSubmit={onSubmit} isGenerating={isGenerating} />
          </div>
        </div>
      </SidebarInset>
    </SidebarProvider>
  );
}
