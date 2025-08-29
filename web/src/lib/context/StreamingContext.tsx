import { ChatStreamContext, useStreamManager } from "./streamManager";

export const ChatStreamProvider = ({
  children,
}: {
  children: React.ReactNode;
}) => {
  const chatStreamManager = useStreamManager();

  return (
    <ChatStreamContext.Provider value={chatStreamManager}>
      {children}
    </ChatStreamContext.Provider>
  );
};
