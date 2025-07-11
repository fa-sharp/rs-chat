import { Bot, Plus, Trash2 } from "lucide-react";
import { useState } from "react";

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
  useCreateProviderKey,
  useDeleteProviderKey,
  useProviderKeys,
} from "@/lib/api/providerKey";
import type { components } from "@/lib/api/types";
import { cn } from "@/lib/utils";

type AIProvider = components["schemas"]["ChatRsProviderKeyType"];

interface ApiKey {
  provider: AIProvider;
  createdAt: string;
  lastUsed?: string;
}

interface ProviderInfo {
  name: string;
  description: string;
  keyFormat: string;
  color: string;
}

//@ts-expect-error not all providers are supported yet
const PROVIDERS: Record<ApiKey["provider"], ProviderInfo> = {
  // Openai: {
  //   name: "OpenAI",
  //   description: "GPT-4, GPT-3.5, and other OpenAI models",
  //   keyFormat: "sk-...",
  //   color:
  //     "bg-green-100 dark:bg-green-900 border-green-300 dark:border-green-700",
  // },
  Anthropic: {
    name: "Anthropic",
    description: "Claude Sonnet, Opus, and other Anthropic models",
    keyFormat: "sk-ant-...",
    color:
      "bg-orange-100 dark:bg-orange-900 border-orange-300 dark:border-orange-700",
  },
  Openrouter: {
    name: "OpenRouter",
    description: "Access multiple AI models via OpenRouter",
    keyFormat: "sk-or-...",
    color: "bg-blue-100 dark:bg-blue-900 border-blue-300 dark:border-blue-700",
  },
};

const availableProviders = Object.keys(PROVIDERS) as AIProvider[];

export function ProviderKeysManager({
  className,
  ...props
}: React.ComponentProps<"div">) {
  const { data: apiKeys } = useProviderKeys();
  const createKey = useCreateProviderKey();
  const deleteKey = useDeleteProviderKey();

  const [isCreateDialogOpen, setIsCreateDialogOpen] = useState(false);
  const [selectedProvider, setSelectedProvider] = useState<AIProvider | null>(
    null,
  );
  const [newApiKey, setNewApiKey] = useState("");

  const handleCreateKey = () => {
    if (!selectedProvider || !newApiKey.trim()) return;

    createKey.mutate(
      { provider: selectedProvider, key: newApiKey },
      {
        onSettled: () => {
          setSelectedProvider(null);
          setIsCreateDialogOpen(false);
          setNewApiKey("");
        },
      },
    );
  };

  const handleDeleteKey = (id: string) => {
    deleteKey.mutate(id);
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
          <h1 className="text-3xl font-bold">Provider Keys</h1>
        </div>
        <p className="text-muted-foreground">
          Manage your API keys for different AI providers. Each provider can
          have only one API key configured at a time.
        </p>
      </div>

      {/* Add Key Button */}
      <div className="flex justify-between items-center">
        <div className="text-sm text-muted-foreground">
          {apiKeys?.length} of {availableProviders.length} providers configured
        </div>
        <Dialog open={isCreateDialogOpen} onOpenChange={setIsCreateDialogOpen}>
          <DialogTrigger asChild>
            <Button>
              <Plus className="size-4 mr-2" />
              Add Provider Key
            </Button>
          </DialogTrigger>
          <DialogContent>
            <DialogHeader>
              <DialogTitle>Add AI Provider API Key</DialogTitle>
              <DialogDescription>
                Select a provider and enter your API key.
              </DialogDescription>
            </DialogHeader>
            <div className="grid gap-4 py-4">
              <div className="grid gap-2">
                <Label>Provider</Label>
                <div className="grid gap-2">
                  {availableProviders
                    .filter(
                      (provider) =>
                        !apiKeys?.some((key) => key.provider === provider),
                    )
                    .map((provider) => (
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
              {selectedProvider && (
                <div className="grid gap-2">
                  <Label htmlFor="api-key">API Key</Label>
                  <Input
                    autoFocus
                    id="api-key"
                    type="password"
                    placeholder={`Enter your ${PROVIDERS[selectedProvider].name} API key`}
                    value={newApiKey}
                    onChange={(e) => setNewApiKey(e.target.value)}
                    onKeyDown={(e) => {
                      if (e.key === "Enter") {
                        handleCreateKey();
                      }
                    }}
                  />
                </div>
              )}
            </div>
            <DialogFooter>
              <Button
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
                onClick={handleCreateKey}
                disabled={
                  !selectedProvider || !newApiKey.trim() || createKey.isPending
                }
              >
                {createKey.isPending ? "Adding..." : "Add Key"}
              </Button>
            </DialogFooter>
          </DialogContent>
        </Dialog>
      </div>

      {/* API Keys List */}
      <div className="grid gap-4">
        {apiKeys?.length === 0 ? (
          <Card>
            <CardContent className="pt-6">
              <div className="text-center py-8">
                <Bot className="size-12 mx-auto text-muted-foreground mb-4" />
                <h3 className="text-lg font-semibold mb-2">
                  No API keys configured yet
                </h3>
              </div>
            </CardContent>
          </Card>
        ) : (
          apiKeys?.map((apiKey) => {
            const provider = PROVIDERS[apiKey.provider];
            return (
              <Card
                key={apiKey.provider}
                className={cn("border-2", provider.color)}
              >
                <CardHeader>
                  <div className="flex items-center justify-between">
                    <div>
                      <CardTitle className="text-lg flex items-center gap-2">
                        <Bot className="size-5" />
                        {provider.name}
                      </CardTitle>
                      <CardDescription>
                        {provider.description}
                        <br />
                        Added {formatDate(apiKey.created_at)}
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
                          <AlertDialogTitle>Remove API Key</AlertDialogTitle>
                          <AlertDialogDescription>
                            Are you sure you want to remove the {provider.name}{" "}
                            API key? You won't be able to use {provider.name}{" "}
                            models until you add a new key.
                          </AlertDialogDescription>
                        </AlertDialogHeader>
                        <AlertDialogFooter>
                          <AlertDialogCancel>Cancel</AlertDialogCancel>
                          <AlertDialogAction
                            onClick={() => handleDeleteKey(apiKey.id)}
                            className="bg-red-600 hover:bg-red-700 focus:ring-red-600"
                          >
                            Remove Key
                          </AlertDialogAction>
                        </AlertDialogFooter>
                      </AlertDialogContent>
                    </AlertDialog>
                  </div>
                </CardHeader>
                <CardContent>
                  <div className="flex items-center gap-2">
                    <div className="flex-1 font-mono text-sm bg-muted px-3 py-2 rounded border">
                      *****************
                    </div>
                  </div>
                </CardContent>
              </Card>
            );
          })
        )}
      </div>

      {/* All Providers Configured */}
      {availableProviders.length === 0 && apiKeys && apiKeys.length > 0 && (
        <Card className="border-green-200 bg-green-50 dark:border-green-800 dark:bg-green-950">
          <CardContent className="pt-6">
            <div className="text-center">
              <Bot className="size-8 mx-auto text-green-600 dark:text-green-400 mb-2" />
              <p className="font-medium text-green-800 dark:text-green-200">
                All providers configured!
              </p>
              <p className="text-green-700 dark:text-green-300 text-sm mt-1">
                You have API keys for all supported AI providers.
              </p>
            </div>
          </CardContent>
        </Card>
      )}
    </div>
  );
}
