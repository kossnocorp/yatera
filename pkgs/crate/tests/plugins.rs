use serde_json::json;
use std::collections::HashMap;
use yatera::render_with_plugins;

#[test]
fn merges_plugin_context_with_input_in_order() {
    let mut context = yatera::Context::new();
    context.insert("home", "/home/test");
    context.insert("ssh_agent", "none");
    context.insert("kept", "from JSON");
    context.insert("env", &json!({"OLD": "replaced"}));
    context.insert("name", "original");
    let original = context.clone();
    let first = r#"
        local home = yatera.get("home")
        local ssh_agent = yatera.get("ssh_agent")
        yatera.context.env = { HOME = home }
        yatera.context.mise_env = ssh_agent == "none" and "" or ssh_agent
        yatera.context.name = "override"
        yatera.context.flag = false
        yatera.context.nothing = yatera.null
        yatera.context.items = {1, 2}
    "#;
    let second = r#"
        assert(yatera.get("name") == "override")
        assert(yatera.get("flag") == false)
        assert(yatera.get("nothing") == yatera.null)
        assert(yatera.context.env.OLD == nil)
        yatera.context.name = yatera.get("name") .. "!"
    "#;
    let output = yatera::render_with_context_and_plugins(
        "{{ env.HOME }}|{{ mise_env }}|{{ kept }}|{{ name }}|{{ flag }}|{{ items | length }}",
        &context,
        vec![first.to_owned(), second.to_owned()],
    )
    .unwrap();
    assert_eq!(output, "/home/test||from JSON|override!|false|2");
    assert_eq!(context, original);
}

#[test]
fn rejects_invalid_plugin_context() {
    for plugin in [
        "yatera.context = 42",
        "yatera.context = {1, 2}",
        "yatera.context.bad = function() end",
        "yatera.context.self = yatera.context",
    ] {
        assert!(
            render("hello", &[plugin])
                .unwrap_err()
                .to_string()
                .contains("context")
        );
    }
}

fn render(source: &str, plugins: &[&str]) -> Result<String, yatera::Error> {
    render_with_plugins(
        source,
        &HashMap::from([
            ("name".to_owned(), json!("Hello 世界")),
            (
                "data".to_owned(),
                json!({"empty": [], "items": [null, true, 42, {"key": "value"}]}),
            ),
        ]),
        plugins.iter().map(|s| s.to_string()).collect(),
    )
}

#[test]
fn filters_functions_tests_and_context() {
    let plugin = r#"
        yatera.filters.greet = function(value, args)
            return (args.prefix or "Hi") .. ": " .. value
        end
        yatera.functions.name = function(args)
            return yatera.get("name") .. args.suffix
        end
        yatera.tests.long = function(value, args) return #value >= args.min end
    "#;
    assert_eq!(
        render(
            r#"{{ name | greet(prefix="Hey") }}|{{ name(suffix="!") }}|{{ name is long(min=3) }}"#,
            &[plugin],
        )
        .unwrap(),
        "Hey: Hello 世界|Hello 世界!|true"
    );
}

#[test]
fn preserves_structured_values_and_null() {
    let plugin = r#"
        yatera.filters.identity = function(value, args) return value end
        yatera.functions.check = function(args)
            local data = yatera.get("data")
            assert(data.items[1] == yatera.null)
            assert(yatera.get("missing") == nil)
            return {ok = true, values = {1, 2, 3}}
        end
    "#;
    assert_eq!(render("{% set result = data | identity %}{% set checked = check() %}{{ result == data }}|{{ checked.ok }}", &[plugin]).unwrap(), "true|true");
}

#[test]
fn ordered_plugins_share_state_but_renders_are_isolated() {
    let first = "counter = 0; yatera.functions.next = function(args) counter = counter + 1; return counter end";
    let second = "assert(counter == 0); yatera.filters.extra = function(value, args) return value .. '!' end";
    for _ in 0..2 {
        assert_eq!(
            render(
                "{{ next() }}{{ next() }}{{ name | extra }}",
                &[first, second]
            )
            .unwrap(),
            "12Hello 世界!"
        );
    }
    assert!(render("{{ next() }}", &[]).is_err());
}

#[test]
fn reports_compile_load_callback_and_conversion_errors() {
    for (template, plugin, expected) in [
        ("hello", "function ???", "source 1"),
        ("hello", "error('load failure')", "load failure"),
        ("hello", "yatera.filters.bad = 42", "must be a function"),
        (
            "{{ bad() }}",
            "yatera.functions.bad = function(args) error('call failure') end",
            "call failure",
        ),
        (
            "{{ bad() }}",
            "yatera.functions.bad = function(args) local t = {}; t.self = t; return t end",
            "cyclic",
        ),
        (
            "{{ bad() }}",
            "yatera.functions.bad = function(args) return function() end end",
            "not a function",
        ),
        ("hello", "while true do end", "execution limit"),
        (
            "{{ bad() }}",
            "yatera.functions.bad = function(args) while true do end end",
            "execution limit",
        ),
    ] {
        let error = render(template, &[plugin]).unwrap_err().to_string();
        assert!(error.contains(expected), "{error}");
    }
}

#[test]
fn can_render_with_plugins_on_multiple_threads() {
    let threads: Vec<_> = (0..4)
        .map(|_| {
            std::thread::spawn(|| {
                render(
                    "{{ answer() }}",
                    &["yatera.functions.answer = function(args) return 42 end"],
                )
                .unwrap()
            })
        })
        .collect();
    for thread in threads {
        assert_eq!(thread.join().unwrap(), "42");
    }
}
