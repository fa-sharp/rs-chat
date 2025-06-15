import * as React from "react";
import {
  ChevronsUpDown,
  MessageCircleHeart,
  Minus,
  Plus,
  RefreshCwIcon,
} from "lucide-react";

import { SearchForm } from "@/components/SearchForm";
import {
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
} from "@/components/ui/collapsible";
import {
  Sidebar,
  SidebarContent,
  SidebarGroup,
  SidebarHeader,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
  SidebarMenuSub,
  SidebarMenuSubButton,
  SidebarMenuSubItem,
  SidebarRail,
} from "@/components/ui/sidebar";
import type { components } from "@/lib/api/types";
import { Avatar, AvatarFallback, AvatarImage } from "./ui/avatar";
import { Button } from "./ui/button";
import { Link, useLocation } from "@tanstack/react-router";
import { DropdownMenu, DropdownMenuTrigger } from "./ui/dropdown-menu";
import type { StreamedChat } from "@/lib/context/StreamingContext";

export function AppSidebar({
  sessions,
  user,
  streamedChats,
  ...props
}: {
  sessions?: components["schemas"]["ChatRsSession"][];
  user?: components["schemas"]["ChatRsUser"];
  streamedChats?: Record<string, StreamedChat | undefined>;
} & React.ComponentProps<typeof Sidebar>) {
  const groupedSessions = React.useMemo(
    () => groupSessionsByDate(sessions || []),
    [sessions],
  );
  const location = useLocation();

  return (
    <Sidebar {...props}>
      <SidebarHeader>
        <SidebarMenu>
          <SidebarMenuItem>
            <DropdownMenu>
              <DropdownMenuTrigger asChild>
                <SidebarMenuButton size="lg" tooltip="Open app menu" asChild>
                  <Button
                    type="button"
                    variant="ghost"
                    className="justify-between"
                  >
                    <div className="flex items-center gap-2">
                      <div className="flex aspect-square size-8 items-center justify-center rounded-lg">
                        <Avatar>
                          <AvatarImage
                            src={
                              !user
                                ? ""
                                : `https://avatars.githubusercontent.com/u/${user.github_id}`
                            }
                            alt="Avatar"
                          />
                          <AvatarFallback>
                            <MessageCircleHeart className="size-6" />
                          </AvatarFallback>
                        </Avatar>
                      </div>
                      {user ? (
                        <div className="flex flex-col gap-0.5 leading-none">
                          <span className="font-medium">{user.name}</span>
                        </div>
                      ) : (
                        <div className="flex flex-col gap-0.5 leading-none">
                          <span className="font-medium">RsChat</span>
                          <span className="">v1.0.0</span>
                        </div>
                      )}
                    </div>
                    <ChevronsUpDown />
                  </Button>
                </SidebarMenuButton>
              </DropdownMenuTrigger>
            </DropdownMenu>
          </SidebarMenuItem>
        </SidebarMenu>
        <SearchForm />
      </SidebarHeader>
      <SidebarContent>
        <SidebarGroup>
          <SidebarMenu>
            {groupedSessions.map(([group, chats], index) => (
              <Collapsible
                key={group}
                defaultOpen={index < 3}
                className="group/collapsible"
              >
                <SidebarMenuItem>
                  <CollapsibleTrigger asChild>
                    <SidebarMenuButton>
                      {DateGroups[group]}{" "}
                      <Plus className="ml-auto group-data-[state=open]/collapsible:hidden" />
                      <Minus className="ml-auto group-data-[state=closed]/collapsible:hidden" />
                    </SidebarMenuButton>
                  </CollapsibleTrigger>
                  <CollapsibleContent>
                    <SidebarMenuSub>
                      {chats.map((session) => (
                        <SidebarMenuSubItem key={session.title}>
                          <SidebarMenuSubButton
                            asChild
                            isActive={
                              location.pathname === `/app/session/${session.id}`
                            }
                          >
                            <Link
                              to="/app/session/$sessionId"
                              params={{ sessionId: session.id }}
                            >
                              {session.title}
                              {streamedChats?.[session.id]?.status ===
                                "streaming" && (
                                <RefreshCwIcon className="ml-auto animate-spin" />
                              )}
                            </Link>
                          </SidebarMenuSubButton>
                        </SidebarMenuSubItem>
                      ))}
                    </SidebarMenuSub>
                  </CollapsibleContent>
                </SidebarMenuItem>
              </Collapsible>
            ))}
          </SidebarMenu>
        </SidebarGroup>
      </SidebarContent>
      <SidebarRail />
    </Sidebar>
  );
}

const DateGroups: Record<string, string> = {
  today: "Today",
  yesterday: "Yesterday",
  thisWeek: "This Week",
  lastWeek: "Last Week",
  older: "Older",
};

function groupSessionsByDate(
  sessions: components["schemas"]["ChatRsSession"][],
) {
  const now = new Date();
  const today = new Date(now.getFullYear(), now.getMonth(), now.getDate());
  const yesterday = new Date(today);
  yesterday.setDate(yesterday.getDate() - 1);

  const thisWeekStart = new Date(today);
  thisWeekStart.setDate(today.getDate() - today.getDay());

  const lastWeekStart = new Date(thisWeekStart);
  lastWeekStart.setDate(thisWeekStart.getDate() - 7);
  const lastWeekEnd = new Date(thisWeekStart);
  lastWeekEnd.setDate(thisWeekStart.getDate() - 1);

  const groups: Record<string, components["schemas"]["ChatRsSession"][]> = {
    today: [],
    yesterday: [],
    thisWeek: [],
    lastWeek: [],
    older: [],
  };

  sessions?.forEach((session) => {
    const sessionDate = new Date(session.created_at);
    const sessionDateOnly = new Date(
      sessionDate.getFullYear(),
      sessionDate.getMonth(),
      sessionDate.getDate(),
    );

    if (sessionDateOnly.getTime() === today.getTime()) {
      groups.today.push(session);
    } else if (sessionDateOnly.getTime() === yesterday.getTime()) {
      groups.yesterday.push(session);
    } else if (sessionDate >= thisWeekStart && sessionDate < today) {
      groups.thisWeek.push(session);
    } else if (sessionDate >= lastWeekStart && sessionDate <= lastWeekEnd) {
      groups.lastWeek.push(session);
    } else {
      groups.older.push(session);
    }
  });

  return Object.entries(groups).filter(([, chats]) => chats.length > 0);
}
