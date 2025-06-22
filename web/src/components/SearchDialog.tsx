import { useNavigate } from "@tanstack/react-router";
import { CommandLoading } from "cmdk";
import { Fragment, useCallback, useEffect, useState } from "react";
import { useDebounce } from "use-debounce";

import { useSearchChats } from "@/lib/api/search";
import type { components } from "@/lib/api/types";
import {
  CommandDialog,
  CommandEmpty,
  CommandInput,
  CommandItem,
  CommandList,
} from "./ui/command";

export default function SearchDialog() {
  const [open, setOpen] = useState(false);

  const navigate = useNavigate();
  const onSelectSession = useCallback(
    (sessionId: string) => {
      navigate({ to: "/app/session/$sessionId", params: { sessionId } });
      setOpen(false);
    },
    [navigate],
  );

  const [searchQuery, setSearchQuery] = useState("");
  const [debouncedQuery, debounceMeta] = useDebounce(searchQuery, 300, {
    trailing: true,
  });

  const { data, isFetching } = useSearchChats(debouncedQuery);

  // Handle command-k triggered
  useEffect(() => {
    const down = (e: KeyboardEvent) => {
      if (e.key === "k" && (e.metaKey || e.ctrlKey)) {
        e.preventDefault();
        setOpen((open) => !open);
      }
    };
    document.addEventListener("keydown", down);
    return () => document.removeEventListener("keydown", down);
  }, []);

  return (
    <CommandDialog open={open} onOpenChange={setOpen} shouldFilter={false}>
      <CommandInput
        placeholder="Search chats..."
        onValueChange={setSearchQuery}
      />
      <CommandList>
        {(isFetching || debounceMeta.isPending()) && (
          <CommandLoading className="p-3" progress={50}>
            <div className="flex items-center gap-2 text-sm text-muted-foreground">
              <div className="animate-spin rounded-full h-4 w-4 border-b-2 border-muted-foreground"></div>
              Searching for '{searchQuery}'...
            </div>
          </CommandLoading>
        )}
        {searchQuery && !(isFetching || debounceMeta.isPending()) && (
          <CommandEmpty>No results</CommandEmpty>
        )}
        {data?.map((result) => (
          <CommandItem
            key={result.session_id}
            value={result.session_id}
            onSelect={onSelectSession}
          >
            <HighlightedResult result={result} />
          </CommandItem>
        ))}
      </CommandList>
    </CommandDialog>
  );
}

const HIGHLIGHT_REGEX = /§§§HIGHLIGHT_START§§§|§§§HIGHLIGHT_END§§§/g;

function HighlightedResult({
  result,
}: {
  result: components["schemas"]["SessionSearchResult"];
}) {
  const titleParts = result.title_highlight.split(HIGHLIGHT_REGEX);
  const messageParts = result.message_highlights.split(HIGHLIGHT_REGEX);

  const highlightedTitle = titleParts.map((part, index) =>
    index % 2 === 1 ? (
      // biome-ignore lint/suspicious/noArrayIndexKey: no alternative key
      <span key={index} className="font-bold">
        {part}
      </span>
    ) : (
      // biome-ignore lint/suspicious/noArrayIndexKey: no alternative key
      <Fragment key={index}>{part}</Fragment>
    ),
  );

  const highlightedMessage = messageParts.map((part, index) =>
    index % 2 === 1 ? (
      // biome-ignore lint/suspicious/noArrayIndexKey: no alternative key
      <span key={index} className="font-bold">
        {part}
      </span>
    ) : (
      // biome-ignore lint/suspicious/noArrayIndexKey: no alternative key
      <Fragment key={index}>{part}</Fragment>
    ),
  );

  return (
    <div className="flex flex-col">
      <span>{highlightedTitle}</span>
      <span className="text-xs text-muted-foreground">
        {highlightedMessage}
      </span>
    </div>
  );
}
