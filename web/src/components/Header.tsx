import { Separator } from "@radix-ui/react-separator";
import {
  Breadcrumb,
  BreadcrumbList,
  BreadcrumbItem,
  BreadcrumbLink,
  BreadcrumbSeparator,
  BreadcrumbPage,
} from "./ui/breadcrumb";
import { SidebarTrigger } from "./ui/sidebar";
import { createLink, useMatchRoute } from "@tanstack/react-router";
import { useGetChatSession } from "@/lib/api/session";
import { ThemeToggle } from "./theme/ThemeToggle";

const RouterBreadcrumbLink = createLink(BreadcrumbLink);

export default function Header() {
  const matchRoute = useMatchRoute();
  const sessionRouteMatch = matchRoute({ to: "/app/session/$sessionId" });
  const { data: session } = useGetChatSession(
    sessionRouteMatch ? sessionRouteMatch.sessionId : "",
  );

  return (
    <header className="flex h-16 shrink-0 items-center gap-2 border-b px-4">
      <SidebarTrigger className="-ml-1" />
      <Separator
        orientation="vertical"
        className="mr-2 data-[orientation=vertical]:h-4"
      />
      <Breadcrumb>
        <BreadcrumbList>
          <BreadcrumbItem className="hidden md:block">
            <RouterBreadcrumbLink to="/app">Chats</RouterBreadcrumbLink>
          </BreadcrumbItem>
          {sessionRouteMatch && (
            <>
              <BreadcrumbSeparator className="hidden md:block" />
              <BreadcrumbItem>
                <BreadcrumbPage>{session?.session.title}</BreadcrumbPage>
              </BreadcrumbItem>
            </>
          )}
        </BreadcrumbList>
      </Breadcrumb>
      <ThemeToggle className="ml-auto" />
    </header>
  );
}
