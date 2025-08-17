pub mod http_request_builder;

use super::{ToolError, ToolJsonSchema, ToolResult};

/// Ensure JSON schema is valid (using Draft 2020-12).
/// Also sets `additionalProperties` to false as required by OpenAI.
pub(super) fn validate_json_schema(input_schema: &mut ToolJsonSchema) -> ToolResult<()> {
    input_schema.additional_properties = Some(false);
    jsonschema::draft202012::meta::validate(&serde_json::to_value(input_schema)?)
        .map_err(|e| ToolError::InvalidJsonSchema(e.to_string()))?;
    Ok(())
}
