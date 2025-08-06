import type { VariantProps } from "class-variance-authority";
import {
  AlertCircle,
  AlertTriangle,
  Check,
  Copy,
  Info,
  Trash2,
} from "lucide-react";
import { type FormEventHandler, useMemo, useState } from "react";

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
import type { components } from "@/lib/api/types";
import type { buttonVariants } from "../ui/button";
import { ChatBubbleAction } from "../ui/chat/chat-bubble";
import { Popover, PopoverContent, PopoverTrigger } from "../ui/popover";

export function CopyButton({
  message,
  variant = "ghost",
}: {
  message: string;
  variant?: VariantProps<typeof buttonVariants>["variant"];
}) {
  const [copied, setCopied] = useState(false);

  const handleClick = async () => {
    try {
      await navigator.clipboard.writeText(message);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch (error) {
      console.error("Failed to copy text: ", error);
    }
  };

  return (
    <ChatBubbleAction
      aria-label="Copy message"
      variant={variant}
      className="size-5"
      icon={
        copied ? (
          <Check className="size-5 text-green-600" />
        ) : (
          <Copy className="size-4" />
        )
      }
      onClick={handleClick}
    />
  );
}

export function DeleteButton({
  onDelete,
  variant = "ghost",
}: {
  onDelete: () => void;
  variant?: VariantProps<typeof buttonVariants>["variant"];
}) {
  const onSubmit: FormEventHandler<HTMLFormElement> = async (event) => {
    event.preventDefault();
    onDelete();
  };

  return (
    <AlertDialog>
      <AlertDialogTrigger asChild>
        <ChatBubbleAction
          aria-label="Delete message"
          variant={variant}
          className="size-5"
          icon={<Trash2 className="size-4" />}
        />
      </AlertDialogTrigger>
      <AlertDialogContent>
        <form onSubmit={onSubmit}>
          <AlertDialogHeader>
            <AlertDialogTitle>
              Are you sure you want to delete this message?
            </AlertDialogTitle>
            <AlertDialogDescription>
              This action cannot be undone.
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>Cancel</AlertDialogCancel>
            <AlertDialogAction variant="destructive" type="submit">
              Yes, delete
            </AlertDialogAction>
          </AlertDialogFooter>
        </form>
      </AlertDialogContent>
    </AlertDialog>
  );
}

export function InfoButton({
  meta,
}: {
  meta: components["schemas"]["ChatRsMessageMeta"];
}) {
  const { provider_options, interrupted, usage, executed_tool_call } = meta;

  return (
    <Popover>
      <PopoverTrigger asChild>
        <ChatBubbleAction
          className="size-5"
          icon={
            interrupted || executed_tool_call?.is_error ? (
              <AlertCircle className="size-4.5 text-yellow-700 dark:text-yellow-300" />
            ) : (
              <Info className="size-4.5" />
            )
          }
          variant="ghost"
          aria-label="Message metadata"
        />
      </PopoverTrigger>
      <PopoverContent className="text-sm">
        {interrupted && (
          <div className="flex items-center gap-1 mb-2">
            <AlertTriangle className="size-5 inline text-yellow-700 dark:text-yellow-300" />{" "}
            Stream was interrupted
          </div>
        )}
        {executed_tool_call?.is_error && (
          <div className="flex items-center gap-1 mb-2">
            <AlertTriangle className="size-5 inline text-yellow-700 dark:text-yellow-300" />{" "}
            Tool call error
          </div>
        )}
        {/*{provider && (
          <div>
            <span className="font-bold">Provider:</span> {provider}
          </div>
        )}*/}
        {provider_options?.model && (
          <div>
            <span className="font-bold">Model:</span> {provider_options.model}
          </div>
        )}
        {provider_options?.temperature && (
          <div>
            <span className="font-bold">Temperature:</span>{" "}
            {provider_options.temperature}
          </div>
        )}
        {executed_tool_call?.id && (
          <div>
            <span className="font-bold">Tool Call ID:</span>{" "}
            {executed_tool_call.id}
          </div>
        )}
        {executed_tool_call?.tool_id && (
          <div>
            <span className="font-bold">Tool ID:</span>{" "}
            {executed_tool_call.tool_id}
          </div>
        )}
        {usage?.input_tokens && (
          <div>
            <span className="font-bold">Input:</span>{" "}
            {usage.input_tokens?.toLocaleString()} tokens
          </div>
        )}
        {usage?.output_tokens && (
          <div>
            <span className="font-bold">Output:</span>{" "}
            {usage.output_tokens?.toLocaleString()} tokens
            {provider_options?.max_tokens
              ? ` (Max: ${provider_options.max_tokens.toLocaleString()})`
              : ""}
          </div>
        )}
        {usage?.cost && (
          <div>
            <span className="font-bold">Cost:</span> {usage.cost.toFixed(3)}
          </div>
        )}
      </PopoverContent>
    </Popover>
  );
}
