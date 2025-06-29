import type { components } from "@/lib/api/types";

export type ProviderKey =
  | "Anthropic"
  | "OpenRouter"
  | "Lorem"
  | "OpenAI"
  | "Deepseek"
  | "Google";

/** Provider and model information. TODO should fetch models rather than list them manually */
export const providers: Array<{
  value: ProviderKey;
  apiKeyType?: components["schemas"]["ChatRsApiKeyProviderType"];
  label: string;
  defaultModel: string;
  models: string[];
}> = [
  {
    value: "Anthropic",
    apiKeyType: "Anthropic",
    label: "Anthropic",
    defaultModel: "claude-3-7-sonnet-latest",
    models: [
      "claude-opus-4-0",
      "claude-sonnet-4-0",
      "claude-3-7-sonnet-latest",
      "claude-3-5-haiku-latest",
      "claude-3-opus-latest",
    ],
  },
  {
    value: "OpenAI",
    label: "OpenAI",
    defaultModel: "gpt-4.1",
    models: [
      "gpt-4.1",
      "gpt-4.1-nano",
      "gpt-4o",
      "gpt-4o-mini",
      "o4-mini",
      "o3",
      "o3-pro",
      "o3-mini",
    ],
  },
  {
    value: "OpenRouter",
    label: "OpenRouter",
    apiKeyType: "Openrouter",
    defaultModel: "openai/gpt-4.1",
    models: [
      "openrouter/auto",
      "openai/gpt-4.1",
      "openai/gpt-4o-mini",
      "openai/o4-mini",
      "openai/o3-pro",
      "openai/o3-mini",
      "deepseek/deepseek-chat-v3-0324:free",
      "deepseek/deepseek-chat-v3-0324",
      "anthropic/claude-sonnet-4",
      "anthropic/claude-3.7-sonnet",
      "anthropic/claude-opus-4",
      "perplexity/sonar",
      "google/gemini-2.5-pro-preview",
      "google/gemini-2.0-flash-001",
    ],
  },
  {
    value: "Lorem",
    label: "Lorem ipsum",
    defaultModel: "",
    models: [],
  },
];
