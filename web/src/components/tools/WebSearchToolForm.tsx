import { Globe } from "lucide-react";
import { useId, useState } from "react";

import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { useCreateTool } from "@/lib/api/tool";
import type { components } from "@/lib/api/types";

const WEB_SEARCH_PROVIDERS = [{ value: "exa", label: "exa.ai" }] as const;

interface WebSearchToolFormProps {
  onSuccess?: () => void;
  onCancel?: () => void;
}

export function WebSearchToolForm({
  onSuccess,
  onCancel,
}: WebSearchToolFormProps) {
  const createTool = useCreateTool();

  const [provider, setProvider] = useState<string>("");
  const [apiKey, setApiKey] = useState("");
  const [count, setCount] = useState<number>(10);
  const [maxCharacters, setMaxCharacters] = useState<number>(5000);
  const [error, setError] = useState<string>("");
  const [isSubmitting, setIsSubmitting] = useState(false);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError("");
    setIsSubmitting(true);

    if (!provider) {
      setError("Please select a provider");
      setIsSubmitting(false);
      return;
    }

    if (!apiKey.trim()) {
      setError("Please enter an API key");
      setIsSubmitting(false);
      return;
    }

    try {
      const config: components["schemas"]["WebSearchConfig"] = {
        provider: { type: provider as "exa" },
        count,
        max_characters: maxCharacters,
      };

      const toolInput: components["schemas"]["CreateToolInput"] = {
        external_api: {
          type: "web_search",
          config,
          secret_1: {
            key: apiKey,
            name: "API Key",
          },
        },
      };

      await createTool.mutateAsync(toolInput);
      onSuccess?.();
    } catch (error) {
      console.error("Failed to create web search tool:", error);
      setError("Failed to create web search tool. Please try again.");
    } finally {
      setIsSubmitting(false);
    }
  };

  const handleReset = () => {
    setProvider("");
    setApiKey("");
    setCount(10);
    setMaxCharacters(5000);
    setError("");
  };

  const apiKeyId = useId();
  const maxResultsId = useId();
  const maxCharsId = useId();

  return (
    <form onSubmit={handleSubmit} className="space-y-4">
      <div className="flex items-center gap-2 mb-4">
        <Globe className="size-5" />
        <h3 className="text-lg font-medium">Web Search Tool</h3>
      </div>

      <div className="space-y-3">
        <div>
          <Label htmlFor="provider" className="mb-2">
            Search Provider
          </Label>
          <Select value={provider} onValueChange={setProvider}>
            <SelectTrigger>
              <SelectValue placeholder="Select a search provider" />
            </SelectTrigger>
            <SelectContent>
              {WEB_SEARCH_PROVIDERS.map((provider) => (
                <SelectItem key={provider.value} value={provider.value}>
                  {provider.label}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>

        <div>
          <Label htmlFor={apiKeyId} className="mb-2">
            API Key
          </Label>
          <Input
            id={apiKeyId}
            type="password"
            value={apiKey}
            onChange={(e) => setApiKey(e.target.value)}
            placeholder="Enter your API key"
          />
        </div>

        <div>
          <Label htmlFor={maxResultsId} className="mb-2">
            Max Results (1-10)
          </Label>
          <Input
            id={maxResultsId}
            type="number"
            min={1}
            max={10}
            value={count}
            onChange={(e) => setCount(Number(e.target.value))}
          />
        </div>

        <div>
          <Label htmlFor={maxCharsId} className="mb-2">
            Max Characters for Content Extraction
          </Label>
          <Input
            id={maxCharsId}
            type="number"
            min={500}
            max={10_000}
            value={maxCharacters}
            onChange={(e) => setMaxCharacters(Number(e.target.value))}
          />
        </div>
      </div>

      {error && (
        <div className="text-sm text-red-600 bg-red-50 p-3 rounded-md">
          {error}
        </div>
      )}

      <div className="flex gap-2 pt-4">
        <Button type="submit" disabled={isSubmitting}>
          {isSubmitting ? "Creating..." : "Create Web Search Tool"}
        </Button>
        <Button type="button" variant="outline" onClick={handleReset}>
          Reset
        </Button>
        {onCancel && (
          <Button type="button" variant="ghost" onClick={onCancel}>
            Cancel
          </Button>
        )}
      </div>
    </form>
  );
}
