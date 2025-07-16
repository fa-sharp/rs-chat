import { Globe, Plus, Settings, Trash2, Wrench } from "lucide-react";
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
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Textarea } from "@/components/ui/textarea";
import { useCreateTool, useDeleteTool, useTools } from "@/lib/api/tool";
import type { components } from "@/lib/api/types";
import { cn } from "@/lib/utils";

type ToolType = "web_search" | "http_request";
type HttpMethod = "GET" | "POST" | "PUT" | "DELETE" | "PATCH";
type ParameterType = "string" | "number" | "boolean";
type ExtendedParameterType = ParameterType | "array";

interface Parameter {
  id: string;
  name: string;
  type: ExtendedParameterType;
  arrayItemType?: ParameterType;
  description: string;
  required: boolean;
}

interface HeaderField {
  id: string;
  key: string;
  value: string;
}

interface BodyField {
  id: string;
  key: string;
  value: string;
  type: ParameterType;
}

const WEB_SEARCH_PROVIDERS = [
  { value: "brave", label: "Brave Search" },
  { value: "serpapi", label: "SerpApi (Google)" },
  { value: "googlecustomsearch", label: "Google Custom Search" },
  { value: "exa", label: "Exa" },
] as const;

export const getToolIcon = (tool: components["schemas"]["ChatRsTool"]) => {
  if (tool.config.type === "Http") {
    return <Wrench className="size-5" aria-hidden />;
  }
  return <Globe className="size-5" aria-hidden />;
};

export const getToolTypeLabel = (tool: components["schemas"]["ChatRsTool"]) => {
  if (tool.config.type === "Http") {
    return "HTTP Request";
  }
  return "Web Search";
};

export function ToolsManager({
  className,
  ...props
}: React.ComponentProps<"div">) {
  const { data: tools } = useTools();
  const createTool = useCreateTool();
  const deleteTool = useDeleteTool();

  const [isCreateDialogOpen, setIsCreateDialogOpen] = useState(false);
  const [selectedToolType, setSelectedToolType] = useState<ToolType | null>(
    null,
  );

  // Common fields
  const [toolName, setToolName] = useState("");
  const [toolDescription, setToolDescription] = useState("");

  // Web search fields
  const [searchProvider, setSearchProvider] = useState<string>("");
  const [searchApiKey, setSearchApiKey] = useState("");

  // HTTP request fields
  const [httpUrl, setHttpUrl] = useState("");
  const [httpMethod, setHttpMethod] = useState<HttpMethod>("GET");
  const [headerFields, setHeaderFields] = useState<HeaderField[]>([]);
  const [bodyFields, setBodyFields] = useState<BodyField[]>([]);
  const [parameters, setParameters] = useState<Parameter[]>([]);

  const resetForm = () => {
    setSelectedToolType(null);
    setToolName("");
    setToolDescription("");
    setSearchProvider("");
    setSearchApiKey("");
    setHttpUrl("");
    setHttpMethod("GET");
    setHeaderFields([]);
    setBodyFields([]);
    setParameters([]);
  };

  const handleCreateTool = () => {
    if (!selectedToolType || !toolName.trim() || !toolDescription.trim()) {
      return;
    }

    if (selectedToolType === "web_search") {
      if (!searchProvider || !searchApiKey.trim()) return;

      const toolInput: components["schemas"]["ToolInput"] = {
        name: toolName,
        description: toolDescription,
        config: {
          type: "WebSearch",
          provider: {
            type: searchProvider as any,
            api_key: searchApiKey,
            country: null,
            search_lang: null,
            ...(searchProvider === "serpapi" && { engine: null }),
            ...(searchProvider === "googlecustomsearch" && { cx: "" }),
          },
          count: 10,
        },
      };

      createTool.mutate(toolInput, {
        onSuccess: () => {
          setIsCreateDialogOpen(false);
          resetForm();
        },
      });
    } else if (selectedToolType === "http_request") {
      if (!httpUrl.trim()) return;

      // Build the JSON schema from parameters
      const properties: Record<string, any> = {};
      const required: string[] = [];

      parameters.forEach((param) => {
        if (param.type === "array") {
          properties[param.name] = {
            type: "array",
            items: {
              type: param.arrayItemType || "string",
            },
            description: param.description,
          };
        } else {
          properties[param.name] = {
            type: param.type,
            description: param.description,
          };
        }
        if (param.required) {
          required.push(param.name);
        }
      });

      // Build headers from header fields
      const headers: Record<string, string> = {};
      headerFields.forEach((field) => {
        if (field.key.trim() && field.value.trim()) {
          headers[field.key] = field.value;
        }
      });

      // Build body from body fields
      let body: any = null;
      if (bodyFields.length > 0) {
        body = {};
        bodyFields.forEach((field) => {
          if (field.key.trim()) {
            let value: any = field.value;
            if (field.type === "number") {
              value = Number(field.value) || 0;
            } else if (field.type === "boolean") {
              value = field.value.toLowerCase() === "true";
            }
            body[field.key] = value;
          }
        });
      }

      const toolInput: components["schemas"]["ToolInput"] = {
        name: toolName,
        description: toolDescription,
        config: {
          type: "Http",
          input_schema: {
            type: "object",
            properties,
            required: required.length > 0 ? required : null,
            additionalProperties: false,
          },
          url: httpUrl,
          method: httpMethod,
          headers: Object.keys(headers).length > 0 ? headers : null,
          body,
          query: null,
        },
      };

      createTool.mutate(toolInput, {
        onSuccess: () => {
          setIsCreateDialogOpen(false);
          resetForm();
        },
      });
    }
  };

  const handleDeleteTool = (toolId: string) => {
    deleteTool.mutate(toolId);
  };

  const addParameter = () => {
    setParameters([
      ...parameters,
      {
        id: crypto.randomUUID(),
        name: "",
        type: "string",
        arrayItemType: undefined,
        description: "",
        required: false,
      },
    ]);
  };

  const updateParameter = (
    index: number,
    field: keyof Parameter,
    value: any,
  ) => {
    const updated = [...parameters];
    updated[index] = { ...updated[index], [field]: value };
    setParameters(updated);
  };

  const removeParameter = (index: number) => {
    setParameters(parameters.filter((_, i) => i !== index));
  };

  const addHeaderField = () => {
    setHeaderFields([
      ...headerFields,
      { id: crypto.randomUUID(), key: "", value: "" },
    ]);
  };

  const updateHeaderField = (
    index: number,
    field: keyof HeaderField,
    value: string,
  ) => {
    const updated = [...headerFields];
    updated[index] = { ...updated[index], [field]: value };
    setHeaderFields(updated);
  };

  const removeHeaderField = (index: number) => {
    setHeaderFields(headerFields.filter((_, i) => i !== index));
  };

  const addBodyField = () => {
    setBodyFields([
      ...bodyFields,
      { id: crypto.randomUUID(), key: "", value: "", type: "string" },
    ]);
  };

  const updateBodyField = (
    index: number,
    field: keyof BodyField,
    value: any,
  ) => {
    const updated = [...bodyFields];
    updated[index] = { ...updated[index], [field]: value };
    setBodyFields(updated);
  };

  const removeBodyField = (index: number) => {
    setBodyFields(bodyFields.filter((_, i) => i !== index));
  };

  const generateJsonSchema = () => {
    const properties: Record<string, any> = {};
    const required: string[] = [];

    parameters.forEach((param) => {
      if (param.type === "array") {
        properties[param.name] = {
          type: "array",
          items: {
            type: param.arrayItemType || "string",
          },
          description: param.description,
        };
      } else {
        properties[param.name] = {
          type: param.type,
          description: param.description,
        };
      }
      if (param.required) {
        required.push(param.name);
      }
    });

    return {
      type: "object",
      properties,
      required: required.length > 0 ? required : undefined,
      additionalProperties: false,
    };
  };

  return (
    <div
      className={cn("flex flex-col gap-6 max-w-4xl mx-auto", className)}
      {...props}
    >
      <div className="flex flex-col gap-2">
        <div className="flex items-center gap-2">
          <h1 className="text-3xl font-bold">Tools</h1>
        </div>
        <p className="text-muted-foreground">
          Configure tools that can be used during conversations to enhance AI
          capabilities.
        </p>
      </div>

      {/* Add Tool Button */}
      <div className="flex justify-between items-center">
        <div className="text-sm text-muted-foreground">
          {tools?.length || 0} tools configured
        </div>
        <Dialog open={isCreateDialogOpen} onOpenChange={setIsCreateDialogOpen}>
          <DialogTrigger asChild>
            <Button>
              <Plus className="size-4" />
              Add Tool
            </Button>
          </DialogTrigger>
          <DialogContent className="max-w-2xl max-h-[80vh] overflow-y-auto">
            <DialogHeader>
              <DialogTitle>Add New Tool</DialogTitle>
              <DialogDescription>
                Configure a new tool to extend AI capabilities.
              </DialogDescription>
            </DialogHeader>
            <div className="grid gap-4 py-4">
              {/* Tool Type Selection */}
              {!selectedToolType && (
                <div className="grid gap-3">
                  <Label>Tool Type</Label>
                  <div className="grid gap-2">
                    <button
                      type="button"
                      className="p-4 border rounded-lg hover:bg-muted/50 transition-colors text-left"
                      onClick={() => setSelectedToolType("web_search")}
                    >
                      <div className="flex items-center gap-3">
                        <Globe className="size-8 text-blue-500" />
                        <div>
                          <div className="font-medium">Web Search</div>
                          <div className="text-sm text-muted-foreground">
                            Search the web for real-time information
                          </div>
                        </div>
                      </div>
                    </button>
                    <button
                      type="button"
                      className="p-4 border rounded-lg hover:bg-muted/50 transition-colors text-left"
                      onClick={() => setSelectedToolType("http_request")}
                    >
                      <div className="flex items-center gap-3">
                        <Wrench className="size-8 text-green-500" />
                        <div>
                          <div className="font-medium">HTTP Request</div>
                          <div className="text-sm text-muted-foreground">
                            Make HTTP requests to external APIs
                          </div>
                        </div>
                      </div>
                    </button>
                  </div>
                </div>
              )}

              {/* Common Fields */}
              {selectedToolType && (
                <>
                  <div className="grid gap-2">
                    <Label htmlFor="tool-name">Name</Label>
                    <Input
                      id="tool-name"
                      placeholder="My Tool"
                      value={toolName}
                      onChange={(e) => setToolName(e.target.value)}
                    />
                  </div>
                  <div className="grid gap-2">
                    <Label htmlFor="tool-description">Description</Label>
                    <Textarea
                      id="tool-description"
                      placeholder="What this tool does..."
                      value={toolDescription}
                      onChange={(e) => setToolDescription(e.target.value)}
                    />
                  </div>
                </>
              )}

              {/* Web Search Configuration */}
              {selectedToolType === "web_search" && (
                <>
                  <div className="grid gap-2">
                    <Label>Search Provider</Label>
                    <Select
                      value={searchProvider}
                      onValueChange={setSearchProvider}
                    >
                      <SelectTrigger>
                        <SelectValue placeholder="Select a provider" />
                      </SelectTrigger>
                      <SelectContent>
                        {WEB_SEARCH_PROVIDERS.map((provider) => (
                          <SelectItem
                            key={provider.value}
                            value={provider.value}
                          >
                            {provider.label}
                          </SelectItem>
                        ))}
                      </SelectContent>
                    </Select>
                  </div>
                  <div className="grid gap-2">
                    <Label htmlFor="search-api-key">API Key</Label>
                    <Input
                      id="search-api-key"
                      type="password"
                      placeholder="Enter your API key"
                      value={searchApiKey}
                      onChange={(e) => setSearchApiKey(e.target.value)}
                    />
                  </div>
                </>
              )}

              {/* HTTP Request Configuration */}
              {selectedToolType === "http_request" && (
                <>
                  <div className="grid gap-2">
                    <Label htmlFor="http-url">URL</Label>
                    <Input
                      id="http-url"
                      placeholder="https://api.example.com/endpoint"
                      value={httpUrl}
                      onChange={(e) => setHttpUrl(e.target.value)}
                    />
                  </div>
                  <div className="grid gap-2">
                    <Label>Method</Label>
                    <Select
                      value={httpMethod}
                      onValueChange={(value: HttpMethod) =>
                        setHttpMethod(value)
                      }
                    >
                      <SelectTrigger>
                        <SelectValue />
                      </SelectTrigger>
                      <SelectContent>
                        <SelectItem value="GET">GET</SelectItem>
                        <SelectItem value="POST">POST</SelectItem>
                        <SelectItem value="PUT">PUT</SelectItem>
                        <SelectItem value="DELETE">DELETE</SelectItem>
                        <SelectItem value="PATCH">PATCH</SelectItem>
                      </SelectContent>
                    </Select>
                  </div>
                  <div className="grid gap-2">
                    <div className="flex items-center justify-between">
                      <Label>Headers</Label>
                      <Button
                        type="button"
                        variant="outline"
                        size="sm"
                        onClick={addHeaderField}
                      >
                        <Plus className="size-3" />
                        Add Header
                      </Button>
                    </div>
                    {headerFields.map((field, index) => (
                      <div
                        key={field.id}
                        className="flex items-center gap-2 border rounded-lg p-2"
                      >
                        <Input
                          className="max-w-36"
                          placeholder="Name"
                          value={field.key}
                          onChange={(e) =>
                            updateHeaderField(index, "key", e.target.value)
                          }
                        />
                        <Input
                          placeholder="Value (use $param for parameters)"
                          value={field.value}
                          onChange={(e) =>
                            updateHeaderField(index, "value", e.target.value)
                          }
                        />
                        <Button
                          type="button"
                          variant="ghost"
                          size="sm"
                          onClick={() => removeHeaderField(index)}
                        >
                          <Trash2 className="size-3" />
                        </Button>
                      </div>
                    ))}
                  </div>
                  <div className="grid gap-2">
                    <div className="flex items-center justify-between">
                      <Label>Body</Label>
                      <Button
                        type="button"
                        variant="outline"
                        size="sm"
                        onClick={addBodyField}
                      >
                        <Plus className="size-3" />
                        Add Field
                      </Button>
                    </div>
                    {bodyFields.map((field, index) => (
                      <div
                        key={field.id}
                        className="border rounded-lg p-2 space-y-2"
                      >
                        <div className="flex items-center gap-2">
                          <Input
                            placeholder="Field name"
                            value={field.key}
                            onChange={(e) =>
                              updateBodyField(index, "key", e.target.value)
                            }
                          />
                          <Select
                            value={field.type}
                            onValueChange={(value: ParameterType) =>
                              updateBodyField(index, "type", value)
                            }
                          >
                            <SelectTrigger className="w-48">
                              <SelectValue />
                            </SelectTrigger>
                            <SelectContent>
                              <SelectItem value="string">
                                String/Replace
                              </SelectItem>
                              <SelectItem value="number">Number</SelectItem>
                              <SelectItem value="boolean">Boolean</SelectItem>
                            </SelectContent>
                          </Select>
                          <Button
                            type="button"
                            variant="ghost"
                            size="sm"
                            onClick={() => removeBodyField(index)}
                          >
                            <Trash2 className="size-3" />
                          </Button>
                        </div>
                        <Input
                          placeholder={`Field value${field.type === "string" ? " (use $param for parameters)" : ""}`}
                          value={field.value}
                          onChange={(e) =>
                            updateBodyField(index, "value", e.target.value)
                          }
                        />
                      </div>
                    ))}
                  </div>

                  {/* Parameters */}
                  <div className="grid gap-2">
                    <div className="flex items-center justify-between">
                      <Label>Input Parameters</Label>
                      <Button
                        type="button"
                        variant="outline"
                        size="sm"
                        onClick={addParameter}
                      >
                        <Plus className="size-3" />
                        Add Parameter
                      </Button>
                    </div>
                    {parameters.map((param, index) => (
                      <div
                        key={param.id}
                        className="border rounded-lg p-3 space-y-2"
                      >
                        <div className="flex items-center justify-between">
                          <span className="text-sm font-medium">
                            Parameter {index + 1}
                          </span>
                          <Button
                            type="button"
                            variant="ghost"
                            size="sm"
                            onClick={() => removeParameter(index)}
                          >
                            <Trash2 className="size-3" />
                          </Button>
                        </div>
                        <div className="grid grid-cols-2 gap-2">
                          <Input
                            placeholder="Name"
                            value={param.name}
                            onChange={(e) =>
                              updateParameter(index, "name", e.target.value)
                            }
                          />
                          <Select
                            value={param.type}
                            onValueChange={(value: ExtendedParameterType) =>
                              updateParameter(index, "type", value)
                            }
                          >
                            <SelectTrigger>
                              <SelectValue />
                            </SelectTrigger>
                            <SelectContent>
                              <SelectItem value="string">String</SelectItem>
                              <SelectItem value="number">Number</SelectItem>
                              <SelectItem value="boolean">Boolean</SelectItem>
                              <SelectItem value="array">Array</SelectItem>
                            </SelectContent>
                          </Select>
                        </div>
                        {param.type === "array" && (
                          <div className="grid gap-2">
                            <Label className="text-sm">Array Item Type</Label>
                            <Select
                              value={param.arrayItemType || "string"}
                              onValueChange={(value: ParameterType) =>
                                updateParameter(index, "arrayItemType", value)
                              }
                            >
                              <SelectTrigger>
                                <SelectValue />
                              </SelectTrigger>
                              <SelectContent>
                                <SelectItem value="string">String</SelectItem>
                                <SelectItem value="number">Number</SelectItem>
                                <SelectItem value="boolean">Boolean</SelectItem>
                              </SelectContent>
                            </Select>
                          </div>
                        )}
                        <Input
                          placeholder="Description"
                          value={param.description}
                          onChange={(e) =>
                            updateParameter(
                              index,
                              "description",
                              e.target.value,
                            )
                          }
                        />
                        <div className="flex items-center gap-2">
                          <input
                            type="checkbox"
                            id={`required-${index}`}
                            checked={param.required}
                            onChange={(e) =>
                              updateParameter(
                                index,
                                "required",
                                e.target.checked,
                              )
                            }
                          />
                          <Label
                            htmlFor={`required-${index}`}
                            className="text-sm"
                          >
                            Required
                          </Label>
                        </div>
                      </div>
                    ))}
                  </div>

                  {/* JSON Schema Preview */}
                  {parameters.length > 0 && (
                    <div className="grid gap-2">
                      <Label>JSON Schema Preview</Label>
                      <div className="bg-muted p-3 rounded-lg">
                        <pre className="text-xs overflow-auto">
                          {JSON.stringify(generateJsonSchema(), null, 2)}
                        </pre>
                      </div>
                    </div>
                  )}
                </>
              )}
            </div>
            <DialogFooter>
              <Button
                variant="outline"
                onClick={() => {
                  setIsCreateDialogOpen(false);
                  resetForm();
                }}
              >
                Cancel
              </Button>
              <Button
                onClick={handleCreateTool}
                disabled={
                  !selectedToolType ||
                  !toolName.trim() ||
                  !toolDescription.trim() ||
                  createTool.isPending ||
                  (selectedToolType === "web_search" &&
                    (!searchProvider || !searchApiKey.trim())) ||
                  (selectedToolType === "http_request" && !httpUrl.trim())
                }
              >
                {createTool.isPending ? "Creating..." : "Create Tool"}
              </Button>
            </DialogFooter>
          </DialogContent>
        </Dialog>
      </div>

      {/* Tools List */}
      <div className="grid gap-4">
        {tools?.length === 0 ? (
          <Card>
            <CardContent className="pt-6">
              <div className="text-center py-8">
                <Settings className="size-12 mx-auto text-muted-foreground mb-4" />
                <h3 className="text-lg font-semibold mb-2">
                  No tools configured yet
                </h3>
                <p className="text-muted-foreground">
                  Add your first tool to extend AI capabilities.
                </p>
              </div>
            </CardContent>
          </Card>
        ) : (
          tools?.map((tool) => (
            <Card
              key={tool.id}
              className="border-2 bg-purple-50 dark:bg-purple-950 border-purple-300 dark:border-purple-700"
            >
              <CardHeader>
                <div className="flex items-center justify-between">
                  <div>
                    <CardTitle className="text-lg flex items-center gap-2">
                      {getToolIcon(tool)}
                      {tool.name}
                    </CardTitle>
                    <CardDescription>
                      {getToolTypeLabel(tool)} • {tool.description}
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
                        <AlertDialogTitle>Delete Tool</AlertDialogTitle>
                        <AlertDialogDescription>
                          Are you sure you want to delete "{tool.name}"? This
                          action cannot be undone.
                        </AlertDialogDescription>
                      </AlertDialogHeader>
                      <AlertDialogFooter>
                        <AlertDialogCancel>Cancel</AlertDialogCancel>
                        <AlertDialogAction
                          onClick={() => handleDeleteTool(tool.id)}
                          className="bg-red-600 hover:bg-red-700 focus:ring-red-600"
                        >
                          Delete Tool
                        </AlertDialogAction>
                      </AlertDialogFooter>
                    </AlertDialogContent>
                  </AlertDialog>
                </div>
              </CardHeader>
              <CardContent>
                <div className="space-y-2">
                  {tool.config.type === "Http" && (
                    <div className="text-sm">
                      <div className="font-mono bg-muted px-2 py-1 rounded">
                        {tool.config.method} {tool.config.url}
                      </div>
                    </div>
                  )}
                  {tool.config.type === "WebSearch" && (
                    <div className="font-semibold">
                      Search Provider:{" "}
                      {
                        WEB_SEARCH_PROVIDERS.find(
                          (p) =>
                            tool.config.type === "WebSearch" &&
                            p.value === tool.config.provider.type,
                        )?.label
                      }
                    </div>
                  )}
                </div>
              </CardContent>
            </Card>
          ))
        )}
      </div>
    </div>
  );
}
