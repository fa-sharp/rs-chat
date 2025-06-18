import * as React from "react";
import { Check, ChevronsUpDown, Lock } from "lucide-react";

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
import { useApiKeys } from "@/lib/api/apiKey";
import { providers, type ProviderKey } from "@/lib/providerInfo";
import {
  Select,
  SelectContent,
  SelectGroup,
  SelectItem,
  SelectLabel,
  SelectTrigger,
  SelectValue,
} from "../ui/select";

export function ChatModelSelect({
  onSelect,
  currentProviderKey,
  currentModel,
  currentMaxTokens,
  onSelectMaxTokens,
  currentTemperature,
  onSelectTemperature,
}: {
  currentProviderKey?: ProviderKey;
  currentModel: string;
  onSelect: (provider?: ProviderKey, model?: string) => void;
  currentMaxTokens: number;
  onSelectMaxTokens: (maxTokens: number) => void;
  currentTemperature: number;
  onSelectTemperature: (temperature: number) => void;
}) {
  const { data: apiKeys } = useApiKeys();

  const currentProvider = React.useMemo(() => {
    return providers.find((p) => p.value === currentProviderKey);
  }, [currentProviderKey]);

  const setCurrentModel = (model: string) => {
    onSelect(currentProvider?.value, model);
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
                {providers.map((provider) => {
                  const missingApiKey =
                    provider.apiKeyType &&
                    !apiKeys?.find(
                      (key) => key.provider === provider.apiKeyType,
                    );

                  return (
                    <CommandItem
                      key={provider.value}
                      disabled={missingApiKey}
                      value={provider.value}
                      onSelect={() => {
                        const newProvider = providers.find(
                          (p) => p.value === provider.value,
                        );
                        onSelect(newProvider?.value, "");
                        setOpen(false);
                      }}
                    >
                      {missingApiKey && <Lock />}
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
                  );
                })}
              </CommandGroup>
            </CommandList>
          </Command>
        </PopoverContent>
      </Popover>
      {currentProvider && currentProvider.value !== "Lorem" && (
        <>
          <ProviderModelSelect
            provider={currentProvider}
            currentModel={currentModel}
            onSelect={setCurrentModel}
          />
          <Select
            value={currentMaxTokens.toString()}
            onValueChange={(tokens) => onSelectMaxTokens(+tokens)}
          >
            <SelectTrigger className="w-[150px]">
              <SelectValue placeholder="Max tokens" />
            </SelectTrigger>
            <SelectContent>
              <SelectGroup>
                <SelectLabel>Max tokens</SelectLabel>
                {[500, 1000, 2000, 5000, 10000, 20000, 50000].map((tokens) => (
                  <SelectItem key={tokens} value={tokens.toString()}>
                    {tokens.toLocaleString()} tokens
                  </SelectItem>
                ))}
              </SelectGroup>
            </SelectContent>
          </Select>
          <Select
            value={currentTemperature.toFixed(1)}
            onValueChange={(temperature) => onSelectTemperature(+temperature)}
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
        </>
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
