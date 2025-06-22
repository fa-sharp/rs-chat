import { CommandLoading } from "cmdk";
import { useEffect, useState } from "react";
import { useDebounce } from "use-debounce";

import { useSearchChats } from "@/lib/api/search";
import {
  CommandDialog,
  CommandEmpty,
  CommandInput,
  CommandItem,
  CommandList,
} from "./ui/command";

export default function SearchDialog() {
  const [open, setOpen] = useState(false);
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
            className="flex flex-col items-start gap-0.5"
            value={result.session_id}
          >
            <div>{result.title_highlight}</div>
            <div className="text-xs text-muted-foreground">
              {result.message_highlights}
            </div>
          </CommandItem>
        ))}
      </CommandList>
    </CommandDialog>
  );
}
