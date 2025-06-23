import { createFileRoute, Outlet, redirect } from "@tanstack/react-router";

import ErrorComponent from "@/components/Error";
import Header from "@/components/Header";
import SearchDialog from "@/components/SearchDialog";
import { AppSidebar } from "@/components/Sidebar";
import { SidebarInset, SidebarProvider } from "@/components/ui/sidebar";
import {
  getRecentChatSessions,
  recentSessionsQueryKey,
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
      queryKey: recentSessionsQueryKey,
      queryFn: getRecentChatSessions,
    });
  },
  component: RouteComponent,
  errorComponent: ErrorComponent,
  pendingComponent: () => (
    <div className="min-h-screen bg-background flex items-center justify-center">
      <div className="text-center">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-primary mx-auto mb-4"></div>
        <p className="text-muted-foreground">Loading RsChat...</p>
      </div>
    </div>
  ),
});

function RouteComponent() {
  const { user } = Route.useRouteContext();
  const { data } = useGetRecentChatSessions();
  const { streamedChats } = useStreamingChats();

  return (
    <SidebarProvider>
      <SearchDialog />
      <AppSidebar user={user} sessions={data} streamedChats={streamedChats} />
      <SidebarInset className="overflow-hidden">
        <Header />
        <Outlet />
      </SidebarInset>
    </SidebarProvider>
  );
}
