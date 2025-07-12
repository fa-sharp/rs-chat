import { createFileRoute, Link, useNavigate } from "@tanstack/react-router";
import { KeyRound } from "lucide-react";

import { Button } from "@/components/ui/button";
import { Card, CardContent, CardTitle } from "@/components/ui/card";
import { useProviderKeys } from "@/lib/api/providerKey";
import { useCreateChatSession } from "@/lib/api/session";

export const Route = createFileRoute("/app/_appLayout/")({
  component: RouteComponent,
});

function RouteComponent() {
  const navigate = useNavigate();
  const { mutate: createChatSession, isPending } = useCreateChatSession();
  const { data: apiKeys } = useProviderKeys();

  const onCreateChat = () => {
    createChatSession(undefined, {
      onSuccess: ({ session_id }) =>
        navigate({
          to: "/app/session/$sessionId",
          params: { sessionId: session_id },
        }),
    });
  };

  if (apiKeys && apiKeys.length === 0) {
    return (
      <div className="flex-1 flex items-center justify-center">
        <Card className="flex flex-col gap-4 items-center">
          <CardTitle className="text-2xl">Welcome!</CardTitle>
          <CardContent>
            Seems like you're new around here 👋. Add an API key below, and then
            you can start chatting!
          </CardContent>
          <Button asChild>
            <Link to="/app/api-keys">
              <KeyRound /> API keys
            </Link>
          </Button>
        </Card>
      </div>
    );
  }

  return (
    <div className="flex-1 flex items-center justify-center">
      <Button disabled={isPending} onClick={onCreateChat}>
        New Chat
      </Button>
    </div>
  );
}
