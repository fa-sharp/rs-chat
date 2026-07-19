import { ChevronsUpDown, Eye, FileText, Wrench } from "lucide-react";
import React from "react";

import PopoverDrawer from "@/components/PopoverDrawer";
import { Button } from "@/components/ui/button";
import {
  Command,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
} from "@/components/ui/command";
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
    <PopoverDrawer
      open={open}
      onOpenChange={setOpen}
      popoverProps={{ className: "w-[250px] md:w-[300px] p-0" }}
      trigger={
        <Button
          variant="outline"
          role="combobox"
          aria-expanded={open}
          className="w-45 md:w-60 justify-between"
        >
          <span className="truncate">
            {currentModelId
              ? models?.find((model) => model.id === currentModelId)?.name ||
                currentModelId
              : "Select model"}
          </span>
          <ChevronsUpDown className="opacity-50" />
        </Button>
      }
    >
      <Command>
        <CommandInput placeholder="Search models..." className="h-9" />
        <CommandList>
          <CommandEmpty>No models found.</CommandEmpty>
          <CommandGroup>
            {models
              ?.toSorted((a, _) => (a.id === currentModelId ? -1 : 0))
              .map((model) => (
                <CommandItem
                  key={model.id}
                  value={`${model.id} ${model.name}`}
                  onSelect={() => {
                    onSelect(model.id);
                    setOpen(false);
                  }}
                >
                  <div
                    className={cn(
                      "flex flex-col",
                      model.id === currentModelId && "font-bold",
                    )}
                  >
                    {model.name}
                    <span className="text-muted-foreground text-xs">
                      {model.id}
                    </span>
                  </div>
                  <div className="flex gap-2 ml-auto">
                    {model.tool_call && <Wrench />}
                    {model.modalities?.input.includes("image") && <Eye />}
                    {model.modalities?.input.includes("pdf") && <FileText />}
                  </div>
                </CommandItem>
              ))}
          </CommandGroup>
        </CommandList>
      </Command>
    </PopoverDrawer>
  );
}
