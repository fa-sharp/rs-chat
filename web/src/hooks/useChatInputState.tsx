import { useCallback, useEffect, useMemo, useRef, useState } from "react";

import type { components } from "@/lib/api/types";

const DEFAULT_MAX_TOKENS = 2000;
const DEFAULT_TEMPERATURE = 0.7;
const DEFAULT_TOOL_INPUT: {
  system: NonNullable<components["schemas"]["SendChatToolInput"]["system"]>;
  external_apis: NonNullable<
    components["schemas"]["SendChatToolInput"]["external_apis"]
  >;
} = {
  system: { code_runner: false, info: false },
  external_apis: [],
};

export const useChatInputState = ({
  sessionId,
  providers,
  initialProviderId,
  initialOptions,
  initialTools,
  isGenerating,
  canGetAgenticResponse,
  onSubmit,
}: {
  sessionId?: string;
  providers?: components["schemas"]["ChatRsProvider"][];
  initialProviderId?: number | null;
  initialOptions?: components["schemas"]["LlmApiProviderSharedOptions"] | null;
  initialTools?: components["schemas"]["SendChatToolInput"] | null;
  isGenerating: boolean;
  canGetAgenticResponse: boolean;
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
  const [toolInput, setToolInput] = useState<
    components["schemas"]["SendChatToolInput"] | null
  >(initialTools || DEFAULT_TOOL_INPUT);
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
    setToolInput(initialTools || null);
    setMaxTokens(initialOptions?.max_tokens ?? DEFAULT_MAX_TOKENS);
    setTemperature(initialOptions?.temperature ?? DEFAULT_TEMPERATURE);
    setError("");
  }, [initialProviderId, initialOptions, initialTools, sessionId]);

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

  type SystemToolInput = NonNullable<
    components["schemas"]["SendChatToolInput"]["system"]
  >;
  type SystemToolType = keyof SystemToolInput;
  const onSetSystemTool = useCallback(
    <T extends SystemToolType>(toolType: T, setting: SystemToolInput[T]) => {
      setToolInput((prevToolInput) => {
        const newSystemInput = {
          ...(prevToolInput?.system || DEFAULT_TOOL_INPUT.system),
        };
        newSystemInput[toolType] = setting;
        return { ...prevToolInput, system: newSystemInput };
      });
    },
    [],
  );

  type ExternalApiToolInput = NonNullable<
    components["schemas"]["SendChatToolInput"]["external_apis"]
  >[number];
  const onToggleExternalApiTool = useCallback(
    (toolInput: ExternalApiToolInput) => {
      setToolInput((prevToolInput) => {
        const newExternalApis = [...(prevToolInput?.external_apis || [])];
        const index = newExternalApis.findIndex(
          (api) => api.id === toolInput.id,
        );
        if (index === -1) newExternalApis.push(toolInput);
        else newExternalApis.splice(index, 1);

        return { ...prevToolInput, external_apis: newExternalApis };
      });
    },
    [],
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
      tools: toolInput,
    });
    formRef.current?.reset();
  }, [
    providerId,
    selectedProvider,
    modelId,
    toolInput,
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
      tools: toolInput,
    });
  }, [
    providerId,
    modelId,
    toolInput,
    temperature,
    maxTokens,
    onSubmit,
    isGenerating,
  ]);

  return useMemo(
    () => ({
      providerId,
      modelId,
      toolInput,
      maxTokens,
      temperature,
      error,
      inputRef,
      formRef,
      setMaxTokens,
      setTemperature,
      isGenerating,
      canGetAgenticResponse,
      onSelectModel,
      onSetSystemTool,
      onToggleExternalApiTool,
      onSubmitUserMessage,
      onSubmitWithoutUserMessage,
    }),
    [
      providerId,
      modelId,
      toolInput,
      maxTokens,
      temperature,
      error,
      isGenerating,
      canGetAgenticResponse,
      onSelectModel,
      onSetSystemTool,
      onToggleExternalApiTool,
      onSubmitUserMessage,
      onSubmitWithoutUserMessage,
    ],
  );
};
