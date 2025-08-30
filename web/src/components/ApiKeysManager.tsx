import { Bot, Check, Copy, ExternalLink, Plus, Trash2 } from "lucide-react";
import { useId, useState } from "react";

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
import { useApiKeys, useCreateApiKey, useDeleteApiKey } from "@/lib/api/apiKey";
import { API_URL } from "@/lib/api/client";
import { cn } from "@/lib/utils";

export function ApiKeysManager({
  className,
  ...props
}: React.ComponentProps<"div">) {
  const { data: apiKeys } = useApiKeys();
  const createKey = useCreateApiKey();
  const deleteKey = useDeleteApiKey();

  const [isCreateDialogOpen, setIsCreateDialogOpen] = useState(false);
  const [newApiKeyName, setNewApiKeyName] = useState("");
  const [newApiKeyValue, setNewApiKeyValue] = useState("");

  const handleCreateKey = () => {
    if (!newApiKeyName.trim()) return;
    createKey.mutate(
      { name: newApiKeyName },
      {
        onSuccess: (data) => {
          setNewApiKeyValue(data.key);
        },
      },
    );
  };

  const [copied, setCopied] = useState(false);
  const handleCopyKey = () => {
    navigator.clipboard.writeText(newApiKeyValue);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
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

  const nameId = useId();
  const valueId = useId();

  return (
    <div
      className={cn("flex flex-col gap-6 max-w-4xl mx-auto", className)}
      {...props}
    >
      <div className="flex flex-col gap-2">
        <div className="flex items-center gap-2">
          <h1 className="text-3xl font-bold">API Keys</h1>
        </div>
        <p className="text-muted-foreground">
          Manage your API keys for programmatic access to RsChat.
        </p>
      </div>

      {/* Add Key Button */}
      <div className="flex justify-between items-center">
        <div className="text-sm text-muted-foreground">
          {apiKeys?.length} API keys configured
        </div>
        <div className="flex gap-2">
          <Button asChild variant="outline">
            <a href={`${API_URL}/docs`} target="_blank">
              <ExternalLink />
              API Docs
            </a>
          </Button>
          <Dialog
            open={isCreateDialogOpen}
            onOpenChange={setIsCreateDialogOpen}
          >
            <DialogTrigger asChild>
              <Button>
                <Plus className="size-4" />
                Add API Key
              </Button>
            </DialogTrigger>
            <DialogContent>
              <DialogHeader>
                <DialogTitle>Add API Key</DialogTitle>
                <DialogDescription>
                  Enter a name for your API key.
                </DialogDescription>
              </DialogHeader>
              <div className="grid gap-4 py-4">
                <div className="grid gap-2">
                  <Label htmlFor={nameId}>Name</Label>
                  <Input
                    autoFocus
                    id={nameId}
                    type="text"
                    placeholder="My API Key"
                    value={newApiKeyName}
                    onChange={(e) => setNewApiKeyName(e.target.value)}
                    onKeyDown={(e) => {
                      if (e.key === "Enter") {
                        handleCreateKey();
                      }
                    }}
                    disabled={createKey.isPending || !!newApiKeyValue}
                  />
                </div>
                {newApiKeyValue && (
                  <div className="grid gap-2">
                    <Label htmlFor={valueId}>
                      Key (copy and save this - won't be shown again!)
                    </Label>
                    <div className="flex gap-2">
                      <Input
                        id={valueId}
                        type="text"
                        readOnly
                        value={newApiKeyValue}
                      />
                      <Button onClick={handleCopyKey} variant="outline">
                        {copied ? <Check /> : <Copy />}
                      </Button>
                    </div>
                  </div>
                )}
              </div>
              <DialogFooter>
                <Button
                  variant="outline"
                  onClick={() => {
                    setIsCreateDialogOpen(false);
                    setNewApiKeyName("");
                    setNewApiKeyValue("");
                  }}
                >
                  {newApiKeyValue ? "Close" : "Cancel"}
                </Button>
                {!newApiKeyValue && (
                  <Button
                    onClick={handleCreateKey}
                    disabled={!newApiKeyName.trim() || createKey.isPending}
                  >
                    {createKey.isPending ? "Creating..." : "Create Key"}
                  </Button>
                )}
              </DialogFooter>
            </DialogContent>
          </Dialog>
        </div>
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
            return (
              <Card
                key={apiKey.id}
                className="border-2 bg-blue-50 dark:bg-blue-800 border-blue-300 dark:border-blue-700"
              >
                <CardHeader>
                  <div className="flex items-center justify-between">
                    <div>
                      <CardTitle className="text-lg flex items-center gap-2">
                        <Bot className="size-5" />
                        {apiKey.name}
                      </CardTitle>
                      <CardDescription>
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
                          <AlertDialogTitle>Delete API Key</AlertDialogTitle>
                          <AlertDialogDescription>
                            Are you sure you want to delete this API key?
                          </AlertDialogDescription>
                        </AlertDialogHeader>
                        <AlertDialogFooter>
                          <AlertDialogCancel>Cancel</AlertDialogCancel>
                          <AlertDialogAction
                            onClick={() => handleDeleteKey(apiKey.id)}
                            className="bg-red-600 hover:bg-red-700 focus:ring-red-600"
                          >
                            Delete Key
                          </AlertDialogAction>
                        </AlertDialogFooter>
                      </AlertDialogContent>
                    </AlertDialog>
                  </div>
                </CardHeader>
                <CardContent>
                  <div className="flex items-center gap-2">
                    <div className="flex-1 font-mono text-sm bg-muted px-3 py-2 rounded border">
                      rs-chat-key|*****************
                    </div>
                  </div>
                </CardContent>
              </Card>
            );
          })
        )}
      </div>
    </div>
  );
}
