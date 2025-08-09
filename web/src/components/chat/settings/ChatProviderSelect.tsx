import { Check, ChevronsUpDown, KeyRound } from "lucide-react";
import React from "react";

import { Button } from "@/components/ui/button";
import {
  Command,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
} from "@/components/ui/command";
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from "@/components/ui/popover";
import type { components } from "@/lib/api/types";
import { cn } from "@/lib/utils";

interface Props {
  currentProvider?: components["schemas"]["ChatRsProvider"];
  onSelectModel: (providerId: number, modelId: string) => void;
  providers?: components["schemas"]["ChatRsProvider"][];
}

export default function ChatProviderSelect({
  currentProvider,
  onSelectModel,
  providers,
}: Props) {
  const [open, setOpen] = React.useState(false);

  return (
    <Popover open={open} onOpenChange={setOpen}>
      <PopoverTrigger asChild>
        <Button
          variant="outline"
          role="combobox"
          aria-expanded={open}
          className="w-[120px] md:w-[130px] justify-between"
        >
          <span className="truncate">
            {currentProvider ? currentProvider.name : "Select provider"}
          </span>
          <ChevronsUpDown className="opacity-50" />
        </Button>
      </PopoverTrigger>
      <PopoverContent className="w-[200px] p-0">
        <Command>
          <CommandInput placeholder="Search providers..." className="h-9" />
          <CommandList>
            <CommandEmpty>No provider found.</CommandEmpty>
            <CommandGroup>
              {providers?.map((provider) => (
                <CommandItem
                  key={provider.id}
                  value={String(provider.id)}
                  onSelect={() => {
                    onSelectModel(provider.id, "");
                    setOpen(false);
                  }}
                >
                  {provider.api_key_id && <KeyRound />}
                  {provider.name}
                  <Check
                    className={cn(
                      "ml-auto",
                      currentProvider?.id === provider.id
                        ? "opacity-100"
                        : "opacity-0",
                    )}
                  />
                </CommandItem>
              ))}
            </CommandGroup>
          </CommandList>
        </Command>
      </PopoverContent>
    </Popover>
  );
}
