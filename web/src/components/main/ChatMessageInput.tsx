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

interface Props {
  sessionId?: string;
  isGenerating: boolean;
  onSubmit: (input: components["schemas"]["SendChatInput"]) => void;
}

export type ProviderKey = "Anthropic" | "OpenAI" | "OpenRouter" | "Lorem";

export default function ChatMessageInput({
  sessionId,
  isGenerating,
  onSubmit,
}: Props) {
  const [provider, setProvider] = useState<ProviderKey>();
  const [model, setModel] = useState("");
  const [error, setError] = useState<string>("");

  const formRef = useRef<HTMLFormElement>(null);
  const inputRef = useRef<HTMLTextAreaElement>(null);
  useEffect(() => {
    if (sessionId) inputRef.current?.focus(); // Focus input when switching session
  }, [sessionId]);

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
            Llm: { backend: provider, model, max_tokens: 1000 },
          },
        });
        break;
      case "OpenRouter":
        onSubmit({
          message: inputRef.current.value,
          provider: {
            OpenRouter: { model },
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
        <Button
          type="button"
          variant={enterKeyShouldSubmit ? "default" : "outline"}
          size="icon"
          title="Toggle Enter key behavior"
          onClick={() => setEnterKeyShouldSubmit((prev) => !prev)}
        >
          <CornerDownLeft className="size-3.5" />
          <span className="sr-only">Toggle Enter key</span>
        </Button>

        <ChatModelSelect
          currentProviderKey={provider}
          currentModel={model}
          onSelect={(provider, model) => {
            setProvider(provider);
            setModel(model || "");
          }}
        />
        {error && <div className="text-sm text-destructive">{error}</div>}

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
      </div>
    </form>
  );
}
