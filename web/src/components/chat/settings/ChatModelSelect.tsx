import { Check, ChevronsUpDown } from "lucide-react";
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
import { useProviderModels } from "@/lib/api/provider";
import { cn } from "@/lib/utils";

export default function ChatModelSelect({
  providerId,
  currentModelId,
  onSelect,
}: {
  providerId?: number | null;
  currentModelId: string;
  onSelect: (model: string) => void;
}) {
  const { data: models } = useProviderModels(providerId);

  const [open, setOpen] = React.useState(false);

  return (
    <Popover open={open} onOpenChange={setOpen}>
      <PopoverTrigger asChild>
        <Button
          variant="outline"
          role="combobox"
          aria-expanded={open}
          className="w-[180px] md:w-[200px] justify-between"
        >
          <span className="truncate">
            {currentModelId
              ? models?.find((model) => model.id === currentModelId)?.name ||
                currentModelId
              : "Select model"}
          </span>
          <ChevronsUpDown className="opacity-50" />
        </Button>
      </PopoverTrigger>
      <PopoverContent className="w-[250px] p-0">
        <Command>
          <CommandInput placeholder="Search models..." className="h-9" />
          <CommandList>
            <CommandEmpty>No models found.</CommandEmpty>
            <CommandGroup>
              {models?.map((model) => (
                <CommandItem
                  key={model.id}
                  value={model.id}
                  onSelect={() => {
                    onSelect(model.id);
                    setOpen(false);
                  }}
                >
                  {model.name}
                  <Check
                    className={cn(
                      "ml-auto",
                      currentModelId === model.id ? "opacity-100" : "opacity-0",
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
