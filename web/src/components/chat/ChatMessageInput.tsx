import { CornerDownLeft } from "lucide-react";
import {
  type FormEventHandler,
  memo,
  useCallback,
  useMemo,
  useState,
} from "react";

import { Button } from "@/components/ui/button";
import { ChatInput } from "@/components/ui/chat/chat-input";
import type { useChatInputState } from "@/hooks/useChatInputState";
import { useProviders } from "@/lib/api/provider";
import { useTools } from "@/lib/api/tool";
import {
  ChatModelSelect,
  ChatMoreSettings,
  ChatProviderSelect,
  ChatToolSelect,
} from "./settings";

/** Handles submitting the user message, along with the current provider/model selection and other settings */
export default memo(function ChatMessageInput({
  inputState,
}: {
  inputState: ReturnType<typeof useChatInputState>;
}) {
  const { data: providers } = useProviders();
  const { data: tools } = useTools();

  const {
    providerId,
    modelId,
    toolIds,
    maxTokens,
    temperature,
    error,
    inputRef,
    formRef,
    isGenerating,
    onSelectModel,
    onToggleTool,
    setMaxTokens,
    setTemperature,
    canGetAgenticResponse,
    onSubmitUserMessage,
    onSubmitWithoutUserMessage,
  } = inputState;

  const currentProvider = useMemo(() => {
    return providers?.find((p) => p.id === providerId);
  }, [providers, providerId]);

  const setCurrentModel = useCallback(
    (model: string) => onSelectModel(providerId, model),
    [providerId, onSelectModel],
  );

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
        <ChatProviderSelect
          onSelectModel={onSelectModel}
          currentProvider={currentProvider}
          providers={providers}
        />
        {currentProvider && currentProvider.provider_type !== "lorem" && (
          <>
            <ChatModelSelect
              providerId={providerId}
              currentModelId={modelId}
              onSelect={setCurrentModel}
            />
            <ChatMoreSettings
              currentMaxTokens={maxTokens}
              currentTemperature={temperature}
              onSelectMaxTokens={setMaxTokens}
              onSelectTemperature={setTemperature}
            />
            <ChatToolSelect
              tools={tools}
              selectedToolIds={toolIds}
              toggleTool={onToggleTool}
            />
          </>
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
        {error && (
          <div className="text-sm text-destructive-foreground">{error}</div>
        )}

        <div className="ml-auto flex gap-2 items-center">
          {canGetAgenticResponse && (
            <Button
              type="button"
              size="sm"
              disabled={isGenerating}
              onClick={onSubmitWithoutUserMessage}
            >
              Get Agent Response
            </Button>
          )}
          <Button
            disabled={isGenerating}
            type="submit"
            size="sm"
            className="gap-1.5 flex items-center"
          >
            Send Message
            {!enterKeyShouldSubmit && <kbd> Shift + </kbd>}
            <CornerDownLeft className="size-3.5" />
          </Button>
        </div>
      </div>
    </form>
  );
});
