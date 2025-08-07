import { useCallback, useEffect, useMemo, useRef, useState } from "react";

import type { components } from "@/lib/api/types";

const DEFAULT_MAX_TOKENS = 2000;
const DEFAULT_TEMPERATURE = 0.7;

export const useChatInputState = ({
  sessionId,
  providers,
  initialProviderId,
  initialOptions,
  isGenerating,
  onSubmit,
}: {
  sessionId?: string;
  providers?: components["schemas"]["ChatRsProvider"][];
  initialProviderId?: number | null;
  initialOptions?: components["schemas"]["LlmApiProviderSharedOptions"] | null;
  isGenerating: boolean;
  onSubmit: (input: components["schemas"]["SendChatInput"]) => void;
}) => {
  const [providerId, setProviderId] = useState<number | null>(
    initialProviderId || null,
  );
  const selectedProvider = useMemo(
    () => providers?.find((p) => p.id === providerId),
    [providers, providerId],
  );
  const [modelId, setModel] = useState(initialOptions?.model || "");
  const [maxTokens, setMaxTokens] = useState<number>(
    initialOptions?.max_tokens ?? DEFAULT_MAX_TOKENS,
  );
  const [temperature, setTemperature] = useState<number>(
    initialOptions?.temperature ?? DEFAULT_TEMPERATURE,
  );
  const [error, setError] = useState<string>("");

  // Reset state when session changes
  useEffect(() => {
    if (!sessionId) return;
    setProviderId(initialProviderId || null);
    setModel(initialOptions?.model || "");
    setMaxTokens(initialOptions?.max_tokens ?? DEFAULT_MAX_TOKENS);
    setTemperature(initialOptions?.temperature ?? DEFAULT_TEMPERATURE);
    setError("");
  }, [initialProviderId, initialOptions, sessionId]);

  const formRef = useRef<HTMLFormElement>(null);
  const inputRef = useRef<HTMLTextAreaElement>(null);
  // Focus input when switching sessions
  useEffect(() => {
    if (sessionId) inputRef.current?.focus();
  }, [sessionId]);

  const onSelectModel = useCallback(
    (providerId?: number | null, modelId?: string) => {
      setProviderId(providerId ?? null);
      if (providerId) {
        setModel(
          modelId ||
            providers?.find((p) => p.id === providerId)?.default_model ||
            "",
        );
      }
    },
    [providers],
  );

  const onSubmitUserMessage = useCallback(() => {
    if (isGenerating || !inputRef.current?.value) {
      return;
    }
    if (!providerId) {
      setError("Must select a provider");
      return;
    }
    if (!modelId && selectedProvider?.provider_type !== "lorem") {
      setError("Must select a model");
      return;
    }
    setError("");

    onSubmit({
      message: inputRef.current?.value,
      provider_id: providerId,
      provider_options: {
        model: modelId,
        temperature,
        max_tokens: maxTokens,
      },
    });
    formRef.current?.reset();
  }, [
    providerId,
    selectedProvider,
    modelId,
    temperature,
    maxTokens,
    onSubmit,
    isGenerating,
  ]);

  const onSubmitWithoutUserMessage = useCallback(() => {
    if (isGenerating || !providerId) {
      return;
    }
    onSubmit({
      provider_id: providerId,
      provider_options: {
        model: modelId,
        temperature,
        max_tokens: maxTokens,
      },
    });
  }, [providerId, modelId, temperature, maxTokens, onSubmit, isGenerating]);

  return {
    providerId,
    modelId,
    maxTokens,
    temperature,
    error,
    inputRef,
    formRef,
    setMaxTokens,
    setTemperature,
    isGenerating,
    onSelectModel,
    onSubmitUserMessage,
    onSubmitWithoutUserMessage,
  };
};
