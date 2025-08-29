import { useQueryClient } from "@tanstack/react-query";
import {
  Link,
  useLocation,
  useNavigate,
  useRouter,
} from "@tanstack/react-router";
import {
  Bot,
  ChevronsUpDown,
  KeyRound,
  LogOut,
  MessageCircleHeart,
  Minus,
  Plus,
  RefreshCwIcon,
  UserRound,
  Wrench,
} from "lucide-react";
import * as React from "react";

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
import { useCreateChatSession } from "@/lib/api/session";
import type { components } from "@/lib/api/types";
import { useStreamingChats } from "@/lib/context";
import { Avatar, AvatarFallback, AvatarImage } from "./ui/avatar";
import { Button } from "./ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuGroup,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "./ui/dropdown-menu";

export function AppSidebar({
  sessions,
  user,
  ...props
}: {
  sessions?: components["schemas"]["ChatRsSession"][];
  user?: components["schemas"]["ChatRsUser"];
} & React.ComponentProps<typeof Sidebar>) {
  const location = useLocation();
  const navigate = useNavigate();
  const router = useRouter();
  const queryClient = useQueryClient();

  const { streamedChats } = useStreamingChats();

  const { mutate: createChatSession, isPending: createChatPending } =
    useCreateChatSession();
  const onCreateChat = () => {
    createChatSession(undefined, {
      onSuccess: ({ session_id }) =>
        navigate({
          to: "/app/session/$sessionId",
          params: { sessionId: session_id },
        }),
    });
  };

  const groupedSessions = React.useMemo(
    () => groupSessionsByDate(sessions || []),
    [sessions],
  );

  const onLogout = React.useCallback(async () => {
    await fetch(`${import.meta.env.VITE_API_URL || ""}/api/auth/logout`, {
      method: "POST",
    });
    queryClient.invalidateQueries({ queryKey: ["user"] });
    router.invalidate();
  }, [queryClient, router]);

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
                            src={user?.avatar_url || ""}
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
              <DropdownMenuContent className="w-48" align="start">
                <DropdownMenuGroup>
                  <DropdownMenuItem asChild>
                    <Link to="/app/providers">
                      <Bot />
                      Providers
                    </Link>
                  </DropdownMenuItem>
                  <DropdownMenuItem asChild>
                    <Link to="/app/tools">
                      <Wrench />
                      Tools (Beta)
                    </Link>
                  </DropdownMenuItem>
                  <DropdownMenuItem asChild>
                    <Link to="/app/api-keys">
                      <KeyRound />
                      API Keys
                    </Link>
                  </DropdownMenuItem>
                  <DropdownMenuItem asChild>
                    <Link to="/app/profile">
                      <UserRound />
                      Profile
                    </Link>
                  </DropdownMenuItem>
                  <DropdownMenuItem onSelect={onLogout}>
                    <LogOut />
                    Logout
                  </DropdownMenuItem>
                </DropdownMenuGroup>
              </DropdownMenuContent>
            </DropdownMenu>
          </SidebarMenuItem>
        </SidebarMenu>
        <Button
          variant="outline"
          disabled={createChatPending}
          onClick={onCreateChat}
        >
          <Plus className="" />
          New Chat
        </Button>
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
                        <SidebarMenuSubItem key={session.id}>
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
                              {streamedChats?.[session.id]?.status ===
                                "streaming" && (
                                <RefreshCwIcon className="animate-spin" />
                              )}
                              <span className="overflow-hidden text-nowrap text-ellipsis">
                                {session.title}
                              </span>
                              <span className="shrink-0 text-muted-foreground ml-auto">
                                {group === "today" || group === "yesterday"
                                  ? formatTime(session.updated_at)
                                  : formatDate(session.updated_at)}
                              </span>
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
  lastWeekEnd.setHours(23, 59, 59, 999);

  const groups: Record<string, components["schemas"]["ChatRsSession"][]> = {
    today: [],
    yesterday: [],
    thisWeek: [],
    lastWeek: [],
    older: [],
  };

  sessions?.forEach((session) => {
    const sessionDate = new Date(session.updated_at);
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

const formatTime = (date: string) => {
  const dateObj = new Date(date);
  const options: Intl.DateTimeFormatOptions = {
    hour: "numeric",
    minute: "numeric",
  };
  return dateObj.toLocaleTimeString(undefined, options);
};

const formatDate = (date: string) => {
  const dateObj = new Date(date);
  const options: Intl.DateTimeFormatOptions = {
    month: "short",
    day: "numeric",
  };
  return dateObj.toLocaleDateString(undefined, options);
};
