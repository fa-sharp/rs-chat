import { ApiKeysManager } from "@/components/ApiKeysManager";
import { createFileRoute } from "@tanstack/react-router";

export const Route = createFileRoute("/app/_appLayout/api-keys")({
  component: RouteComponent,
});

function RouteComponent() {
  return (
    <div className="p-10">
      <ApiKeysManager />
    </div>
  );
}
