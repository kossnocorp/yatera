//! Browser bindings for the Yatera renderer.

use std::collections::HashMap;
use wasm_bindgen::prelude::*;

fn render_json(source: &str, input: &str) -> Result<String, String> {
    let vars: HashMap<String, serde_json::Value> =
        serde_json::from_str(input).map_err(|error| format!("Invalid JSON input: {error}"))?;
    yatera::render(source, &vars).map_err(|error| error.to_string())
}

/// Render Tera source with a JSON object containing template variables.
#[wasm_bindgen]
pub fn render(source: &str, input: &str) -> Result<String, JsValue> {
    render_json(source, input).map_err(|error| JsValue::from_str(&error))
}

/// Render with Lua plugin sources, evaluated in order.
#[wasm_bindgen]
pub fn render_with_plugins(
    source: &str,
    input: &str,
    plugins: Vec<String>,
) -> Result<String, JsValue> {
    let vars: HashMap<String, serde_json::Value> = serde_json::from_str(input)
        .map_err(|error| JsValue::from_str(&format!("Invalid JSON input: {error}")))?;
    yatera::render_with_plugins(source, &vars, plugins)
        .map_err(|error| JsValue::from_str(&error.to_string()))
}

#[cfg(test)]
mod tests {
    use super::render_json;

    #[test]
    fn renders_structured_input_and_yatera_filters() {
        assert_eq!(
            render_json(
                "{% for item in items %}{{ item.name | snake_case }}\n{% endfor %}",
                r#"{"items":[{"name":"Hello World"},{"name":"Wasm Playground"}]}"#,
            )
            .unwrap(),
            "hello_world\nwasm_playground\n"
        );
    }

    #[test]
    fn reports_invalid_input_and_templates() {
        for input in ["{", "[]", "null", "42"] {
            assert!(
                render_json("hello", input)
                    .unwrap_err()
                    .contains("Invalid JSON input")
            );
        }
        assert!(render_json("{{ missing }}", "{}").is_err());
        assert!(render_json("{% if %}", "{}").is_err());
    }
}
