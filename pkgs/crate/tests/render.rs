use std::io::Write;
use std::process::{Command, Output, Stdio};

fn render(template: &str, args: &[&str]) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_tera"))
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(template.as_bytes())
        .unwrap();
    child.wait_with_output().unwrap()
}

fn assert_render(template: &str, args: &[&str], expected: &str) {
    let output = render(template, args);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(String::from_utf8(output.stdout).unwrap(), expected);
    assert!(output.stderr.is_empty());
}

#[test]
fn renders_strings_and_preserves_output() {
    assert_render(
        "{{ first }}|{{ second }}|{{ empty }}",
        &["first=你好 <&>", "second=a=b", "empty="],
        "你好 <&>|a=b|",
    );
    assert_render("{{ value }}", &["value=first", "value=last"], "last");
    assert_render("unchanged\n", &[], "unchanged\n");
    assert_render("", &[], "");
}

const CONTEXTS: [(&str, &str); 3] = [
    (
        "json",
        r#"{"user":{"name":"Ada"},"enabled":true,"numbers":[1,2]}"#,
    ),
    (
        "toml",
        "enabled = true\nnumbers = [1, 2]\n[user]\nname = 'Ada'",
    ),
    ("yaml", "user:\n  name: Ada\nenabled: true\nnumbers: [1, 2]"),
];

#[test]
fn renders_heck_case_filters() {
    for (filter, expected) in [
        ("to_upper_camel_case", "XmlHttpRequest"),
        ("to_pascal_case", "XmlHttpRequest"),
        ("to_lower_camel_case", "xmlHttpRequest"),
        ("to_snake_case", "xml_http_request"),
        ("to_kebab_case", "xml-http-request"),
        ("to_shouty_snake_case", "XML_HTTP_REQUEST"),
        ("to_shouty_kebab_case", "XML-HTTP-REQUEST"),
        ("to_title_case", "Xml Http Request"),
        ("to_train_case", "Xml-Http-Request"),
    ] {
        assert_render(
            &format!("{{{{ name | {filter} }}}}"),
            &["name=XMLHttp request"],
            expected,
        );
        assert_render(&format!("{{{{ name | {filter} }}}}"), &["name="], "");
    }
    assert_render(
        "{{ name | to_snake_case | upper }}",
        &["name=Déjà Vu"],
        "DÉJÀ_VU",
    );
}

#[test]
fn case_filters_reject_non_strings() {
    for value in ["42", "true", "null", "[]", "{}"] {
        let output = render(
            "before {{ name | to_snake_case }}",
            &["--json", &format!(r#"{{"name":{value}}}"#)],
        );
        assert!(!output.status.success(), "accepted {value}");
        assert!(output.stdout.is_empty());
        assert!(String::from_utf8_lossy(&output.stderr).contains("Failed to render template"));
    }
}

const TEMPLATE: &str = "{% if enabled %}{{ user.name | upper }}:{% for n in numbers %}{{ n + 1 }}{% endfor %}{% endif %}";

#[test]
fn renders_typed_inline_contexts() {
    for (format, context) in CONTEXTS {
        assert_render(TEMPLATE, &[&format!("--{format}"), context], "ADA:23");
        assert_render(
            "{{ enabled }}",
            &[&format!("--{format}"), context, "enabled=override"],
            "override",
        );
    }
}

#[test]
fn renders_context_files() {
    let directory = tempfile::tempdir().unwrap();
    for (format, context) in CONTEXTS {
        let path = directory.path().join(format!("vars.{format}"));
        std::fs::write(&path, context).unwrap();
        assert_render(
            TEMPLATE,
            &[&format!("--{format}-file"), path.to_str().unwrap()],
            "ADA:23",
        );
    }
}

#[test]
fn reports_invalid_inputs_without_stdout() {
    for (template, args, message) in [
        ("", vec!["invalid"], "expected name=value"),
        ("", vec!["=value"], "must not be empty"),
        ("", vec!["--json", "{"], "Invalid --json context"),
        ("", vec!["--toml", "value ="], "Invalid --toml context"),
        ("", vec!["--yaml", "value: ["], "Invalid --yaml context"),
        ("", vec!["--json", "[]"], "object or mapping"),
        ("", vec!["--yaml", "hello"], "object or mapping"),
        (
            "",
            vec!["--json", "{}", "--yaml", "{}"],
            "Choose only one context source",
        ),
        (
            "",
            vec!["--json", "{}", "--json-file", "unused"],
            "Choose only one context source",
        ),
        (
            "",
            vec!["--json-file", "nonexistent-directory/vars.json"],
            "Failed to read context file",
        ),
        ("before {{", vec![], "Failed to render template"),
        ("before {{ missing }}", vec![], "Failed to render template"),
    ] {
        let output = render(template, &args);
        assert!(!output.status.success(), "{args:?}");
        assert!(output.stdout.is_empty());
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains(message),
            "expected {message:?}, got {stderr:?}"
        );
    }
}

#[test]
fn help_exposes_root_options() {
    let output = render("", &["--help"]);
    assert!(output.status.success());
    let help = String::from_utf8(output.stdout).unwrap();
    for flag in [
        "--from",
        "--to",
        "--json",
        "--json-file",
        "--toml",
        "--toml-file",
        "--yaml",
        "--yaml-file",
    ] {
        assert!(help.contains(flag));
    }
}

#[test]
fn renders_from_and_to_files_independently() {
    let directory = tempfile::tempdir().unwrap();
    let source = directory.path().join("template.ts.tera");
    let target = directory.path().join("module.ts");
    let from = source.to_str().unwrap();
    let to = target.to_str().unwrap();
    std::fs::write(
        &source,
        "export const {{ name | to_snake_case }} = {{ value }};",
    )
    .unwrap();
    assert_render(
        "ignored stdin",
        &["--from", from, "name=HelloWorld", "value=42"],
        "export const hello_world = 42;",
    );
    assert_render(
        "ignored stdin",
        &[
            "--from",
            from,
            "--to",
            to,
            "--json",
            r#"{"name":"HelloWorld","value":42}"#,
        ],
        "",
    );
    assert_eq!(
        std::fs::read_to_string(&target).unwrap(),
        "export const hello_world = 42;"
    );
    assert_render("{{ value }}", &["--to", to, "value=short"], "");
    assert_eq!(std::fs::read_to_string(&target).unwrap(), "short");
    assert_render("", &["--from", to, "--to", to], "");
    assert_eq!(std::fs::read_to_string(&target).unwrap(), "short");
}

#[test]
fn file_errors_preserve_existing_output() {
    let directory = tempfile::tempdir().unwrap();
    let source = directory.path().join("broken.tera");
    let target = directory.path().join("output.txt");
    let missing = directory.path().join("missing.tera");
    std::fs::write(&source, "before {{").unwrap();
    std::fs::write(&target, "existing output").unwrap();
    for (path, message) in [
        (&source, "Failed to render template file"),
        (&missing, "Failed to read template file"),
    ] {
        let output = render(
            "",
            &[
                "--from",
                path.to_str().unwrap(),
                "--to",
                target.to_str().unwrap(),
            ],
        );
        assert!(!output.status.success());
        assert!(output.stdout.is_empty());
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains(message), "{stderr}");
        assert!(stderr.contains(path.to_str().unwrap()), "{stderr}");
        assert_eq!(std::fs::read_to_string(&target).unwrap(), "existing output");
    }
    let output = render("rendered", &["--to", directory.path().to_str().unwrap()]);
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("Failed to write rendered template"));
}
