import { useCallback, useEffect, useRef, useState } from "react";

import type { components } from "@/lib/api/types";
import { type ProviderKey, providers } from "@/lib/providerInfo";

const DEFAULT_MAX_TOKENS = 2000;
const DEFAULT_TEMPERATURE = 0.7;

export const useChatInputState = ({
  sessionId,
  providerConfig,
  isGenerating,
  onSubmit,
}: {
  sessionId?: string;
  providerConfig?: components["schemas"]["ProviderConfigInput"] | null;
  isGenerating: boolean;
  onSubmit: (input: components["schemas"]["SendChatInput"]) => void;
}) => {
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

    const message = inputRef.current?.value;

    switch (provider) {
      case undefined:
        setError("Must select a provider");
        break;
      case "OpenAI":
        onSubmit({
          message,
          provider: {
            OpenAI: {
              model,
              temperature,
              max_tokens: maxTokens,
            },
          },
        });
        break;
      case "Anthropic":
        onSubmit({
          message,
          provider: {
            Anthropic: {
              model,
              temperature,
              max_tokens: maxTokens,
            },
          },
        });
        break;
      case "OpenRouter":
        onSubmit({
          message,
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
          message,
          provider,
        });
        break;
    }
    formRef.current?.reset();
  }, [provider, model, temperature, maxTokens, onSubmit, isGenerating]);

  const onSubmitWithoutUserMessage = useCallback(() => {
    if (isGenerating) {
      return;
    }
    switch (provider) {
      case undefined:
        setError("Must select a provider");
        break;
      case "OpenAI":
        onSubmit({
          provider: {
            OpenAI: {
              model,
              temperature,
              max_tokens: maxTokens,
            },
          },
        });
        break;
      case "Anthropic":
        onSubmit({
          provider: {
            Anthropic: {
              model,
              temperature,
              max_tokens: maxTokens,
            },
          },
        });
        break;
      case "OpenRouter":
        onSubmit({
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
          provider,
        });
        break;
    }
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
    isGenerating,
    onSelectModel,
    onSubmitUserMessage,
    onSubmitWithoutUserMessage,
  };
};

const getCommonSettingsFromConfig = (
  providerConfig?: components["schemas"]["ProviderConfigInput"] | null,
):
  | {
      provider: ProviderKey;
      model: string;
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
  if ("OpenAI" in providerConfig)
    return {
      provider: "OpenAI",
      model: providerConfig.OpenAI.model,
      maxTokens: providerConfig.OpenAI.max_tokens,
      temperature: providerConfig.OpenAI.temperature,
    };
  if ("Anthropic" in providerConfig)
    return {
      provider: "Anthropic",
      model: providerConfig.Anthropic.model,
      maxTokens: providerConfig.Anthropic.max_tokens,
      temperature: providerConfig.Anthropic.temperature,
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
      provider: providerConfig.Llm.backend as ProviderKey,
      model: providerConfig.Llm.model,
      maxTokens: providerConfig.Llm.max_tokens,
      temperature: providerConfig.Llm.temperature,
    };
};
