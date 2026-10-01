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
    let mut tera = Tera::default();
    filters::register_filters(&mut tera);
    tera.render_str(template, context, false)
}
