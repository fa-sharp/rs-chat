import { ApiKeysManager } from "@/components/ApiKeysManager";
import { createFileRoute } from "@tanstack/react-router";

export const Route = createFileRoute("/app/_appLayout/api-keys")({
  component: RouteComponent,
});

function RouteComponent() {
  return (
    <div>
      <ApiKeysManager />
    </div>
  );
}
