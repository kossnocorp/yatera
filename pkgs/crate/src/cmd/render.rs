use crate::prelude::*;
use anyhow::{Context as _, bail, ensure};
use std::io::{self, Read, Write};
use std::path::PathBuf;
use tera::{Context, Tera};

#[derive(Args, Debug)]
pub struct YtrCmdRender {
    /// Read the template from a file instead of stdin
    #[usage(long)]
    pub from: Option<PathBuf>,
    /// Write the rendered result to a file instead of stdout (overwrites existing files)
    #[usage(long)]
    pub to: Option<PathBuf>,
    /// Variables as name=value strings; override structured context values
    #[usage(required = false)]
    pub arguments: Vec<String>,
    /// Context as a JSON object (choose one context source)
    #[usage(long)]
    pub json: Option<String>,
    /// Read context from a JSON file
    #[usage(long)]
    pub json_file: Option<PathBuf>,
    /// Context as a TOML table (choose one context source)
    #[usage(long)]
    pub toml: Option<String>,
    /// Read context from a TOML file
    #[usage(long)]
    pub toml_file: Option<PathBuf>,
    /// Context as a YAML mapping (choose one context source)
    #[usage(long)]
    pub yaml: Option<String>,
    /// Read context from a YAML file
    #[usage(long)]
    pub yaml_file: Option<PathBuf>,
}

impl Run for YtrCmdRender {
    type Output = Result<()>;

    fn run(self) -> Self::Output {
        let context = self.context()?;
        let template = if let Some(path) = &self.from {
            std::fs::read_to_string(path)
                .with_context(|| format!("Failed to read template file {}", path.display()))?
        } else {
            let mut template = String::new();
            io::stdin()
                .read_to_string(&mut template)
                .context("Failed to read template from stdin")?;
            template
        };
        let mut tera = Tera::default();
        register_filters(&mut tera);
        let rendered =
            tera.render_str(&template, &context, false)
                .with_context(|| match &self.from {
                    Some(path) => format!("Failed to render template file {}", path.display()),
                    None => "Failed to render template from stdin".to_owned(),
                })?;
        if let Some(path) = &self.to {
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
            ("json", self.json.as_deref(), self.json_file.as_deref()),
            ("toml", self.toml.as_deref(), self.toml_file.as_deref()),
            ("yaml", self.yaml.as_deref(), self.yaml_file.as_deref()),
        ];
        let count: usize = sources
            .iter()
            .map(|(_, inline, file)| usize::from(inline.is_some()) + usize::from(file.is_some()))
            .sum();
        ensure!(
            count <= 1,
            "Choose only one context source: --json, --json-file, --toml, --toml-file, --yaml, or --yaml-file"
        );

        let mut context = Context::new();
        for (format, inline, file) in sources {
            let contents;
            let input = if let Some(path) = file {
                contents = std::fs::read_to_string(path)
                    .with_context(|| format!("Failed to read context file {}", path.display()))?;
                contents.as_str()
            } else if let Some(input) = inline {
                input
            } else {
                continue;
            };
            context = parse_context(format, input).with_context(|| match file {
                Some(path) => format!("Invalid {format} context in {}", path.display()),
                None => format!("Invalid --{format} context"),
            })?;
        }

        for argument in &self.arguments {
            let Some((name, value)) = argument.split_once('=') else {
                bail!("Invalid variable {argument:?}: expected name=value");
            };
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
