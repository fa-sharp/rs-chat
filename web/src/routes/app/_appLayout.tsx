import Header from "@/components/Header";
import { AppSidebar } from "@/components/Sidebar";
import { SidebarInset, SidebarProvider } from "@/components/ui/sidebar";
import {
  getRecentChatSessions,
  useGetRecentChatSessions,
} from "@/lib/api/session";
import { getUser } from "@/lib/api/user";
import { createFileRoute, Outlet, redirect } from "@tanstack/react-router";

export const Route = createFileRoute("/app/_appLayout")({
  beforeLoad: async ({ context }) => {
    try {
      const user = await context.queryClient.ensureQueryData({
        queryKey: ["user"],
        queryFn: getUser,
      });
      return { user };
    } catch (error) {
      throw redirect({ to: "/" });
    }
  },
  loader: async ({ context }) => {
    await context.queryClient.ensureQueryData({
      queryKey: ["recentChatSessions"],
      queryFn: getRecentChatSessions,
    });
  },
  component: RouteComponent,
});

function RouteComponent() {
  const { user } = Route.useRouteContext();
  const { data } = useGetRecentChatSessions();

  return (
    <SidebarProvider>
      <AppSidebar user={user} sessions={data} />
      <SidebarInset className="overflow-hidden">
        <Header />
        <Outlet />
      </SidebarInset>
    </SidebarProvider>
  );
}
