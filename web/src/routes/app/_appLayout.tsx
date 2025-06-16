import { createFileRoute, Outlet, redirect } from "@tanstack/react-router";

import Header from "@/components/Header";
import { AppSidebar } from "@/components/Sidebar";
import { SidebarInset, SidebarProvider } from "@/components/ui/sidebar";
import {
  getRecentChatSessions,
  useGetRecentChatSessions,
} from "@/lib/api/session";
import { useStreamingChats } from "@/lib/context/StreamingContext";

export const Route = createFileRoute("/app/_appLayout")({
  beforeLoad: async ({ context }) => {
    if (!context.user) {
      throw redirect({ to: "/" });
    }
    return { user: context.user };
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
  const { streamedChats } = useStreamingChats();

  return (
    <SidebarProvider>
      <AppSidebar user={user} sessions={data} streamedChats={streamedChats} />
      <SidebarInset className="overflow-hidden">
        <Header />
        <Outlet />
      </SidebarInset>
    </SidebarProvider>
  );
}
