# Yatera

Yet another [Tera template v2](https://keats.github.io/tera/) CLI.

It features [rich DX](#usage) and [extra built-in filters](#built-ins).

## Installation

### macOS/Linux

Install Yatera on macOS or Linux by running the following command in your terminal:

```bash
curl -fsSL https://yatera.fyi/install.sh | sh
```

### Windows

Install Yatera on Windows by running the following command in PowerShell:

```powershell
irm https://yatera.fyi/install.ps1 | iex
```

### Cargo Binstall

Install Yatera using [Cargo Binstall](https://github.com/cargo-bins/cargo-binstall) by running the following command in your terminal:

```bash
cargo binstall yatera
```

### Build from Source

To compile Yatera from source code, [install Rust](https://rust-lang.org/learn/get-started/) and then install Yatera using Cargo:

```bash
cargo install yatera
```

## Usage

Use the `tera` command to render templates from stdin:

```bash
# Render a template with variables
cat template.tera | tera var1=value var2=value > output.txt

# Use structured data as variables (JSON, TOML, or YAML)
cat template.tera | tera --json '{"hello":"world"}' > output.txt
cat template.tera | tera --json-file vars.json > output.txt

# Read and write files directly (can be used individually)
tera --from template.tera --to output.txt --json-file vars.json
```

### Arguments

The following arguments are available (also see `tera --help`):

| Argument                    | Description                                           |
| --------------------------- | ----------------------------------------------------- |
| `var1=value var2=value`     | Set template variables from the command line          |
| `--json '{"key":"value"}'`  | Set template variables from a JSON string             |
| `--json-file vars.json`     | Set template variables from a JSON file               |
| `--toml-file vars.toml`     | Set template variables from a TOML file               |
| `--toml "$(cat vars.toml)"` | Set template variables from a TOML string             |
| `--yaml-file vars.yaml`     | Set template variables from a YAML file               |
| `--yaml "$(cat vars.yaml)"` | Set template variables from a YAML string             |
| `--from template.tera`      | Read the template from a file instead of stdin        |
| `--to output.txt`           | Write the rendered result to a file instead of stdout |

## Built-Ins

### Filters

#### heck

[heck](https://crates.io/crates/heck) methods are available as string filters:

```bash
printf '{{ name | to_snake_case }}' | tera name="Hello World"
#=> hello_world
```

| Filter                                   | Result for `Hello World` |
| ---------------------------------------- | ------------------------ |
| `to_upper_camel_case` / `to_pascal_case` | `HelloWorld`             |
| `to_lower_camel_case`                    | `helloWorld`             |
| `to_snake_case`                          | `hello_world`            |
| `to_kebab_case`                          | `hello-world`            |
| `to_shouty_snake_case`                   | `HELLO_WORLD`            |
| `to_shouty_kebab_case`                   | `HELLO-WORLD`            |
| `to_title_case`                          | `Hello World`            |
| `to_train_case`                          | `Hello-World`            |

## License

[MIT © Sasha Koss](https://koss.nocorp.me/mit/)
