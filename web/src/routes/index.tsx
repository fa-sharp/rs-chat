import { createFileRoute, redirect } from "@tanstack/react-router";

import { LoginForm } from "@/components/LoginForm";
import { Card, CardContent } from "@/components/ui/card";

export const Route = createFileRoute("/")({
  beforeLoad: async ({ context }) => {
    if (context.user) throw redirect({ to: "/app" });
  },
  component: RouteComponent,
});

function RouteComponent() {
  return (
    <div className="bg-background flex min-h-svh flex-col items-center justify-center gap-6 p-6 md:p-10">
      <Card>
        <CardContent>
          <LoginForm />
        </CardContent>
      </Card>
    </div>
  );
}
