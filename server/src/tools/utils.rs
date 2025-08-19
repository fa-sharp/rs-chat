mod http_request_builder;

use schemars::{gen::SchemaSettings, JsonSchema};

use super::{ToolError, ToolJsonSchema, ToolResult};

pub use http_request_builder::HttpRequestBuilder;

/// Get the JSON schema for a given type.
pub fn get_json_schema<T: JsonSchema>() -> serde_json::Value {
    let settings = SchemaSettings::draft07().with(|s| {
        s.inline_subschemas = true; // Enable inline subschemas for compatibility with LLM providers
        s.meta_schema = None;
    });
    let schema = settings.into_generator().into_root_schema_for::<T>();
    serde_json::to_value(schema).expect("Should be valid JSON")
}

/// Ensure JSON schema is valid (using Draft 2020-12).
/// Also sets `additionalProperties` to false as required by OpenAI.
pub fn validate_json_schema(input_schema: &mut ToolJsonSchema) -> ToolResult<()> {
    input_schema.additional_properties = Some(false);
    jsonschema::draft202012::meta::validate(&serde_json::to_value(input_schema)?)
        .map_err(|e| ToolError::InvalidJsonSchema(e.to_string()))?;
    Ok(())
}
