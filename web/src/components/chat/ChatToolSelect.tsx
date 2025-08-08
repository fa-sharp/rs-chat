import { Check, Wrench } from "lucide-react";
import { useState } from "react";

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
import { getToolIcon } from "../ToolsManager";

export default function ChatToolSelect({
  selectedToolIds,
  toggleTool,
  tools,
}: {
  selectedToolIds: string[];
  toggleTool: (toolId: string) => void;
  tools?: components["schemas"]["ChatRsTool"][];
}) {
  const [open, setOpen] = useState(false);

  return (
    <Popover open={open} onOpenChange={setOpen}>
      <PopoverTrigger asChild>
        <Button aria-label="Tools" size="icon" variant="outline">
          <Wrench />
        </Button>
      </PopoverTrigger>
      <PopoverContent className="w-[200px] p-0">
        <Command>
          <CommandInput placeholder="Search tools..." className="h-9" />
          <CommandList>
            <CommandEmpty>No tools found.</CommandEmpty>
            <CommandGroup>
              {tools?.map((tool) => (
                <CommandItem
                  key={tool.id}
                  value={tool.name}
                  onSelect={() => toggleTool(tool.id)}
                >
                  {getToolIcon(tool)}
                  {tool.name}
                  <Check
                    className={cn(
                      "ml-auto",
                      selectedToolIds.includes(tool.id)
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
