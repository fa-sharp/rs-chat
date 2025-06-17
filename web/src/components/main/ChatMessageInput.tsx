import { CornerDownLeft } from "lucide-react";
import {
  useCallback,
  useEffect,
  useRef,
  useState,
  type FormEventHandler,
} from "react";
import { Button } from "../ui/button";
import { ChatInput } from "../ui/chat/chat-input";
import { ChatModelSelect } from "./ChatModelSelect";
import type { components } from "@/lib/api/types";
import { providers, type ProviderKey } from "@/lib/providerInfo";

interface Props {
  sessionId?: string;
  providerConfig?: components["schemas"]["ProviderConfigInput"] | null;
  isGenerating: boolean;
  onSubmit: (input: components["schemas"]["SendChatInput"]) => void;
}

export default function ChatMessageInput({
  sessionId,
  providerConfig,
  isGenerating,
  onSubmit,
}: Props) {
  const [provider, setProvider] = useState<ProviderKey | undefined>();
  const [model, setModel] = useState("");
  const [error, setError] = useState<string>("");

  // Set initial provider and model for this session from the chat metadata
  useEffect(() => {
    if (sessionId && providerConfig) {
      const { provider, model } =
        getProviderAndModelFromConfig(providerConfig) || {};
      setProvider(provider);
      setModel(model || "");
    }
    return () => {
      setProvider(undefined);
      setModel("");
    };
  }, [sessionId, providerConfig]);

  const formRef = useRef<HTMLFormElement>(null);
  const inputRef = useRef<HTMLTextAreaElement>(null);
  // Focus input when switching sessions
  useEffect(() => {
    if (sessionId) inputRef.current?.focus();
  }, [sessionId]);

  const onSelectModel = useCallback(
    (provider?: ProviderKey, model?: string) => {
      setProvider(provider);
      if (provider) {
        setModel(
          model ||
            providers.find((p) => p.value === provider)?.defaultModel ||
            "",
        );
      }
    },
    [],
  );

  const onSubmitUserMessage = useCallback(() => {
    if (isGenerating || !inputRef.current?.value) {
      return;
    }
    if (!model && provider !== "Lorem") {
      setError("Must select a model");
      return;
    }
    setError("");

    switch (provider) {
      case undefined:
        setError("Must select a provider");
        break;
      case "Anthropic":
        onSubmit({
          message: inputRef.current.value,
          provider: {
            Llm: {
              backend: provider,
              model,
              temperature: 0.7,
              max_tokens: 1000,
            },
          },
        });
        break;
      case "OpenRouter":
        onSubmit({
          message: inputRef.current.value,
          provider: {
            OpenRouter: {
              model,
              temperature: 0.7,
              max_tokens: 1000,
            },
          },
        });
        break;
      case "Lorem":
        onSubmit({
          message: inputRef.current.value,
          provider,
        });
        break;
    }
    formRef.current?.reset();
  }, [provider, model, onSubmit, isGenerating]);

  const [enterKeyShouldSubmit, setEnterKeyShouldSubmit] = useState(true);
  const onKeyDown = useCallback(
    (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
      if (
        (enterKeyShouldSubmit && e.key === "Enter" && !e.shiftKey) ||
        (!enterKeyShouldSubmit && e.key === "Enter" && e.shiftKey)
      ) {
        e.preventDefault();
        onSubmitUserMessage();
      }
    },
    [enterKeyShouldSubmit, onSubmitUserMessage],
  );

  const handleFormSubmit: FormEventHandler<HTMLFormElement> = useCallback(
    (ev) => {
      ev.preventDefault();
      onSubmitUserMessage();
    },
    [onSubmitUserMessage],
  );

  return (
    <form
      ref={formRef}
      onSubmit={handleFormSubmit}
      className="flex flex-col gap-2 relative rounded-lg border bg-background focus-within:ring-1 focus-within:ring-ring"
    >
      <ChatInput
        ref={inputRef}
        onKeyDown={onKeyDown}
        placeholder="Type your message..."
        className="rounded-lg bg-background text-foreground border-0 shadow-none focus-visible:ring-0"
      />
      <div className="flex flex-wrap items-center gap-2 p-3 pt-0">
        <ChatModelSelect
          currentProviderKey={provider}
          currentModel={model}
          onSelect={onSelectModel}
        />
        {error && (
          <div className="text-sm text-destructive-foreground">{error}</div>
        )}

        <Button
          disabled={isGenerating}
          type="submit"
          size="sm"
          className="ml-auto gap-1.5 flex items-center"
        >
          Send Message
          {!enterKeyShouldSubmit && <kbd> Shift + </kbd>}
          <CornerDownLeft className="size-3.5" />
        </Button>
        <Button
          type="button"
          variant="outline"
          size="sm"
          title="Toggle Enter key behavior"
          onClick={() => setEnterKeyShouldSubmit((prev) => !prev)}
        >
          <CornerDownLeft className="size-3.5" />
          <span className="sr-only">Toggle Enter key</span>
        </Button>
      </div>
    </form>
  );
}

const getProviderAndModelFromConfig = (
  providerConfig?: components["schemas"]["ProviderConfigInput"] | null,
): { provider: ProviderKey; model: string } | undefined => {
  if (!providerConfig) return undefined;
  if (typeof providerConfig === "string")
    return { provider: providerConfig, model: "" };
  if ("OpenRouter" in providerConfig)
    return { provider: "OpenRouter", model: providerConfig.OpenRouter.model };
  if ("Llm" in providerConfig)
    return {
      provider: providerConfig.Llm.backend,
      model: providerConfig.Llm.model,
    };
};
