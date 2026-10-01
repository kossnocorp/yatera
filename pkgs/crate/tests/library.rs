use std::collections::HashMap;

#[test]
fn renders_borrowed_variables_with_extra_filters() {
    let vars = HashMap::from([("name".to_owned(), "Hello World")]);
    let output: String = yatera::render("{{ name | snake_case }}", &vars).unwrap();
    assert_eq!(output, "hello_world");
    assert_eq!(vars["name"], "Hello World");
}

#[test]
fn renders_structured_values_and_preserves_output() {
    let vars = HashMap::from([
        ("user".to_owned(), serde_json::json!({"name": "<Ada>"})),
        ("enabled".to_owned(), serde_json::json!(true)),
        ("numbers".to_owned(), serde_json::json!([1, 2])),
    ]);
    assert_eq!(
        yatera::render(
            "{% if enabled %}{{ user.name }}:{% for n in numbers %}{{ n + 1 }}{% endfor %}{% endif %}\n",
            &vars,
        ).unwrap(),
        "<Ada>:23\n",
    );
    let empty = HashMap::<String, String>::new();
    assert_eq!(yatera::render("", &empty).unwrap(), "");
}

#[test]
fn returns_template_errors() {
    let vars = HashMap::<String, String>::new();
    assert!(yatera::render("{{", &vars).is_err());
    assert!(yatera::render("{{ missing }}", &vars).is_err());
}
