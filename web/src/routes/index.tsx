import { createFileRoute, redirect } from "@tanstack/react-router";

import { LoginForm } from "@/components/LoginForm";
import { Card, CardContent } from "@/components/ui/card";
import { client } from "@/lib/api/client";

export const Route = createFileRoute("/")({
  beforeLoad: async ({ context }) => {
    if (context.user) throw redirect({ to: "/app" });
  },
  loader: async () => {
    const response = await client.GET("/auth/config");
    if (response.error) throw new Error("Failed to fetch auth configuration");
    return { authConfig: response.data };
  },
  component: RouteComponent,
});

function RouteComponent() {
  const { authConfig } = Route.useLoaderData();

  return (
    <div className="bg-background flex min-h-svh flex-col items-center justify-center gap-6 p-6 md:p-10">
      <Card>
        <CardContent>
          <LoginForm config={authConfig} />
        </CardContent>
      </Card>
    </div>
  );
}
