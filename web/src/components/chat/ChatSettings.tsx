import * as React from "react";

import { useProviders } from "@/lib/api/provider";
import { useTools } from "@/lib/api/tool";
import ChatModelSelect from "./settings/ChatModelSelect";
import ChatMoreSettings from "./settings/ChatMoreSettings";
import ChatProviderSelect from "./settings/ChatProviderSelect";
import ChatToolSelect from "./settings/ChatToolSelect";

/** Provider, model, tool selection and other settings */
export default function ChatSettings({
  currentProviderId,
  currentModel,
  onSelectModel,
  currentMaxTokens,
  onSelectMaxTokens,
  currentTemperature,
  onSelectTemperature,
  currentToolIds,
  onToggleTool,
}: {
  currentProviderId?: number | null;
  currentModel: string;
  onSelectModel: (providerId?: number | null, model?: string) => void;
  currentMaxTokens: number;
  onSelectMaxTokens: (maxTokens: number) => void;
  currentTemperature: number;
  onSelectTemperature: (temperature: number) => void;
  currentToolIds: string[];
  onToggleTool: (toolId: string) => void;
}) {
  const { data: providers } = useProviders();
  const { data: tools } = useTools();

  const currentProvider = React.useMemo(() => {
    return providers?.find((p) => p.id === currentProviderId);
  }, [providers, currentProviderId]);

  const setCurrentModel = React.useCallback(
    (model: string) => onSelectModel(currentProviderId, model),
    [currentProviderId, onSelectModel],
  );

  return (
    <>
      <ChatProviderSelect
        onSelectModel={onSelectModel}
        currentProvider={currentProvider}
        providers={providers}
      />
      {currentProvider && currentProvider.provider_type !== "lorem" && (
        <>
          <ChatModelSelect
            providerId={currentProviderId}
            currentModelId={currentModel}
            onSelect={setCurrentModel}
          />
          <ChatMoreSettings
            currentMaxTokens={currentMaxTokens}
            currentTemperature={currentTemperature}
            onSelectMaxTokens={onSelectMaxTokens}
            onSelectTemperature={onSelectTemperature}
          />
          <ChatToolSelect
            tools={tools}
            selectedToolIds={currentToolIds}
            toggleTool={onToggleTool}
          />
        </>
      )}
    </>
  );
}
