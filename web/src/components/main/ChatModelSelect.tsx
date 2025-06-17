import * as React from "react";
import { Check, ChevronsUpDown } from "lucide-react";

import { cn } from "@/lib/utils";
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
import type { ProviderKey } from "./ChatMessageInput";

const providers: Array<{
  value: ProviderKey;
  label: string;
  models: string[];
}> = [
  {
    value: "Anthropic",
    label: "Anthropic",
    models: [
      "claude-opus-4-0",
      "claude-sonnet-4-0",
      "claude-3-7-sonnet-latest",
      "claude-3-5-haiku-latest",
      "claude-3-opus-latest",
    ],
  },
  {
    value: "OpenAI",
    label: "OpenAI",
    models: [
      "gpt-4.1",
      "gpt-4.1-nano",
      "gpt-4o",
      "o4-mini",
      "o3",
      "o3-pro",
      "o3-mini",
    ],
  },
  {
    value: "OpenRouter",
    label: "OpenRouter",
    models: [
      "openrouter/auto",
      "openai/gpt-4.1",
      "openai/o4-mini",
      "openai/o3-pro",
      "openai/o3-mini",
      "anthropic/claude-3.7-sonnet",
      "anthropic/claude-opus-4",
      "perplexity/sonar",
      "google/gemini-2.5-pro-preview",
      "google/gemini-2.0-flash-001",
    ],
  },
];

export function ChatModelSelect({
  onSelect,
  currentProviderKey,
  currentModel,
}: {
  currentProviderKey?: ProviderKey;
  currentModel: string;
  onSelect: (provider?: ProviderKey, model?: string) => void;
}) {
  const [open, setOpen] = React.useState(false);

  const currentProvider = React.useMemo(() => {
    return providers.find((p) => p.value === currentProviderKey);
  }, [currentProviderKey]);

  const setCurrentModel = (model: string) => {
    onSelect(currentProvider?.value, model);
  };

  return (
    <>
      <Popover open={open} onOpenChange={setOpen}>
        <PopoverTrigger asChild>
          <Button
            variant="outline"
            role="combobox"
            aria-expanded={open}
            className="w-[150px] justify-between"
          >
            {currentProvider
              ? providers.find((p) => p.value === currentProvider.value)?.label
              : "Select provider"}
            <ChevronsUpDown className="opacity-50" />
          </Button>
        </PopoverTrigger>
        <PopoverContent className="w-[200px] p-0">
          <Command>
            <CommandInput placeholder="Search providers..." className="h-9" />
            <CommandList>
              <CommandEmpty>No provider found.</CommandEmpty>
              <CommandGroup>
                {providers.map((provider) => (
                  <CommandItem
                    key={provider.value}
                    value={provider.value}
                    onSelect={() => {
                      const newProvider = providers.find(
                        (p) => p.value === provider.value,
                      );
                      onSelect(newProvider?.value, "");
                      setOpen(false);
                    }}
                  >
                    {provider.label}
                    <Check
                      className={cn(
                        "ml-auto",
                        currentProvider?.value === provider.value
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
      {currentProvider && (
        <ProviderModelSelect
          provider={currentProvider}
          currentModel={currentModel}
          onSelect={setCurrentModel}
        />
      )}
    </>
  );
}

function ProviderModelSelect({
  provider,
  currentModel,
  onSelect,
}: {
  provider: (typeof providers)[number];
  currentModel: string;
  onSelect: (model: string) => void;
}) {
  const [open, setOpen] = React.useState(false);

  return (
    <Popover open={open} onOpenChange={setOpen}>
      <PopoverTrigger asChild>
        <Button
          variant="outline"
          role="combobox"
          aria-expanded={open}
          className="w-[180px] justify-between"
        >
          <span className="overflow-hidden text-ellipsis">
            {currentModel
              ? provider.models.find((model) => model === currentModel)
              : "Select model"}
          </span>
          <ChevronsUpDown className="opacity-50" />
        </Button>
      </PopoverTrigger>
      <PopoverContent className="w-[200px] p-0">
        <Command>
          <CommandInput placeholder="Search models..." className="h-9" />
          <CommandList>
            <CommandEmpty>No models found.</CommandEmpty>
            <CommandGroup>
              {provider.models.map((model) => (
                <CommandItem
                  key={model}
                  value={model}
                  onSelect={() => {
                    onSelect(model);
                    setOpen(false);
                  }}
                >
                  {model}
                  <Check
                    className={cn(
                      "ml-auto",
                      currentModel === model ? "opacity-100" : "opacity-0",
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
