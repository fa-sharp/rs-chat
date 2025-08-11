import { Check, Wrench } from "lucide-react";
import { useMemo, useState } from "react";

import PopoverDrawer from "@/components/PopoverDrawer";
import { getToolIcon } from "@/components/ToolsManager";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
  Command,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
} from "@/components/ui/command";
import type { components } from "@/lib/api/types";

export default function ChatToolSelect({
  selectedToolIds,
  toggleTool,
  tools,
}: {
  selectedToolIds: string[];
  toggleTool: (toolId: string) => void;
  tools?: components["schemas"]["ChatRsToolPublic"][];
}) {
  const [open, setOpen] = useState(false);
  const selectedTools = useMemo(
    () => tools?.filter((tool) => selectedToolIds.includes(tool.id)),
    [tools, selectedToolIds],
  );

  return (
    <PopoverDrawer
      open={open}
      onOpenChange={setOpen}
      popoverProps={{ className: "w-[240px] p-0" }}
      trigger={
        <Button aria-label="Tools" size="icon" variant="outline">
          {selectedTools && selectedTools.length > 0 && (
            <Badge className="absolute top-[-4px] right-[-4px] h-4 min-w-4 rounded-full px-1 font-mono tabular-nums">
              {selectedTools.length}
            </Badge>
          )}
          <Wrench />
        </Button>
      }
    >
      <Command>
        <CommandInput placeholder="Search tools..." className="h-9" />
        <CommandList>
          <CommandEmpty>No tools found.</CommandEmpty>
          <CommandGroup>
            {tools?.map((tool) => (
              <CommandItem
                key={tool.id}
                value={tool.name}
                aria-checked={selectedToolIds.includes(tool.id)}
                onSelect={() => toggleTool(tool.id)}
              >
                {getToolIcon(tool)}
                {tool.name}
                {selectedToolIds.includes(tool.id) && (
                  <Check className="ml-auto" />
                )}
              </CommandItem>
            ))}
          </CommandGroup>
        </CommandList>
      </Command>
    </PopoverDrawer>
  );
}
