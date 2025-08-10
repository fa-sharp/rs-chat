import { Separator } from "@radix-ui/react-separator";
import { createLink, useMatchRoute, useNavigate } from "@tanstack/react-router";
import { Edit2, Trash, X } from "lucide-react";
import { type FormEventHandler, useCallback, useState } from "react";

import {
  useDeleteChatSession,
  useGetChatSession,
  useUpdateChatSession,
} from "@/lib/api/session";
import { ThemeToggle } from "./theme/ThemeToggle";
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
  AlertDialogTrigger,
} from "./ui/alert-dialog";
import {
  Breadcrumb,
  BreadcrumbItem,
  BreadcrumbLink,
  BreadcrumbList,
  BreadcrumbPage,
  BreadcrumbSeparator,
} from "./ui/breadcrumb";
import { Button } from "./ui/button";
import { ChatBubbleAction } from "./ui/chat/chat-bubble";
import { Input } from "./ui/input";
import { SidebarTrigger } from "./ui/sidebar";

const RouterBreadcrumbLink = createLink(BreadcrumbLink);

export default function Header() {
  const navigate = useNavigate();
  const matchRoute = useMatchRoute();
  const sessionRouteMatch = matchRoute({ to: "/app/session/$sessionId" });
  const { data } = useGetChatSession(
    sessionRouteMatch ? sessionRouteMatch.sessionId : "",
  );

  const { mutate: updateSession } = useUpdateChatSession();
  const [isEditingTitle, setIsEditingTitle] = useState(false);
  const onSubmitTitleChange: FormEventHandler<HTMLFormElement> = useCallback(
    (e) => {
      e.preventDefault();
      const newTitle = new FormData(e.currentTarget).get("title")?.toString();
      if (data && newTitle) {
        updateSession({
          sessionId: data.session.id,
          data: { title: newTitle },
        });
      }
      setIsEditingTitle(false);
    },
    [updateSession, data],
  );

  const { mutate: deleteSession } = useDeleteChatSession();
  const onDeleteSession = useCallback(() => {
    if (!data) return;
    deleteSession({ sessionId: data.session.id });
    navigate({ to: "/app" });
  }, [data, deleteSession, navigate]);

  return (
    <header className="flex h-14 md:h-16 shrink-0 items-center gap-2 border-b px-4">
      <SidebarTrigger className="-ml-1" />
      <Separator
        orientation="vertical"
        className="hidden md:block mr-2 data-[orientation=vertical]:h-4"
      />
      <Breadcrumb className="min-w-0">
        <BreadcrumbList>
          <BreadcrumbItem className="hidden md:block">
            <RouterBreadcrumbLink to="/app">Chats</RouterBreadcrumbLink>
          </BreadcrumbItem>
          {sessionRouteMatch &&
            (!isEditingTitle ? (
              <>
                <BreadcrumbSeparator className="hidden md:block" />
                <BreadcrumbItem className="overflow-hidden">
                  <BreadcrumbPage className="truncate">
                    {data?.session.title}
                  </BreadcrumbPage>
                  <Button
                    size="icon"
                    variant="ghost"
                    className="size-6"
                    onClick={() => setIsEditingTitle(true)}
                  >
                    <Edit2 className="size-4" />
                  </Button>
                  <DeleteSessionButton onDelete={onDeleteSession} />
                </BreadcrumbItem>
              </>
            ) : (
              <>
                <BreadcrumbSeparator className="hidden md:block" />
                <BreadcrumbItem>
                  <form onSubmit={onSubmitTitleChange}>
                    <Input
                      name="title"
                      autoFocus
                      className="text-foreground w-[200px]"
                      defaultValue={data?.session.title}
                    />
                  </form>
                  <Button
                    type="button"
                    size="icon"
                    variant="ghost"
                    onClick={() => setIsEditingTitle(false)}
                  >
                    <X className="size-4" />
                  </Button>
                </BreadcrumbItem>
              </>
            ))}
        </BreadcrumbList>
      </Breadcrumb>
      <ThemeToggle className="ml-auto" />
    </header>
  );
}

function DeleteSessionButton({ onDelete }: { onDelete: () => void }) {
  const onSubmit: FormEventHandler<HTMLFormElement> = async (event) => {
    event.preventDefault();
    onDelete();
  };

  return (
    <AlertDialog>
      <AlertDialogTrigger asChild>
        <ChatBubbleAction
          aria-label="Delete session"
          variant="ghost"
          className="size-6"
          icon={<Trash className="size-4" />}
        />
      </AlertDialogTrigger>
      <AlertDialogContent>
        <form onSubmit={onSubmit}>
          <AlertDialogHeader>
            <AlertDialogTitle>
              Are you sure you want to delete this session?
            </AlertDialogTitle>
            <AlertDialogDescription>
              This action cannot be undone.
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>Cancel</AlertDialogCancel>
            <AlertDialogAction variant="destructive" type="submit">
              Yes, delete
            </AlertDialogAction>
          </AlertDialogFooter>
        </form>
      </AlertDialogContent>
    </AlertDialog>
  );
}
