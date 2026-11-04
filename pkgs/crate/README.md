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

### CLI

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

#### Input

By default Yatera reads the template from stdin. You can override this behavior by using the `--in-file` option to specify a template file:

```bash
tera --in-file template.tera > output.txt
```

#### Output

By default Yatera writes the rendered result to stdout. You can override this behavior by using the `--out-file` option to specify an output file:

```bash
tera --in-file template.tera --out-file output.txt
```

#### Variables

Yatera supports multiple ways to provide template variables, from passing arguments to JSON/TOML/YAML strings or files.

##### Argument Variables

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

##### Variable Files

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

##### Variable Strings

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

###### Environment Variable Strings

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

## Lua Plugins

Load Lua plugins in order with repeatable `--plugin` flags:

```sh
tera --in-file template.tera --json-file vars.json \
  --plugin ./path/to/plugin1.lua --plugin ./path/to/plugin2.lua
```

Plugins register filters, functions, and tests by assigning functions to the
`yatera` tables:

```lua
-- {{ name | greet(prefix="Hi") }}
yatera.filters.greet = function(value, args)
  return (args.prefix or "Hello") .. ", " .. value .. "!"
end

-- {{ project_names() }}
yatera.functions.project_names = function(args)
  local names = ""
  for i, name in ipairs(yatera.get("projects") or {}) do
    if i > 1 then names = names .. ", " end
    names = names .. string.upper(name)
  end
  return names
end

-- {% if name is long(min=3) %}...{% endif %}
yatera.tests.long = function(value, args)
  return #value >= (args.min or 3)
end
```

### Runtime API

- `yatera.filters.NAME(value, args)`: receives the filtered value and named arguments.
- `yatera.functions.NAME(args)`: receives named arguments.
- `yatera.tests.NAME(value, args)`: receives the tested value and named arguments;
  must return a boolean.
- `yatera.get("name")`: reads an original render-input variable, or `nil` if absent.
  This includes CLI variable overrides, but not Tera loop variables or `{% set %}`
  assignments. Pass those as filter values or named arguments instead.
- `yatera.null`: represents an explicit JSON null, including within arrays/objects.
- Lua helpers include `pairs`, `ipairs`, `tostring`, `tonumber`, `assert`, `error`,
  and the runtime's `string`, `table`, and `math` libraries.

Yatera embeds [Piccolo](https://github.com/kyren/piccolo), a pure-Rust Lua runtime,
on both native and WebAssembly targets. Its core standard library is available;
it is not a complete implementation of every PUC Lua standard-library function.
File/process APIs and external module loading are not provided.

Objects become string-keyed Lua tables; arrays become 1-based tables; scalars keep
their types. Input arrays preserve their array identity, including empty arrays.
Returned consecutive integer-keyed tables become arrays, and string-keyed tables
become objects (an untagged empty `{}` is an object). Return `nil` or `yatera.null`
for null. Cycles, mixed/sparse table keys, non-finite numbers, and unsupported return
types produce render errors. Integers must fit Lua's signed 64-bit range.

Each render starts a fresh runtime. Plugins share globals within that render and
are evaluated in the order supplied; later definitions replace earlier ones,
including same-named built-ins. Top-level code and each callback have an execution
budget so runaway Lua loops produce errors. Syntax/runtime errors fail the render
without writing partial CLI output.

The [playground](https://yatera.fyi/playground) has a **Lua plugin** tab in the Input
pane. Its example is placeholder text only; enter code to activate a plugin.

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

### Rust Library

Add Yatera to your project with `cargo add yatera`, then render with a hash map:

```rust
use std::collections::HashMap;

fn main() -> Result<(), yatera::Error> {
    let vars = HashMap::from([("name".to_owned(), "Hello World")]);
    let output: String = yatera::render("{{ name | snake_case }}", &vars)?;
    assert_eq!(output, "hello_world");
    Ok(())
}
```

Values may be strings or other serializable values, such as `serde_json::Value` for mixed types.

For an existing `yatera::Context`, use:

```rs
yatera::render_with_context(template, &context)
```

Pass Lua source code as `Vec<String>` with `render_with_plugins`:

```rust
let plugins = vec![r#"
    yatera.filters.shout = function(value, args)
        return string.upper(value) .. "!"
    end
"#.to_owned()];
let output = yatera::render_with_plugins("{{ name | shout }}", &vars, plugins)?;
```

For an existing context, use
`yatera::render_with_context_and_plugins(template, &context, plugins)`.

## License

[MIT © Sasha Koss](https://koss.nocorp.me/mit/)
