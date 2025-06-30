import { CornerDownLeft } from "lucide-react";
import {
  type FormEventHandler,
  useCallback,
  useEffect,
  useRef,
  useState,
} from "react";

import type { components } from "@/lib/api/types";
import { type ProviderKey, providers } from "@/lib/providerInfo";
import { Button } from "../ui/button";
import { ChatInput } from "../ui/chat/chat-input";
import { ChatModelSelect } from "./ChatModelSelect";

interface Props {
  sessionId?: string;
  providerConfig?: components["schemas"]["ProviderConfigInput"] | null;
  isGenerating: boolean;
  onSubmit: (input: components["schemas"]["SendChatInput"]) => void;
}

/** Handles submitting the user message, and the current provider and model selection */
export default function ChatMessageInput(props: Props) {
  const {
    provider,
    model,
    maxTokens,
    temperature,
    error,
    inputRef,
    formRef,
    onSelectModel,
    setMaxTokens,
    setTemperature,
    onSubmitUserMessage,
  } = useChatMessageInputState(props);

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
          currentMaxTokens={maxTokens}
          currentTemperature={temperature}
          onSelect={onSelectModel}
          onSelectMaxTokens={setMaxTokens}
          onSelectTemperature={setTemperature}
        />
        {error && (
          <div className="text-sm text-destructive-foreground">{error}</div>
        )}

        <Button
          disabled={props.isGenerating}
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

const DEFAULT_MAX_TOKENS = 2000;
const DEFAULT_TEMPERATURE = 0.7;

const useChatMessageInputState = ({
  sessionId,
  providerConfig,
  isGenerating,
  onSubmit,
}: Props) => {
  const [provider, setProvider] = useState<ProviderKey | undefined>();
  const [model, setModel] = useState("");
  const [maxTokens, setMaxTokens] = useState<number>(DEFAULT_MAX_TOKENS);
  const [temperature, setTemperature] = useState<number>(DEFAULT_TEMPERATURE);
  const [error, setError] = useState<string>("");

  // Set initial provider, model, and settings for this session from the chat metadata
  useEffect(() => {
    if (sessionId && providerConfig) {
      const { provider, model, maxTokens, temperature } =
        getCommonSettingsFromConfig(providerConfig) || {};
      setProvider(provider);
      setModel(model || "");
      setMaxTokens(maxTokens || DEFAULT_MAX_TOKENS);
      setTemperature(temperature || DEFAULT_TEMPERATURE);
    }
    return () => {
      setProvider(undefined);
      setModel("");
      setMaxTokens(DEFAULT_MAX_TOKENS);
      setTemperature(DEFAULT_TEMPERATURE);
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
      case "OpenAI":
        onSubmit({
          message: inputRef.current.value,
          provider: {
            Llm: {
              backend: provider,
              model,
              temperature: temperature,
              max_tokens: maxTokens,
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
              temperature: temperature,
              max_tokens: maxTokens,
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
  }, [provider, model, temperature, maxTokens, onSubmit, isGenerating]);

  return {
    provider,
    model,
    maxTokens,
    temperature,
    error,
    inputRef,
    formRef,
    setMaxTokens,
    setTemperature,
    onSelectModel,
    onSubmitUserMessage,
  };
};

const getCommonSettingsFromConfig = (
  providerConfig?: components["schemas"]["ProviderConfigInput"] | null,
):
  | {
      provider: ProviderKey;
      model?: string | null;
      maxTokens?: number | null;
      temperature?: number | null;
    }
  | undefined => {
  if (!providerConfig) return undefined;
  if (typeof providerConfig === "string")
    return {
      provider: providerConfig,
      model: "",
    };
  if ("OpenRouter" in providerConfig)
    return {
      provider: "OpenRouter",
      model: providerConfig.OpenRouter.model,
      maxTokens: providerConfig.OpenRouter.max_tokens,
      temperature: providerConfig.OpenRouter.temperature,
    };
  if ("Llm" in providerConfig)
    return {
      provider: providerConfig.Llm.backend,
      model: providerConfig.Llm.model,
      maxTokens: providerConfig.Llm.max_tokens,
      temperature: providerConfig.Llm.temperature,
    };
};
