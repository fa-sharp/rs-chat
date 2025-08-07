import { Bot, Plus, Trash2 } from "lucide-react";
import { type FormEventHandler, useState } from "react";

import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
  AlertDialogTrigger,
} from "@/components/ui/alert-dialog";
import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
  useCreateProvider,
  useDeleteProvider,
  useProviders,
} from "@/lib/api/provider";
import type { components } from "@/lib/api/types";
import { cn } from "@/lib/utils";

type AIProvider = "anthropic" | "openai" | "openrouter" | "lorem";

interface ProviderInfo {
  name: string;
  description: string;
  apiType: components["schemas"]["ChatRsProvider"]["provider_type"];
  baseUrl?: string;
  keyFormat?: string;
  color: string;
  defaultModel: string;
}

const PROVIDERS: Record<AIProvider, ProviderInfo> = {
  openai: {
    name: "OpenAI",
    description: "GPT-4, GPT-3.5, and other OpenAI models",
    apiType: "openai",
    keyFormat: "sk-...",
    color:
      "bg-green-100 dark:bg-green-900 border-green-300 dark:border-green-700",
    defaultModel: "gpt-4o-mini",
  },
  anthropic: {
    name: "Anthropic",
    description: "Claude Sonnet, Opus, and other Anthropic models",
    apiType: "anthropic",
    keyFormat: "sk-ant-...",
    color:
      "bg-orange-100 dark:bg-orange-900 border-orange-300 dark:border-orange-700",
    defaultModel: "claude-3-7-sonnet-latest",
  },
  openrouter: {
    name: "OpenRouter",
    description: "Access multiple AI models via OpenRouter",
    apiType: "openai",
    baseUrl: "https://openrouter.ai/api/v1",
    keyFormat: "sk-or-...",
    color: "bg-blue-100 dark:bg-blue-900 border-blue-300 dark:border-blue-700",
    defaultModel: "openai/gpt-4o-mini",
  },
  lorem: {
    name: "Lorem Ipsum",
    description: "Generate placeholder text for testing",
    apiType: "lorem",
    color:
      "bg-purple-100 dark:bg-purple-900 border-purple-300 dark:border-purple-700",
    defaultModel: "lorem-ipsum",
  },
};

const availableProviders = Object.keys(PROVIDERS) as AIProvider[];

export function ProviderManager({
  className,
  ...props
}: React.ComponentProps<"div">) {
  const { data: providers } = useProviders();
  const createProvider = useCreateProvider();
  const deleteProvider = useDeleteProvider();

  const [isCreateDialogOpen, setIsCreateDialogOpen] = useState(false);
  const [selectedProvider, setSelectedProvider] = useState<AIProvider | null>(
    null,
  );
  const [name, setName] = useState("");
  const [newApiKey, setNewApiKey] = useState("");

  const handleCreateKey: FormEventHandler<HTMLFormElement> = (event) => {
    event.preventDefault();
    if (
      !selectedProvider ||
      (PROVIDERS[selectedProvider].apiType !== "lorem" && !newApiKey.trim())
    )
      return;

    createProvider.mutate(
      {
        name,
        type: PROVIDERS[selectedProvider].apiType,
        base_url: PROVIDERS[selectedProvider].baseUrl,
        default_model: PROVIDERS[selectedProvider].defaultModel,
        api_key: newApiKey || null,
      },
      {
        onSettled: () => {
          setSelectedProvider(null);
          setIsCreateDialogOpen(false);
          setNewApiKey("");
          setName("");
        },
      },
    );
  };

  const handleDeleteKey = (id: number) => {
    deleteProvider.mutate(id);
  };

  const formatDate = (dateString: string) => {
    return new Date(dateString).toLocaleDateString("en-US", {
      year: "numeric",
      month: "short",
      day: "numeric",
      hour: "2-digit",
      minute: "2-digit",
    });
  };

  return (
    <div
      className={cn("flex flex-col gap-6 max-w-4xl mx-auto", className)}
      {...props}
    >
      <div className="flex flex-col gap-2">
        <div className="flex items-center gap-2">
          <h1 className="text-3xl font-bold">Providers</h1>
        </div>
        <p className="text-muted-foreground">
          Manage your configuration and API keys for different AI providers.
        </p>
      </div>

      {/* Add Provider Button */}
      <div className="flex justify-between items-center">
        <div className="text-sm text-muted-foreground">
          {providers?.length} providers configured
        </div>
        <Dialog open={isCreateDialogOpen} onOpenChange={setIsCreateDialogOpen}>
          <DialogTrigger asChild>
            <Button>
              <Plus className="size-4" />
              Add Provider
            </Button>
          </DialogTrigger>
          <DialogContent>
            <DialogHeader>
              <DialogTitle>Add AI Provider</DialogTitle>
              <DialogDescription>Select a provider to add.</DialogDescription>
            </DialogHeader>
            <form onSubmit={handleCreateKey}>
              <div className="grid gap-4 py-4">
                <div className="grid gap-2">
                  <Label>Provider</Label>
                  <div className="grid gap-2">
                    {availableProviders.map((provider) => (
                      <button
                        key={provider}
                        type="button"
                        className={cn(
                          "p-3 border rounded-lg cursor-pointer transition-colors",
                          selectedProvider === provider
                            ? "border-primary bg-primary/5"
                            : "border-border hover:bg-muted/50",
                        )}
                        onClick={() => setSelectedProvider(provider)}
                      >
                        <div className="flex items-center justify-between">
                          <div className="flex flex-col items-start">
                            <div className="font-medium">
                              {PROVIDERS[provider].name}
                            </div>
                            <div className="text-sm text-muted-foreground">
                              {PROVIDERS[provider].description}
                            </div>
                          </div>
                          <div className="text-xs text-muted-foreground font-mono">
                            {PROVIDERS[provider].keyFormat}
                          </div>
                        </div>
                      </button>
                    ))}
                  </div>
                </div>
                <div className="grid gap-2">
                  <Label htmlFor="name">Name</Label>
                  <Input
                    required
                    id="name"
                    type="text"
                    placeholder={
                      selectedProvider ? PROVIDERS[selectedProvider].name : ""
                    }
                    value={name}
                    onChange={(e) => setName(e.target.value)}
                  />
                </div>
                {selectedProvider && PROVIDERS[selectedProvider].keyFormat && (
                  <div className="grid gap-2">
                    <Label htmlFor="api-key">API Key</Label>
                    <Input
                      required
                      id="api-key"
                      type="password"
                      placeholder={`Enter your ${PROVIDERS[selectedProvider].name} API key`}
                      value={newApiKey}
                      onChange={(e) => setNewApiKey(e.target.value)}
                    />
                  </div>
                )}
              </div>
              <DialogFooter>
                <Button
                  type="reset"
                  variant="outline"
                  onClick={() => {
                    setIsCreateDialogOpen(false);
                    setSelectedProvider(null);
                    setNewApiKey("");
                  }}
                >
                  Cancel
                </Button>
                <Button
                  type="submit"
                  disabled={!selectedProvider || createProvider.isPending}
                >
                  {createProvider.isPending ? "Adding..." : "Add Provider"}
                </Button>
              </DialogFooter>
            </form>
          </DialogContent>
        </Dialog>
      </div>

      {/* Configured Providers List */}
      <div className="grid gap-4">
        {providers?.length === 0 ? (
          <Card>
            <CardContent className="pt-6">
              <div className="text-center py-8">
                <Bot className="size-12 mx-auto text-muted-foreground mb-4" />
                <h3 className="text-lg font-semibold mb-2">
                  No providers configured yet
                </h3>
              </div>
            </CardContent>
          </Card>
        ) : (
          providers?.map((provider) => {
            return (
              <Card
                key={provider.id}
                className={cn(
                  "border-2",
                  PROVIDERS[provider.provider_type].color,
                )}
              >
                <CardHeader>
                  <div className="flex items-center justify-between">
                    <div>
                      <CardTitle className="text-lg flex items-center gap-2">
                        <Bot className="size-5" />
                        {provider.name}
                      </CardTitle>
                      <CardDescription>
                        <p>Type: {PROVIDERS[provider.provider_type].name}</p>
                        {provider.base_url && (
                          <p>Base URL: {provider.base_url}</p>
                        )}
                        <p>Added: {formatDate(provider.created_at)}</p>
                      </CardDescription>
                    </div>
                    <AlertDialog>
                      <AlertDialogTrigger asChild>
                        <Button variant="outline" size="sm">
                          <Trash2 className="size-4 text-destructive-foreground" />
                          Delete
                        </Button>
                      </AlertDialogTrigger>
                      <AlertDialogContent>
                        <AlertDialogHeader>
                          <AlertDialogTitle>Delete Provider</AlertDialogTitle>
                          <AlertDialogDescription>
                            Are you sure you want to delete the {provider.name}{" "}
                            provider? This will remove any associated API key as
                            well.
                          </AlertDialogDescription>
                        </AlertDialogHeader>
                        <AlertDialogFooter>
                          <AlertDialogCancel>Cancel</AlertDialogCancel>
                          <AlertDialogAction
                            onClick={() => handleDeleteKey(provider.id)}
                            className="bg-red-600 hover:bg-red-700 dark:bg-red-400 dark:hover:bg-red-300"
                          >
                            Delete Provider
                          </AlertDialogAction>
                        </AlertDialogFooter>
                      </AlertDialogContent>
                    </AlertDialog>
                  </div>
                </CardHeader>
                <CardContent>
                  {provider.api_key_id && (
                    <div className="flex items-center gap-2">
                      <div className="flex-1 font-mono text-sm bg-muted px-3 py-2 rounded border">
                        *****************
                      </div>
                    </div>
                  )}
                </CardContent>
              </Card>
            );
          })
        )}
      </div>
    </div>
  );
}
