import type { components } from "./api/types";

export function getToolFromToolCall(
  toolCall:
    | components["schemas"]["ChatRsToolCall"]
    | components["schemas"]["ChatRsExecutedToolCall"],
  tools?: components["schemas"]["GetAllToolsResponse"],
) {
  switch (toolCall.tool_type) {
    case "system":
      return tools?.system.find((tool) => tool.id === toolCall?.tool_id);
    default:
      return tools?.external_api.find((tool) => tool.id === toolCall?.tool_id);
  }
}
