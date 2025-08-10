import { createFileRoute } from "@tanstack/react-router";

import { ApiKeysManager } from "@/components/ApiKeysManager";

export const Route = createFileRoute("/app/_appLayout/api-keys")({
  component: RouteComponent,
});

function RouteComponent() {
  return (
    <div className="overflow-auto bg-background">
      <div className="container mx-auto px-4 py-8 max-w-3xl">
        <ApiKeysManager />
      </div>
    </div>
  );
}
