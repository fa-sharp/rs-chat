import { ApiKeysManager } from "@/components/ApiKeysManager";
import { createFileRoute } from "@tanstack/react-router";

export const Route = createFileRoute("/app/_appLayout/api-keys")({
  component: RouteComponent,
});

function RouteComponent() {
  return (
    <div className="overflow-auto bg-background">
      <div className="container mx-auto px-4 py-8 max-w-2xl">
        <ApiKeysManager />
      </div>
    </div>
  );
}
