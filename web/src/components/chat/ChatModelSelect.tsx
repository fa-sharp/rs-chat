import { Check, ChevronsUpDown, KeyRound, Settings } from "lucide-react";
import * as React from "react";

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
import { useProviderModels, useProviders } from "@/lib/api/provider";
import { useTools } from "@/lib/api/tool";
import { cn } from "@/lib/utils";
import { Label } from "../ui/label";
import {
  Select,
  SelectContent,
  SelectGroup,
  SelectItem,
  SelectLabel,
  SelectTrigger,
  SelectValue,
} from "../ui/select";
import ChatToolSelect from "./ChatToolSelect";

export function ChatModelSelect({
  currentProviderId,
  currentModel,
  onSelectModel,
  currentMaxTokens,
  onSelectMaxTokens,
  currentTemperature,
  onSelectTemperature,
  currentToolIds,
  onToggleTool,
}: {
  currentProviderId?: number | null;
  currentModel: string;
  onSelectModel: (providerId?: number | null, model?: string) => void;
  currentMaxTokens: number;
  onSelectMaxTokens: (maxTokens: number) => void;
  currentTemperature: number;
  onSelectTemperature: (temperature: number) => void;
  currentToolIds: string[];
  onToggleTool: (toolId: string) => void;
}) {
  const { data: providers } = useProviders();
  const { data: tools } = useTools();
  const currentProvider = React.useMemo(() => {
    return providers?.find((p) => p.id === currentProviderId);
  }, [providers, currentProviderId]);

  const setCurrentModel = (model: string) => {
    onSelectModel(currentProviderId, model);
  };

  const [open, setOpen] = React.useState(false);

  return (
    <>
      <Popover open={open} onOpenChange={setOpen}>
        <PopoverTrigger asChild>
          <Button
            variant="outline"
            role="combobox"
            aria-expanded={open}
            className="w-[140px] justify-between"
          >
            {currentProvider ? currentProvider.name : "Select provider"}
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
      {currentProvider && currentProvider.provider_type !== "lorem" && (
        <>
          <ProviderModelSelect
            providerId={currentProviderId}
            currentModelId={currentModel}
            onSelect={setCurrentModel}
          />

          <Popover>
            <PopoverTrigger asChild>
              <Button aria-label="More settings" size="icon" variant="outline">
                <Settings />
              </Button>
            </PopoverTrigger>
            <PopoverContent className="flex flex-col gap-2">
              <Label>
                Max tokens
                <Select
                  value={currentMaxTokens.toString()}
                  onValueChange={(tokens) => onSelectMaxTokens(+tokens)}
                >
                  <SelectTrigger className="w-[100px]">
                    <SelectValue placeholder="Max tokens" />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectGroup>
                      <SelectLabel>Max tokens</SelectLabel>
                      {[500, 1000, 2000, 5000, 10000, 20000, 50000].map(
                        (tokens) => (
                          <SelectItem key={tokens} value={tokens.toString()}>
                            {tokens.toLocaleString()}
                          </SelectItem>
                        ),
                      )}
                    </SelectGroup>
                  </SelectContent>
                </Select>
              </Label>
              <Label>
                Temperature
                <Select
                  value={currentTemperature.toFixed(1)}
                  onValueChange={(temperature) =>
                    onSelectTemperature(+temperature)
                  }
                >
                  <SelectTrigger className="w-[70px]">
                    <SelectValue placeholder="Temperature" />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectGroup>
                      <SelectLabel>Temperature</SelectLabel>
                      {[0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9].map(
                        (temperature) => (
                          <SelectItem
                            key={temperature}
                            value={temperature.toString()}
                          >
                            {temperature.toFixed(1)}
                          </SelectItem>
                        ),
                      )}
                    </SelectGroup>
                  </SelectContent>
                </Select>
              </Label>
            </PopoverContent>
          </Popover>
          <ChatToolSelect
            tools={tools}
            selectedToolIds={currentToolIds}
            toggleTool={onToggleTool}
          />
        </>
      )}
    </>
  );
}

function ProviderModelSelect({
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
          className="w-[280px] justify-between"
        >
          <span className="overflow-hidden text-ellipsis">
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
