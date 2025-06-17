import { Bot } from "lucide-react";
import {
  Card,
  CardHeader,
  CardTitle,
  CardDescription,
  CardContent,
} from "./ui/card";

export default function ErrorComponent({ error }: { error?: Error }) {
  return (
    <div className="p-8">
      <Card
        className={
          "border-2 bg-red-100 dark:bg-red-900 border-red-300 dark:border-red-700"
        }
      >
        <CardHeader>
          <div className="flex items-center justify-between">
            <div>
              <CardTitle className="text-lg flex items-center gap-2">
                <Bot className="size-5" />
                We had an error 🥲
              </CardTitle>
              <CardDescription>Error details</CardDescription>
            </div>
          </div>
        </CardHeader>
        <CardContent>
          <div className="flex flex-col items-center gap-2">
            <div className="flex-1 font-mono text-sm bg-muted px-3 py-2 rounded border">
              {error?.message || "Unknown error"}
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
