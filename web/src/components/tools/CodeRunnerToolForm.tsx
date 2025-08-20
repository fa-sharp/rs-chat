import { Code2 } from "lucide-react";
import { useId, useState } from "react";

import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { useCreateTool } from "@/lib/api/tool";
import type { components } from "@/lib/api/types";

interface CodeRunnerToolFormProps {
  onSuccess?: () => void;
  onCancel?: () => void;
}

export function CodeRunnerToolForm({
  onSuccess,
  onCancel,
}: CodeRunnerToolFormProps) {
  const createTool = useCreateTool();

  const [timeoutSeconds, setTimeoutSeconds] = useState<number>(30);
  const [memoryLimitMb, setMemoryLimitMb] = useState<number>(512);
  const [cpuLimit, setCpuLimit] = useState<number>(0.5);
  const [error, setError] = useState<string>("");
  const [isSubmitting, setIsSubmitting] = useState(false);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError("");
    setIsSubmitting(true);

    // Validation
    if (timeoutSeconds < 1 || timeoutSeconds > 300) {
      setError("Timeout must be between 1 and 300 seconds");
      setIsSubmitting(false);
      return;
    }

    if (memoryLimitMb < 64 || memoryLimitMb > 2048) {
      setError("Memory limit must be between 64 and 2048 MB");
      setIsSubmitting(false);
      return;
    }

    if (cpuLimit < 0.1 || cpuLimit > 4.0) {
      setError("CPU limit must be between 0.1 and 4.0");
      setIsSubmitting(false);
      return;
    }

    try {
      const config: components["schemas"]["CodeRunnerConfig"] = {
        timeout_seconds: timeoutSeconds,
        memory_limit_mb: memoryLimitMb,
        cpu_limit: cpuLimit,
      };

      const toolInput: components["schemas"]["CreateToolInput"] = {
        system: {
          type: "code_runner",
          config,
        },
      };

      await createTool.mutateAsync(toolInput);
      onSuccess?.();
    } catch (error) {
      console.error("Failed to create code runner tool:", error);
      setError("Failed to create code runner tool. Please try again.");
    } finally {
      setIsSubmitting(false);
    }
  };

  const handleReset = () => {
    setTimeoutSeconds(30);
    setMemoryLimitMb(512);
    setCpuLimit(0.5);
    setError("");
  };

  const timeoutId = useId();
  const memoryLimitId = useId();
  const cpuLimitId = useId();

  return (
    <form onSubmit={handleSubmit} className="space-y-4">
      <div className="flex items-center gap-2 mb-4">
        <Code2 className="size-5" />
        <h3 className="text-lg font-medium">Code Runner Tool</h3>
      </div>

      <div className="space-y-3">
        <div>
          <Label htmlFor={timeoutId} className="mb-2">
            Timeout (seconds) - 5 to 60
          </Label>
          <Input
            id={timeoutId}
            type="number"
            min={5}
            max={60}
            value={timeoutSeconds}
            onChange={(e) => setTimeoutSeconds(Number(e.target.value))}
          />
          <p className="text-sm text-gray-500 mt-1">
            Maximum time allowed for code execution
          </p>
        </div>

        <div>
          <Label htmlFor={memoryLimitId} className="mb-2">
            Memory Limit (MB) - 100 to 1024
          </Label>
          <Input
            id={memoryLimitId}
            type="number"
            min={100}
            max={1024}
            value={memoryLimitMb}
            onChange={(e) => setMemoryLimitMb(Number(e.target.value))}
          />
          <p className="text-sm text-gray-500 mt-1">
            Maximum memory the code can use
          </p>
        </div>

        <div>
          <Label htmlFor={cpuLimitId} className="mb-2">
            CPU Limit (cores) - 0.1 to 1.2
          </Label>
          <Input
            id={cpuLimitId}
            type="number"
            min={0.1}
            max={1.2}
            step={0.1}
            value={cpuLimit}
            onChange={(e) => setCpuLimit(Number(e.target.value))}
          />
          <p className="text-sm text-gray-500 mt-1">
            Number of CPU cores the code can use
          </p>
        </div>
      </div>

      {error && (
        <div className="text-sm text-red-600 bg-red-50 p-3 rounded-md">
          {error}
        </div>
      )}

      <div className="flex gap-2 pt-4">
        <Button type="submit" disabled={isSubmitting}>
          {isSubmitting ? "Creating..." : "Create Code Runner Tool"}
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
