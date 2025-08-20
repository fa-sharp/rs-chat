import { CloudCog, Plus, Trash2 } from "lucide-react";
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
import { Textarea } from "@/components/ui/textarea";
import { useCreateTool } from "@/lib/api/tool";
import type { components } from "@/lib/api/types";

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

interface HttpRequest {
  id: string;
  name: string;
  description: string;
  url: string;
  method: HttpMethod;
  headers: HeaderField[];
  body: string;
  parameters: Parameter[];
}

interface CustomApiToolFormProps {
  onSuccess?: () => void;
  onCancel?: () => void;
}

export function CustomApiToolForm({
  onSuccess,
  onCancel,
}: CustomApiToolFormProps) {
  const createTool = useCreateTool();

  const [toolName, setToolName] = useState("");
  const [requests, setRequests] = useState<HttpRequest[]>([]);
  const [error, setError] = useState<string>("");
  const [isSubmitting, setIsSubmitting] = useState(false);

  const addRequest = () => {
    const newRequest: HttpRequest = {
      id: crypto.randomUUID(),
      name: "",
      description: "",
      url: "",
      method: "GET",
      headers: [],
      body: "",
      parameters: [],
    };
    setRequests([...requests, newRequest]);
  };

  const removeRequest = (id: string) => {
    setRequests(requests.filter((req) => req.id !== id));
  };

  const updateRequest = (id: string, updates: Partial<HttpRequest>) => {
    setRequests(
      requests.map((req) => (req.id === id ? { ...req, ...updates } : req)),
    );
  };

  const addParameter = (requestId: string) => {
    const newParameter: Parameter = {
      id: crypto.randomUUID(),
      name: "",
      type: "string",
      description: "",
      required: false,
    };
    updateRequest(requestId, {
      parameters: [
        ...(requests.find((r) => r.id === requestId)?.parameters || []),
        newParameter,
      ],
    });
  };

  const removeParameter = (requestId: string, parameterId: string) => {
    const request = requests.find((r) => r.id === requestId);
    if (request) {
      updateRequest(requestId, {
        parameters: request.parameters.filter((p) => p.id !== parameterId),
      });
    }
  };

  const updateParameter = (
    requestId: string,
    parameterId: string,
    updates: Partial<Parameter>,
  ) => {
    const request = requests.find((r) => r.id === requestId);
    if (request) {
      updateRequest(requestId, {
        parameters: request.parameters.map((p) =>
          p.id === parameterId ? { ...p, ...updates } : p,
        ),
      });
    }
  };

  const addHeader = (requestId: string) => {
    const newHeader: HeaderField = {
      id: crypto.randomUUID(),
      key: "",
      value: "",
    };
    const request = requests.find((r) => r.id === requestId);
    if (request) {
      updateRequest(requestId, {
        headers: [...request.headers, newHeader],
      });
    }
  };

  const removeHeader = (requestId: string, headerId: string) => {
    const request = requests.find((r) => r.id === requestId);
    if (request) {
      updateRequest(requestId, {
        headers: request.headers.filter((h) => h.id !== headerId),
      });
    }
  };

  const updateHeader = (
    requestId: string,
    headerId: string,
    updates: Partial<HeaderField>,
  ) => {
    const request = requests.find((r) => r.id === requestId);
    if (request) {
      updateRequest(requestId, {
        headers: request.headers.map((h) =>
          h.id === headerId ? { ...h, ...updates } : h,
        ),
      });
    }
  };

  const generateJsonSchema = (parameters: Parameter[]) => {
    const properties: Record<string, any> = {};
    const required: string[] = [];

    for (const param of parameters) {
      if (param.required) {
        required.push(param.name);
      }

      if (param.type === "array") {
        properties[param.name] = {
          type: "array",
          items: { type: param.arrayItemType || "string" },
          description: param.description,
        };
      } else {
        properties[param.name] = {
          type: param.type,
          description: param.description,
        };
      }
    }

    return {
      type: "object" as const,
      properties,
      required: required.length > 0 ? required : undefined,
      additionalProperties: false,
    };
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError("");
    setIsSubmitting(true);

    if (!toolName.trim()) {
      setError("Please enter a tool name");
      setIsSubmitting(false);
      return;
    }

    if (requests.length === 0) {
      setError("Please add at least one HTTP request");
      setIsSubmitting(false);
      return;
    }

    // Validate requests
    for (const request of requests) {
      if (!request.name.trim()) {
        setError("All requests must have a name");
        setIsSubmitting(false);
        return;
      }
      if (!request.url.trim()) {
        setError("All requests must have a URL");
        setIsSubmitting(false);
        return;
      }
      for (const param of request.parameters) {
        if (!param.name.trim()) {
          setError("All parameters must have a name");
          setIsSubmitting(false);
          return;
        }
      }
    }

    try {
      const tools: Record<string, components["schemas"]["HttpRequestConfig"]> =
        {};

      for (const request of requests) {
        const headersObj: Record<string, string> = {};
        for (const header of request.headers) {
          if (header.key.trim() && header.value.trim()) {
            headersObj[header.key] = header.value;
          }
        }

        tools[request.name] = {
          description: request.description,
          url: request.url,
          method: request.method,
          headers: Object.keys(headersObj).length > 0 ? headersObj : undefined,
          body: request.body.trim() ? JSON.parse(request.body) : undefined,
          input_schema: generateJsonSchema(request.parameters),
        };
      }

      const config: components["schemas"]["CustomApiConfig"] = {
        name: toolName,
        tools,
      };

      const toolInput: components["schemas"]["CreateToolInput"] = {
        external_api: {
          type: "custom_api",
          config,
        },
      };

      await createTool.mutateAsync(toolInput);
      onSuccess?.();
    } catch (error) {
      console.error("Failed to create custom API tool:", error);
      setError(
        "Failed to create custom API tool. Please check your configuration.",
      );
    } finally {
      setIsSubmitting(false);
    }
  };

  const handleReset = () => {
    setToolName("");
    setRequests([]);
    setError("");
  };

  const toolNameId = useId();

  return (
    <form onSubmit={handleSubmit} className="space-y-6">
      <div className="flex items-center gap-2 mb-4">
        <CloudCog className="size-5" />
        <h3 className="text-lg font-medium">Custom API Tool</h3>
      </div>

      <div>
        <Label htmlFor={toolNameId} className="mb-2">
          Tool Name
        </Label>
        <Input
          id={toolNameId}
          value={toolName}
          pattern="^[a-zA-Z0-9_]+$"
          onChange={(e) => setToolName(e.target.value)}
          placeholder="No spaces, e.g. my_cool_api, other_api, etc."
        />
      </div>

      <div className="space-y-4">
        <div className="flex items-center justify-between">
          <h4 className="font-medium">HTTP Requests</h4>
          <Button type="button" onClick={addRequest} size="sm">
            <Plus className="size-4 mr-1" />
            Add Request
          </Button>
        </div>

        {requests.map((request, requestIndex) => (
          <div key={request.id} className="border rounded-lg p-4 space-y-4">
            <div className="flex items-center justify-between">
              <h5 className="font-medium">Request {requestIndex + 1}</h5>
              <Button
                type="button"
                variant="ghost"
                size="sm"
                onClick={() => removeRequest(request.id)}
              >
                <Trash2 className="size-4" />
              </Button>
            </div>

            <div className="grid grid-cols-2 gap-4">
              <div>
                <Label className="mb-1">Request Name</Label>
                <Input
                  value={request.name}
                  onChange={(e) =>
                    updateRequest(request.id, { name: e.target.value })
                  }
                  placeholder="No spaces, e.g. get_user"
                />
              </div>
              <div>
                <Label className="mb-1">Method</Label>
                <Select
                  value={request.method}
                  onValueChange={(value) =>
                    updateRequest(request.id, { method: value as HttpMethod })
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
            </div>

            <div>
              <Label className="mb-1">Description</Label>
              <Input
                value={request.description}
                onChange={(e) =>
                  updateRequest(request.id, { description: e.target.value })
                }
                placeholder="What does this request do?"
              />
            </div>

            <div>
              <Label className="mb-1">URL</Label>
              <Input
                value={request.url}
                onChange={(e) =>
                  updateRequest(request.id, { url: e.target.value })
                }
                placeholder="https://api.example.com/users/${user_id}"
              />
            </div>

            <div>
              <div className="flex items-center justify-between mb-2">
                <Label>Headers</Label>
                <Button
                  type="button"
                  variant="outline"
                  size="sm"
                  onClick={() => addHeader(request.id)}
                >
                  <Plus className="size-3 mr-1" />
                  Add Header
                </Button>
              </div>
              {request.headers.map((header) => (
                <div key={header.id} className="flex gap-2 mb-2">
                  <Input
                    placeholder="Header name"
                    value={header.key}
                    onChange={(e) =>
                      updateHeader(request.id, header.id, {
                        key: e.target.value,
                      })
                    }
                  />
                  <Input
                    placeholder="Header value"
                    value={header.value}
                    onChange={(e) =>
                      updateHeader(request.id, header.id, {
                        value: e.target.value,
                      })
                    }
                  />
                  <Button
                    type="button"
                    variant="ghost"
                    size="sm"
                    onClick={() => removeHeader(request.id, header.id)}
                  >
                    <Trash2 className="size-4" />
                  </Button>
                </div>
              ))}
            </div>

            <div>
              <Label className="mb-1">Request Body (JSON)</Label>
              <Textarea
                value={request.body}
                onChange={(e) =>
                  updateRequest(request.id, { body: e.target.value })
                }
                placeholder='{"key": "$parameter_name"}'
                rows={4}
              />
            </div>

            <div>
              <div className="flex items-center justify-between mb-2">
                <Label>Input Parameters</Label>
                <Button
                  type="button"
                  variant="outline"
                  size="sm"
                  onClick={() => addParameter(request.id)}
                >
                  <Plus className="size-3 mr-1" />
                  Add Parameter
                </Button>
              </div>
              {request.parameters.map((param, idx) => (
                <div key={param.id} className="flex items-center gap-2 mb-4">
                  <div className="min-w-4">{idx + 1}.</div>
                  <div className="grid grid-cols-[1fr 2fr] gap-2 w-full">
                    <Input
                      placeholder="Name"
                      value={param.name}
                      onChange={(e) =>
                        updateParameter(request.id, param.id, {
                          name: e.target.value,
                        })
                      }
                    />
                    <div className="flex items-center gap-2">
                      <Select
                        value={param.type}
                        onValueChange={(value) =>
                          updateParameter(request.id, param.id, {
                            type: value as ExtendedParameterType,
                          })
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

                      {param.type === "array" && (
                        <Select
                          value={param.arrayItemType || "string"}
                          onValueChange={(value) =>
                            updateParameter(request.id, param.id, {
                              arrayItemType: value as ParameterType,
                            })
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
                      )}
                      <Button
                        type="button"
                        variant="ghost"
                        size="sm"
                        onClick={() => removeParameter(request.id, param.id)}
                      >
                        <Trash2 className="size-4" />
                      </Button>
                    </div>

                    <Input
                      className="col-span-2"
                      placeholder="Description"
                      value={param.description}
                      onChange={(e) =>
                        updateParameter(request.id, param.id, {
                          description: e.target.value,
                        })
                      }
                    />
                  </div>
                </div>
              ))}
            </div>
          </div>
        ))}
      </div>

      {error && (
        <div className="text-sm text-red-600 bg-red-50 p-3 rounded-md">
          {error}
        </div>
      )}

      <div className="flex gap-2 pt-4">
        <Button type="submit" disabled={isSubmitting}>
          {isSubmitting ? "Creating..." : "Create Custom API Tool"}
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
