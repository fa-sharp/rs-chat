import { createFileRoute } from "@tanstack/react-router";

export const Route = createFileRoute("/app/_appLayout/")({
  component: RouteComponent,
});

function RouteComponent() {
  return (
    <div className="flex-1 items-center justify-center">
      This is the app index page
    </div>
  );
}
