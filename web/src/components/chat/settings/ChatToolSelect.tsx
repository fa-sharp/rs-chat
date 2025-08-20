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
import type { useChatInputState } from "@/hooks/useChatInputState";
import type { components } from "@/lib/api/types";

export default function ChatToolSelect({
  toolInput,
  onSetSystemTool,
  onToggleExternalApiTool,
  tools,
}: {
  toolInput: components["schemas"]["SendChatToolInput"] | null;
  onSetSystemTool: ReturnType<typeof useChatInputState>["onSetSystemTool"];
  onToggleExternalApiTool: ReturnType<
    typeof useChatInputState
  >["onToggleExternalApiTool"];
  tools?: components["schemas"]["GetAllToolsResponse"];
}) {
  const [open, setOpen] = useState(false);
  const codeRunnerTool = useMemo(
    () => tools?.system.find((tool) => tool.data.type === "code_runner"),
    [tools?.system],
  );
  const systemInfoTool = useMemo(
    () => tools?.system.find((tool) => tool.data.type === "system_info"),
    [tools?.system],
  );
  const externalApiTools = useMemo(
    () =>
      tools?.external_api.map((tool) => {
        const icon = getToolIcon(tool);
        switch (tool.data.type) {
          case "web_search":
            return {
              id: tool.id,
              name: `Web Search (${tool.data.config.provider.type})`,
              icon,
            };
          case "custom_api":
            return {
              id: tool.id,
              name: `Custom API: ${tool.data.config.name}`,
              icon,
            };
          default:
            return {
              id: tool.id,
              name: "Unknown Tool",
              icon,
            };
        }
      }),
    [tools?.external_api],
  );

  const numToolsSelected = useMemo(() => {
    let numTools =
      tools?.external_api.filter((tool) =>
        toolInput?.external_apis?.some((t) => t.id === tool.id),
      ).length ?? 0;
    if (codeRunnerTool && toolInput?.system?.code_runner) {
      numTools += 1;
    }
    if (systemInfoTool && toolInput?.system?.info) {
      numTools += 1;
    }
    return numTools;
  }, [tools, toolInput, codeRunnerTool, systemInfoTool]);

  return (
    <PopoverDrawer
      open={open}
      onOpenChange={setOpen}
      popoverProps={{ className: "w-[240px] p-0" }}
      trigger={
        <Button aria-label="Tools" size="icon" variant="outline">
          {numToolsSelected > 0 && (
            <Badge className="absolute top-[-4px] right-[-4px] h-4 min-w-4 rounded-full px-1 font-mono tabular-nums">
              {numToolsSelected}
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
            {codeRunnerTool && (
              <CommandItem
                key={codeRunnerTool.id}
                value="code runner"
                aria-checked={!!toolInput?.system?.code_runner}
                onSelect={() =>
                  onSetSystemTool(
                    "code_runner",
                    !toolInput?.system?.code_runner,
                  )
                }
              >
                {getToolIcon(codeRunnerTool)}
                Code Runner
                {!!toolInput?.system?.code_runner && (
                  <Check className="ml-auto" />
                )}
              </CommandItem>
            )}
            {systemInfoTool && (
              <CommandItem
                key={systemInfoTool.id}
                value="system info"
                aria-checked={!!toolInput?.system?.info}
                onSelect={() =>
                  onSetSystemTool("info", !toolInput?.system?.info)
                }
              >
                {getToolIcon(systemInfoTool)}
                Time & Date / System
                {!!toolInput?.system?.info && <Check className="ml-auto" />}
              </CommandItem>
            )}
            {externalApiTools?.map((tool) => (
              <CommandItem
                key={tool.id}
                value={tool.name}
                aria-checked={toolInput?.external_apis?.some(
                  (t) => t.id === tool.id,
                )}
                onSelect={() => onToggleExternalApiTool({ id: tool.id })}
              >
                {tool.icon}
                {tool.name}
                {toolInput?.external_apis?.some((t) => t.id === tool.id) && (
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
