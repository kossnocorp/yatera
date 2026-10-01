use crate::prelude::*;
use anyhow::{Context as _, bail, ensure};
use std::io::{self, Read, Write};
use std::path::PathBuf;
use yatera::Context;

#[derive(Args, Debug)]
pub struct YtrCmdRender {
    /// Read the template from a file instead of stdin
    #[usage(long)]
    pub in_file: Option<PathBuf>,
    /// Write the rendered result to a file instead of stdout (overwrites existing files)
    #[usage(long)]
    pub out_file: Option<PathBuf>,
    /// Variables as name=value strings; override structured context values
    #[usage(required = false)]
    pub arguments: Vec<String>,
    /// Set a string variable (alias: --arg); overrides context and name=value arguments
    #[usage(long = "var", visible_alias = "arg", num_args = 2, value_names = ["NAME", "VALUE"])]
    pub variables: Vec<String>,
    /// Context as a JSON object (choose one context source)
    #[usage(long)]
    pub json: Option<String>,
    /// Read context from a JSON file
    #[usage(long)]
    pub json_file: Option<PathBuf>,
    /// Read JSON context from the named environment variable
    #[usage(long)]
    pub json_env: Option<String>,
    /// Context as a TOML table (choose one context source)
    #[usage(long)]
    pub toml: Option<String>,
    /// Read context from a TOML file
    #[usage(long)]
    pub toml_file: Option<PathBuf>,
    /// Read TOML context from the named environment variable
    #[usage(long)]
    pub toml_env: Option<String>,
    /// Context as a YAML mapping (choose one context source)
    #[usage(long)]
    pub yaml: Option<String>,
    /// Read context from a YAML file
    #[usage(long)]
    pub yaml_file: Option<PathBuf>,
    /// Read YAML context from the named environment variable
    #[usage(long)]
    pub yaml_env: Option<String>,
}

impl Run for YtrCmdRender {
    type Output = Result<()>;

    fn run(self) -> Self::Output {
        let context = self.context()?;
        let template = if let Some(path) = &self.in_file {
            std::fs::read_to_string(path)
                .with_context(|| format!("Failed to read template file {}", path.display()))?
        } else {
            let mut template = String::new();
            io::stdin()
                .read_to_string(&mut template)
                .context("Failed to read template from stdin")?;
            template
        };
        let rendered =
            yatera::render_with_context(&template, &context).with_context(|| {
                match &self.in_file {
                    Some(path) => format!("Failed to render template file {}", path.display()),
                    None => "Failed to render template from stdin".to_owned(),
                }
            })?;
        if let Some(path) = &self.out_file {
            return std::fs::write(path, rendered).with_context(|| {
                format!("Failed to write rendered template to {}", path.display())
            });
        }
        match io::stdout().lock().write_all(rendered.as_bytes()) {
            Err(err) if err.kind() == io::ErrorKind::BrokenPipe => Ok(()),
            result => result.context("Failed to write rendered template to stdout"),
        }
    }
}

impl YtrCmdRender {
    fn context(&self) -> Result<Context> {
        let sources = [
            (
                "json",
                self.json.as_deref(),
                self.json_file.as_deref(),
                self.json_env.as_deref(),
            ),
            (
                "toml",
                self.toml.as_deref(),
                self.toml_file.as_deref(),
                self.toml_env.as_deref(),
            ),
            (
                "yaml",
                self.yaml.as_deref(),
                self.yaml_file.as_deref(),
                self.yaml_env.as_deref(),
            ),
        ];
        let count: usize = sources
            .iter()
            .map(|(_, inline, file, env)| {
                usize::from(inline.is_some())
                    + usize::from(file.is_some())
                    + usize::from(env.is_some())
            })
            .sum();
        ensure!(
            count <= 1,
            "Choose only one context source: --json, --json-file, --json-env, --toml, --toml-file, --toml-env, --yaml, --yaml-file, or --yaml-env"
        );

        let mut context = Context::new();
        for (format, inline, file, env) in sources {
            let contents;
            let input = if let Some(path) = file {
                contents = std::fs::read_to_string(path)
                    .with_context(|| format!("Failed to read context file {}", path.display()))?;
                contents.as_str()
            } else if let Some(name) = env {
                contents = std::env::var(name).with_context(|| {
                    format!("Failed to read context environment variable {name:?}")
                })?;
                contents.as_str()
            } else if let Some(input) = inline {
                input
            } else {
                continue;
            };
            context = parse_context(format, input).with_context(|| match (file, env) {
                (Some(path), _) => format!("Invalid {format} context in {}", path.display()),
                (_, Some(name)) => {
                    format!("Invalid {format} context in environment variable {name:?}")
                }
                _ => format!("Invalid --{format} context"),
            })?;
        }

        for argument in &self.arguments {
            let Some((name, value)) = argument.split_once('=') else {
                bail!("Invalid variable {argument:?}: expected name=value");
            };
            ensure!(!name.is_empty(), "Variable names must not be empty");
            context.insert(name.to_owned(), value);
        }
        ensure!(
            self.variables.len().is_multiple_of(2),
            "--var/--arg requires NAME VALUE pairs"
        );
        for [name, value] in self.variables.as_chunks::<2>().0 {
            ensure!(!name.is_empty(), "Variable names must not be empty");
            context.insert(name.to_owned(), value);
        }
        Ok(context)
    }
}

fn parse_context(format: &str, input: &str) -> Result<Context> {
    let context = match format {
        "json" => Context::from_serialize(&serde_json::from_str::<serde_json::Value>(input)?),
        "toml" => Context::from_serialize(&toml::from_str::<toml::Value>(input)?),
        "yaml" => Context::from_serialize(&serde_yaml_ng::from_str::<serde_yaml_ng::Value>(input)?),
        _ => unreachable!("unsupported context format"),
    };
    context.context("Context must be an object or mapping")
}
