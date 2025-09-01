import { Badge } from "@/components/ui/badge";

export default function ChatSettingsBadge({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <Badge className="absolute top-[-4px] right-[-4px] h-4 min-w-4 rounded-full px-1 font-mono tabular-nums">
      {children}
    </Badge>
  );
}
