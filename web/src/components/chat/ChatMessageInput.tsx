import { CornerDownLeft } from "lucide-react";
import { type FormEventHandler, useCallback, useState } from "react";

import type { useChatInputState } from "@/hooks/useChatInputState";
import { Button } from "../ui/button";
import { ChatInput } from "../ui/chat/chat-input";
import { ChatModelSelect } from "./ChatModelSelect";

/** Handles submitting the user message, and the current provider and model selection */
export default function ChatMessageInput({
  inputState,
}: {
  inputState: ReturnType<typeof useChatInputState>;
}) {
  const {
    providerId,
    modelId,
    maxTokens,
    temperature,
    error,
    inputRef,
    formRef,
    isGenerating,
    onSelectModel,
    setMaxTokens,
    setTemperature,
    onSubmitUserMessage,
  } = inputState;

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
          currentProviderId={providerId}
          currentModel={modelId}
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
          type="button"
          variant="outline"
          size="icon"
          title="Toggle Enter key behavior"
          onClick={() => setEnterKeyShouldSubmit((prev) => !prev)}
        >
          <CornerDownLeft className="size-3.5" />
          <span className="sr-only">Toggle Enter key</span>
        </Button>
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
