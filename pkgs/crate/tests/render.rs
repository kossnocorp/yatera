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
