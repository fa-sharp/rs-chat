import { useState } from "react";

import { Button } from "@/components/ui/button";
import {
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
} from "@/components/ui/collapsible";

export default function ChatMessageToolLogs({
  logs,
  initialOpen,
}: {
  logs: string[];
  initialOpen?: boolean;
}) {
  const [showLogs, setShowLogs] = useState(initialOpen ?? false);

  return (
    <Collapsible open={showLogs} onOpenChange={setShowLogs}>
      <CollapsibleTrigger asChild>
        <Button variant="outline" size="sm" className="w-full justify-between">
          <span>Logs ({logs.length})</span>
          <span className="text-xs">{showLogs ? "Hide" : "Show"}</span>
        </Button>
      </CollapsibleTrigger>
      <CollapsibleContent className="space-y-1 pt-2">
        {logs.map((log, index) => (
          <div
            key={`log-${index}-${log.slice(0, 20)}`}
            className="rounded bg-muted/30 px-2 py-1 text-xs font-mono"
          >
            {log}
          </div>
        ))}
      </CollapsibleContent>
    </Collapsible>
  );
}
