import { getUser } from "@/lib/api/user";
import { createFileRoute, redirect } from "@tanstack/react-router";

export const Route = createFileRoute("/")({
  beforeLoad: async ({ context }) => {
    try {
      await context.queryClient.fetchQuery({
        queryKey: ["user"],
        queryFn: getUser,
        retry: 1,
      });
      throw redirect({ to: "/app" });
    } finally {
      // User not logged in. Do nothing
    }
  },
  component: RouteComponent,
});

function RouteComponent() {
  return <div>Login form</div>;
}
