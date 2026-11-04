//! Render Tera v2 templates with Yatera's extra case-conversion filters.
//!
//! ```
//! use std::collections::HashMap;
//!
//! let vars = HashMap::from([("name".to_owned(), "Hello World")]);
//! let output = yatera::render("{{ name | snake_case }}", &vars)?;
//! assert_eq!(output, "hello_world");
//! # Ok::<(), yatera::Error>(())
//! ```

use serde::Serialize;
use std::collections::HashMap;
use std::hash::BuildHasher;
use tera::Tera;

mod filters;
mod plugins;

pub use tera::{Context, Error};

/// Render a template using a map of variables, returning an owned string.
///
/// Values may be strings or any serializable values (for example,
/// `serde_json::Value` for mixed types). The map is borrowed and remains unchanged.
/// Includes Tera's built-ins and Yatera's extra filters. HTML autoescaping is
/// disabled and no newline is added.
///
/// # Errors
///
/// Returns an error if variables cannot be serialized, or the template cannot
/// be parsed or rendered.
pub fn render<V: Serialize, S: BuildHasher>(
    template: &str,
    vars: &HashMap<String, V, S>,
) -> Result<String, Error> {
    render_with_context(template, &Context::from_serialize(vars)?)
}

/// Render using an existing Tera context, with the same filters and output
/// behavior as [`render`]. Returns an error if parsing or rendering fails.
pub fn render_with_context(template: &str, context: &Context) -> Result<String, Error> {
    render_with_context_and_plugins(template, context, Vec::new())
}

/// Render with Lua plugin sources, evaluated in order in a fresh runtime.
/// Plugins define `yatera.filters`, `yatera.functions`, and `yatera.tests` entries.
/// See the crate README for the Lua API and value conversion rules.
pub fn render_with_plugins<V: Serialize, S: BuildHasher>(
    template: &str,
    vars: &HashMap<String, V, S>,
    plugins: Vec<String>,
) -> Result<String, Error> {
    render_with_context_and_plugins(template, &Context::from_serialize(vars)?, plugins)
}

/// Like [`render_with_plugins`], using an existing Tera context.
pub fn render_with_context_and_plugins(
    template: &str,
    context: &Context,
    plugins: Vec<String>,
) -> Result<String, Error> {
    let mut tera = Tera::default();
    filters::register_filters(&mut tera);
    let _runtime = plugins::register(&mut tera, context, &plugins)?;
    tera.render_str(template, context, false)
}
