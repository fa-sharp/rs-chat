import { createFileRoute } from "@tanstack/react-router";

import { ToolsManager } from "@/components/ToolsManager";

export const Route = createFileRoute("/app/_appLayout/tools")({
  component: RouteComponent,
});

function RouteComponent() {
  return (
    <div className="overflow-auto bg-background">
      <div className="container mx-auto px-4 py-8 max-w-3xl">
        <ToolsManager />
      </div>
    </div>
  );
}
