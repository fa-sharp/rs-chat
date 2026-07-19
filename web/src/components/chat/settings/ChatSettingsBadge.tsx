import { Badge } from "@/components/ui/badge";

export default function ChatSettingsBadge({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <Badge className="absolute -top-1 -right-1 h-4 min-w-4 rounded-full px-1 font-mono tabular-nums">
      {children}
    </Badge>
  );
}
