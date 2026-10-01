# Yatera

Yet another [Tera template v2](https://keats.github.io/tera/) CLI.

It features [rich DX](#usage) and [extra built-ins](#built-ins).

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
# Render a template with variables:
cat template.tera | tera var1=value var2=value > output.txt

# Use structured data as variables (JSON/TOML/YAML):
cat template.tera | tera --json '{"var":"value"}' > output.txt
cat template.tera | tera --json-file vars.json > output.txt

# Read and write files directly:
tera --in-file template.tera --out-file output.txt --json-file vars.json
```

Environment sources parse the variable's contents as JSON, TOML, or YAML. Choose
one structured context source; `name=value` arguments override its values.
Missing variables or invalid context contents produce an error.

### Input

By default Yatera reads the template from stdin. You can override this behavior by using the `--in-file` option to specify a template file:

```bash
tera --in-file template.tera > output.txt
```

### Output

By default Yatera writes the rendered result to stdout. You can override this behavior by using the `--out-file` option to specify an output file:

```bash
tera --in-file template.tera --out-file output.txt
```

### Variables

Yatera supports multiple ways to provide template variables, from passing arguments to JSON/TOML/YAML strings or files.

#### Argument Variables

To provide template variables directly as command-line arguments, use the `name=value` format:

```bash
cat template.tera | tera var1=value var2=value > output.txt
```

Alternatively, use repeatable `--var NAME VALUE` flags, or the jq-compatible
`--arg NAME VALUE` alias:

```bash
cat template.tera | tera --var var1 value --var var2 value > output.txt
cat template.tera | tera --arg var1 value --arg var2 value > output.txt
```

#### Variable Files

To read variables from JSON/TOML/YAML files:

```bash
cat template.tera | tera --json-file vars.json > output.txt
```

**CLI options**:

| Argument                | Format |
| ----------------------- | ------ |
| `--json-file vars.json` | JSON   |
| `--toml-file vars.toml` | TOML   |
| `--yaml-file vars.yaml` | YAML   |

#### Variable Strings

To read variables from JSON/TOML/YAML strings:

```bash
cat template.tera | tera --json '{"var":"value"}' > output.txt
```

**CLI options**:

| Argument                    | Format |
| --------------------------- | ------ |
| `--json '{"var":"value"}'`  | JSON   |
| `--toml "$(cat vars.toml)"` | TOML   |
| `--yaml "$(cat vars.yaml)"` | YAML   |

##### Environment Variable Strings

Yatera also supports reading variables from JSON/TOML/YAML strings stored in environment variables:

```bash
export VARS='{"var":"value"}'
cat template.tera | tera --json-env VARS
```

**CLI options**:

| Argument          | Format |
| ----------------- | ------ |
| `--json-env VARS` | JSON   |
| `--toml-env VARS` | TOML   |
| `--yaml-env VARS` | YAML   |

## Built-Ins

Yatera supports all default [Tera built-ins](https://keats.github.io/tera/#built-ins) as well as extra filters.

### Extra Filters

#### heck

[heck](https://crates.io/crates/heck) methods are available as string filters:

```bash
printf '{{ name | snake_case }}' | tera name="Hello World"
#=> hello_world
```

| Filter                             | Result for `Hello World` |
| ---------------------------------- | ------------------------ |
| `upper_camel_case` / `pascal_case` | `HelloWorld`             |
| `lower_camel_case`                 | `helloWorld`             |
| `snake_case`                       | `hello_world`            |
| `kebab_case`                       | `hello-world`            |
| `shouty_snake_case`                | `HELLO_WORLD`            |
| `shouty_kebab_case`                | `HELLO-WORLD`            |
| `train_case`                       | `Hello-World`            |

## License

[MIT © Sasha Koss](https://koss.nocorp.me/mit/)
