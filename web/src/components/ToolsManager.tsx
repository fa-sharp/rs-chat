import {
  CloudCog,
  Code2,
  FolderCog,
  Globe,
  Plus,
  Trash2,
  Wrench,
} from "lucide-react";
import { useState } from "react";

import {
  CodeRunnerToolForm,
  CustomApiToolForm,
  SystemInfoToolForm,
  WebSearchToolForm,
} from "@/components/tools";
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
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { useDeleteExternalApiTool, useTools } from "@/lib/api/tool";
import type { components } from "@/lib/api/types";
import { cn } from "@/lib/utils";

type ToolCreationType =
  | "code_runner"
  | "system_info"
  | "web_search"
  | "custom_api";

const TOOL_TYPE_LABELS: Record<ToolCreationType, string> = {
  code_runner: "Code Runner",
  system_info: "System Info",
  web_search: "Web Search",
  custom_api: "Custom API",
};

export const getToolIcon = (
  tool:
    | components["schemas"]["ChatRsSystemTool"]
    | components["schemas"]["ChatRsExternalApiTool"],
) => {
  if (tool.data.type === "system_info") {
    return <Wrench className="size-5" aria-hidden />;
  }
  if (tool.data.type === "code_runner") {
    return <Code2 className="size-5" aria-hidden />;
  }
  if (tool.data.type === "web_search") {
    return <Globe className="size-5" aria-hidden />;
  }
  if (tool.data.type === "custom_api") {
    return <CloudCog className="size-5" aria-hidden />;
  }
  if (tool.data.type === "files") {
    return <FolderCog className="size-5" aria-hidden />;
  }
  return <Wrench className="size-5" aria-hidden />;
};

export const getToolTypeLabel = (
  tool:
    | components["schemas"]["ChatRsSystemTool"]
    | components["schemas"]["ChatRsExternalApiTool"],
) => {
  switch (tool.data.type) {
    case "custom_api":
      return tool.data.config.name;
    case "code_runner":
      return "Code Runner";
    case "web_search":
      return "Web Search";
    case "system_info":
      return "System Info";
    case "files":
      return "Files";
    default:
      return "Unknown";
  }
};

export function ToolsManager({
  className,
  ...props
}: React.ComponentProps<"div">) {
  const { data: tools } = useTools();
  const deleteExternalApiTool = useDeleteExternalApiTool();

  const [isCreateDialogOpen, setIsCreateDialogOpen] = useState(false);
  const [selectedToolType, setSelectedToolType] =
    useState<ToolCreationType | null>(null);

  const systemTools = tools?.system || [];
  const externalApiTools = tools?.external_api || [];

  // Group tools by type for easier management
  const codeRunnerTool = systemTools.find((t) => t.data.type === "code_runner");
  const systemInfoTool = systemTools.find((t) => t.data.type === "system_info");
  const webSearchTool = externalApiTools.find(
    (t) => t.data.type === "web_search",
  );
  const customApiTools = externalApiTools.filter(
    (t) => t.data.type === "custom_api",
  );

  const handleCreateSuccess = () => {
    setIsCreateDialogOpen(false);
    setSelectedToolType(null);
  };

  const handleCreateCancel = () => {
    setSelectedToolType(null);
  };

  const handleDeleteExternalApiTool = async (toolId: string) => {
    try {
      await deleteExternalApiTool.mutateAsync(toolId);
    } catch (error) {
      console.error("Failed to delete external API tool:", error);
    }
  };

  const handleEnableSystemTool = (toolType: ToolCreationType) => {
    setSelectedToolType(toolType);
    setIsCreateDialogOpen(true);
  };

  const renderCreateForm = () => {
    switch (selectedToolType) {
      case "code_runner":
        return (
          <CodeRunnerToolForm
            onSuccess={handleCreateSuccess}
            onCancel={handleCreateCancel}
          />
        );
      case "system_info":
        return (
          <SystemInfoToolForm
            onSuccess={handleCreateSuccess}
            onCancel={handleCreateCancel}
          />
        );
      case "web_search":
        return (
          <WebSearchToolForm
            onSuccess={handleCreateSuccess}
            onCancel={handleCreateCancel}
          />
        );
      case "custom_api":
        return (
          <CustomApiToolForm
            onSuccess={handleCreateSuccess}
            onCancel={handleCreateCancel}
          />
        );
      default:
        return (
          <div className="space-y-4">
            <h3 className="text-lg font-medium">Select Tool Type</h3>
            <Select
              value={selectedToolType || ""}
              onValueChange={(value) =>
                setSelectedToolType(value as ToolCreationType)
              }
            >
              <SelectTrigger>
                <SelectValue placeholder="Choose a tool type" />
              </SelectTrigger>
              <SelectContent>
                {!codeRunnerTool && (
                  <SelectItem value="code_runner">
                    {TOOL_TYPE_LABELS.code_runner}
                  </SelectItem>
                )}
                {!systemInfoTool && (
                  <SelectItem value="system_info">
                    {TOOL_TYPE_LABELS.system_info}
                  </SelectItem>
                )}
                {!webSearchTool && (
                  <SelectItem value="web_search">
                    {TOOL_TYPE_LABELS.web_search}
                  </SelectItem>
                )}
                <SelectItem value="custom_api">
                  {TOOL_TYPE_LABELS.custom_api}
                </SelectItem>
              </SelectContent>
            </Select>
          </div>
        );
    }
  };

  return (
    <div className={cn("space-y-6", className)} {...props}>
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold">Tools (Beta)</h2>
          <p className="text-gray-600">
            Manage your system tools and external API integrations
          </p>
        </div>
        <Dialog open={isCreateDialogOpen} onOpenChange={setIsCreateDialogOpen}>
          <DialogTrigger asChild>
            <Button>
              <Plus className="size-4 mr-2" />
              Add Tool
            </Button>
          </DialogTrigger>
          <DialogContent className="max-w-4xl max-h-[80vh] overflow-y-auto">
            <DialogHeader>
              <DialogTitle>Create New Tool</DialogTitle>
              <DialogDescription>
                Add a new tool to extend your assistant's capabilities
              </DialogDescription>
            </DialogHeader>
            {renderCreateForm()}
          </DialogContent>
        </Dialog>
      </div>

      {/* System Tools Section */}
      <div className="space-y-4">
        <h3 className="text-lg font-semibold">System Tools</h3>
        <div className="grid gap-4 md:grid-cols-2">
          {/* Code Runner Tool */}
          <Card>
            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
              <div className="flex items-center gap-2">
                <Code2 className="size-5" />
                <CardTitle className="text-base">Code Runner</CardTitle>
              </div>
              {!codeRunnerTool && (
                <Button
                  variant="outline"
                  size="sm"
                  onClick={() => handleEnableSystemTool("code_runner")}
                >
                  <Plus className="size-4 mr-1" />
                  Enable
                </Button>
              )}
            </CardHeader>
            <CardContent>
              <CardDescription>
                Execute code in various programming languages
              </CardDescription>
              {codeRunnerTool && codeRunnerTool.data.type === "code_runner" && (
                <div className="mt-2 text-xs text-gray-500">
                  Timeout: {codeRunnerTool.data.config.timeout_seconds}s,
                  Memory: {codeRunnerTool.data.config.memory_limit_mb}MB, CPU:{" "}
                  {codeRunnerTool.data.config.cpu_limit}
                </div>
              )}
            </CardContent>
          </Card>

          {/* System Info Tool */}
          <Card>
            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
              <div className="flex items-center gap-2">
                <Wrench className="size-5" />
                <CardTitle className="text-base">System / Time</CardTitle>
              </div>
              {!systemInfoTool && (
                <Button
                  variant="outline"
                  size="sm"
                  onClick={() => handleEnableSystemTool("system_info")}
                >
                  <Plus className="size-4 mr-1" />
                  Enable
                </Button>
              )}
            </CardHeader>
            <CardContent>
              <CardDescription>
                Get system information, current date and time, etc.
              </CardDescription>
            </CardContent>
          </Card>
        </div>
      </div>

      {/* External API Tools Section */}
      <div className="space-y-4">
        <h3 className="text-lg font-semibold">External API Tools</h3>

        {/* Web Search Tool */}
        <div>
          <h4 className="text-md font-medium mb-2">Web Search</h4>
          {webSearchTool ? (
            <Card>
              <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                <div className="flex items-center gap-2">
                  <Globe className="size-5" />
                  <CardTitle className="text-base">Web Search</CardTitle>
                </div>
                <AlertDialog>
                  <AlertDialogTrigger asChild>
                    <Button variant="ghost" size="sm">
                      <Trash2 className="size-4" />
                    </Button>
                  </AlertDialogTrigger>
                  <AlertDialogContent>
                    <AlertDialogHeader>
                      <AlertDialogTitle>
                        Remove Web Search Tool
                      </AlertDialogTitle>
                      <AlertDialogDescription>
                        Are you sure you want to remove this tool? This action
                        cannot be undone.
                      </AlertDialogDescription>
                    </AlertDialogHeader>
                    <AlertDialogFooter>
                      <AlertDialogCancel>Cancel</AlertDialogCancel>
                      <AlertDialogAction
                        onClick={() =>
                          handleDeleteExternalApiTool(webSearchTool.id)
                        }
                      >
                        Remove
                      </AlertDialogAction>
                    </AlertDialogFooter>
                  </AlertDialogContent>
                </AlertDialog>
              </CardHeader>
              <CardContent>
                <CardDescription>
                  Search the web and extract content
                </CardDescription>
                {webSearchTool.data.type === "web_search" && (
                  <div className="mt-2 text-xs text-gray-500">
                    Provider: {webSearchTool.data.config.provider.type}, Max
                    Results: {webSearchTool.data.config.count}, Max Characters:{" "}
                    {webSearchTool.data.config.max_characters}
                  </div>
                )}
              </CardContent>
            </Card>
          ) : (
            <Card className="border-dashed">
              <CardContent className="flex items-center justify-center p-6">
                <Button
                  variant="outline"
                  onClick={() => handleEnableSystemTool("web_search")}
                >
                  <Plus className="size-4 mr-2" />
                  Add Web Search Tool
                </Button>
              </CardContent>
            </Card>
          )}
        </div>

        {/* Custom API Tools */}
        <div>
          <div className="flex items-center justify-between mb-2">
            <h4 className="text-md font-medium">Custom APIs</h4>
            <Button
              variant="outline"
              size="sm"
              onClick={() => handleEnableSystemTool("custom_api")}
            >
              <Plus className="size-4 mr-1" />
              Add Custom API
            </Button>
          </div>

          {customApiTools.length > 0 ? (
            <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
              {customApiTools.map(
                (tool) =>
                  tool.data.type === "custom_api" && (
                    <Card key={tool.id}>
                      <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                        <div className="flex items-center gap-2">
                          <CloudCog className="size-5" />
                          <CardTitle className="text-base truncate">
                            {tool.data.config.name}
                          </CardTitle>
                        </div>
                        <AlertDialog>
                          <AlertDialogTrigger asChild>
                            <Button variant="ghost" size="sm">
                              <Trash2 className="size-4" />
                            </Button>
                          </AlertDialogTrigger>
                          <AlertDialogContent>
                            <AlertDialogHeader>
                              <AlertDialogTitle>
                                Delete {tool.data.config.name}
                              </AlertDialogTitle>
                              <AlertDialogDescription>
                                Are you sure you want to delete this custom API
                                tool? This action cannot be undone.
                              </AlertDialogDescription>
                            </AlertDialogHeader>
                            <AlertDialogFooter>
                              <AlertDialogCancel>Cancel</AlertDialogCancel>
                              <AlertDialogAction
                                onClick={() =>
                                  handleDeleteExternalApiTool(tool.id)
                                }
                              >
                                Remove
                              </AlertDialogAction>
                            </AlertDialogFooter>
                          </AlertDialogContent>
                        </AlertDialog>
                      </CardHeader>
                      <CardContent>
                        <CardDescription>
                          {Object.keys(tool.data.config.tools).length} HTTP
                          request
                          {Object.keys(tool.data.config.tools).length !== 1
                            ? "s"
                            : ""}
                        </CardDescription>
                        <div className="mt-2 text-xs text-gray-500">
                          Requests:{" "}
                          {Object.keys(tool.data.config.tools).join(", ")}
                        </div>
                      </CardContent>
                    </Card>
                  ),
              )}
            </div>
          ) : (
            <Card className="border-dashed">
              <CardContent className="flex items-center justify-center p-6">
                <div className="text-center">
                  <CloudCog className="size-8 mx-auto mb-2 text-gray-400" />
                  <p className="text-sm text-gray-600 mb-3">
                    No custom API tools yet
                  </p>
                  <Button
                    variant="outline"
                    onClick={() => handleEnableSystemTool("custom_api")}
                  >
                    <Plus className="size-4 mr-2" />
                    Create Your First Custom API
                  </Button>
                </div>
              </CardContent>
            </Card>
          )}
        </div>
      </div>
    </div>
  );
}
