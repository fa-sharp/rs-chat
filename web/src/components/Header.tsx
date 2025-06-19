import { Separator } from "@radix-ui/react-separator";
import { createLink, useMatchRoute } from "@tanstack/react-router";
import { Edit2, Trash } from "lucide-react";
import { useState } from "react";

import { useGetChatSession } from "@/lib/api/session";
import { ThemeToggle } from "./theme/ThemeToggle";
import {
  Breadcrumb,
  BreadcrumbItem,
  BreadcrumbLink,
  BreadcrumbList,
  BreadcrumbPage,
  BreadcrumbSeparator,
} from "./ui/breadcrumb";
import { Button } from "./ui/button";
import { Input } from "./ui/input";
import { SidebarTrigger } from "./ui/sidebar";

const RouterBreadcrumbLink = createLink(BreadcrumbLink);

export default function Header() {
  const matchRoute = useMatchRoute();
  const sessionRouteMatch = matchRoute({ to: "/app/session/$sessionId" });
  const { data: session } = useGetChatSession(
    sessionRouteMatch ? sessionRouteMatch.sessionId : "",
  );

  const [isEditingTitle, setIsEditingTitle] = useState(false);

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
          {sessionRouteMatch &&
            (!isEditingTitle ? (
              <>
                <BreadcrumbSeparator className="hidden md:block" />
                <BreadcrumbItem>
                  <BreadcrumbPage>{session?.session.title}</BreadcrumbPage>
                </BreadcrumbItem>
                <Button
                  size="icon"
                  variant="ghost"
                  className="size-6"
                  onClick={() => setIsEditingTitle(true)}
                >
                  <Edit2 className="size-4" />
                </Button>
                <Button size="icon" variant="ghost" className="size-6">
                  <Trash className="size-4" />
                </Button>
              </>
            ) : (
              <>
                <BreadcrumbSeparator className="hidden md:block" />
                <BreadcrumbItem>
                  <form
                    onSubmit={(e) => {
                      e.preventDefault();
                      setIsEditingTitle(false);
                      // Update session title in API
                    }}
                  >
                    <Input
                      autoFocus
                      className="text-foreground"
                      defaultValue={session?.session.title}
                    />
                  </form>
                </BreadcrumbItem>
              </>
            ))}
        </BreadcrumbList>
      </Breadcrumb>
      <ThemeToggle className="ml-auto" />
    </header>
  );
}
