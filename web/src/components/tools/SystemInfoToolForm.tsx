import { Wrench } from "lucide-react";
import { useState } from "react";

import { Button } from "@/components/ui/button";
import { useCreateTool } from "@/lib/api/tool";
import type { components } from "@/lib/api/types";

interface SystemInfoToolFormProps {
  onSuccess?: () => void;
  onCancel?: () => void;
}

export function SystemInfoToolForm({
  onSuccess,
  onCancel,
}: SystemInfoToolFormProps) {
  const createTool = useCreateTool();

  const [error, setError] = useState<string>("");
  const [isSubmitting, setIsSubmitting] = useState(false);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError("");
    setIsSubmitting(true);

    try {
      const toolInput: components["schemas"]["CreateToolInput"] = {
        system: {
          type: "system_info",
        },
      };

      await createTool.mutateAsync(toolInput);
      onSuccess?.();
    } catch (error) {
      console.error("Failed to create system info tool:", error);
      setError("Failed to create system info tool. Please try again.");
    } finally {
      setIsSubmitting(false);
    }
  };

  return (
    <form onSubmit={handleSubmit} className="space-y-4">
      <div className="flex items-center gap-2 mb-4">
        <Wrench className="size-5" />
        <h3 className="text-lg font-medium">System Info Tool</h3>
      </div>

      <div className="p-4 bg-blue-50 rounded-md">
        <p className="text-sm text-blue-800">
          This tool provides access to system information such as the current
          date/time and server details. No configuration is required.
        </p>
      </div>

      {error && (
        <div className="text-sm text-red-600 bg-red-50 p-3 rounded-md">
          {error}
        </div>
      )}

      <div className="flex gap-2 pt-4">
        <Button type="submit" disabled={isSubmitting}>
          {isSubmitting ? "Creating..." : "Create System Info Tool"}
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
