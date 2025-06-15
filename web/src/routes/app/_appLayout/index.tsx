import { Button } from "@/components/ui/button";
import { createFileRoute } from "@tanstack/react-router";

export const Route = createFileRoute("/app/_appLayout/")({
  component: RouteComponent,
});

function RouteComponent() {
  return (
    <div className="flex-1 flex items-center justify-center">
      <Button>New Chat</Button>
    </div>
  );
}
