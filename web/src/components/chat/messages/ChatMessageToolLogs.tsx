import { ChevronDown, ChevronUp } from "lucide-react";
import { useRef, useState } from "react";

import { Button } from "@/components/ui/button";
import { useAutoScroll } from "@/components/ui/chat/hooks/useAutoScroll";
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

  const contentRef = useRef<HTMLDivElement>(null);
  const { scrollRef } = useAutoScroll({ contentRef });

  return (
    <Collapsible open={showLogs} onOpenChange={setShowLogs}>
      <CollapsibleTrigger asChild>
        <Button variant="outline" size="sm" className="w-full justify-between">
          <span className="flex items-center gap-1">
            {showLogs ? <ChevronUp /> : <ChevronDown />}Logs ({logs.length})
          </span>
        </Button>
      </CollapsibleTrigger>
      <CollapsibleContent>
        <div ref={scrollRef} className="pt-2 max-h-32 overflow-auto">
          <div ref={contentRef} className="space-y-0.5">
            {logs.map((log, index) => (
              <div
                key={`log-${index}-${log.slice(0, 20)}`}
                className="rounded bg-muted/30 px-2 py-1 text-xs font-mono"
              >
                {log}
              </div>
            ))}
          </div>
        </div>
      </CollapsibleContent>
    </Collapsible>
  );
}
