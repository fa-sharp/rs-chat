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
import { ChatBubbleAction } from "../ui/chat/chat-bubble";
import { Popover, PopoverContent, PopoverTrigger } from "../ui/popover";

export function CopyButton({ message }: { message: string }) {
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
      variant="ghost"
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

export function DeleteButton({ onDelete }: { onDelete: () => void }) {
  const onSubmit: FormEventHandler<HTMLFormElement> = async (event) => {
    event.preventDefault();
    onDelete();
  };

  return (
    <AlertDialog>
      <AlertDialogTrigger asChild>
        <ChatBubbleAction
          aria-label="Delete message"
          variant="ghost"
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
  const { provider, model, temperature, maxTokens, interrupted } =
    useMessageMeta(meta);

  return (
    <Popover>
      <PopoverTrigger asChild>
        <ChatBubbleAction
          className="size-5"
          icon={
            meta.interrupted ? (
              <AlertCircle className="size-4.5 text-yellow-700 dark:text-yellow-300" />
            ) : (
              <Info className="size-4.5" />
            )
          }
          variant="ghost"
          aria-label="Message metadata"
        />
      </PopoverTrigger>
      <PopoverContent>
        <div className="p-2">
          <p className="text-sm">
            {interrupted && (
              <div className="flex items-center gap-1 mb-2">
                <AlertTriangle className="size-5 inline text-yellow-700 dark:text-yellow-300" />{" "}
                Stream was interrupted
              </div>
            )}
            {provider && (
              <div>
                <span className="font-bold">Provider:</span> {provider}
              </div>
            )}
            {model && (
              <div>
                <span className="font-bold">Model:</span> {model}
              </div>
            )}
            {temperature && (
              <div>
                <span className="font-bold">Temperature:</span> {temperature}
              </div>
            )}
            {maxTokens && (
              <div>
                <span className="font-bold">Max Tokens:</span>{" "}
                {maxTokens.toLocaleString()}
              </div>
            )}
          </p>
        </div>
      </PopoverContent>
    </Popover>
  );
}

function useMessageMeta(meta: components["schemas"]["ChatRsMessageMeta"]) {
  const extractedMetadata = useMemo(() => {
    if (typeof meta.provider_config === "string") {
      return {
        provider: meta.provider_config,
        interrupted: !!meta.interrupted,
      };
    } else if (meta.provider_config && "Llm" in meta.provider_config) {
      return {
        provider: meta.provider_config.Llm.backend,
        model: meta.provider_config.Llm.model,
        temperature: meta.provider_config.Llm.temperature,
        interrupted: !!meta.interrupted,
        maxTokens: meta.provider_config.Llm.max_tokens,
      };
    } else if (meta.provider_config && "Anthropic" in meta.provider_config) {
      return {
        provider: "Anthropic",
        model: meta.provider_config.Anthropic.model,
        temperature: meta.provider_config.Anthropic.temperature,
        maxTokens: meta.provider_config.Anthropic.max_tokens,
        interrupted: !!meta.interrupted,
      };
    } else if (meta.provider_config && "OpenRouter" in meta.provider_config) {
      return {
        provider: "OpenRouter",
        model: meta.provider_config.OpenRouter.model,
        temperature: meta.provider_config.OpenRouter.temperature,
        interrupted: !!meta.interrupted,
        maxTokens: meta.provider_config.OpenRouter.max_tokens,
      };
    }

    return {
      interrupted: !!meta.interrupted,
    };
  }, [meta]);

  return extractedMetadata;
}
